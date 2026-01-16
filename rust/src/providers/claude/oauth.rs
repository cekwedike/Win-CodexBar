//! Claude OAuth implementation

use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::Deserialize;

use crate::core::{OAuthCredentials, ProviderError, RateWindow};

/// OAuth usage response from Claude API
#[derive(Debug, Deserialize)]
pub struct OAuthUsageResponse {
    #[serde(rename = "fiveHour")]
    pub five_hour: Option<UsageWindow>,

    #[serde(rename = "sevenDay")]
    pub seven_day: Option<UsageWindow>,

    #[serde(rename = "sevenDaySonnet")]
    pub seven_day_sonnet: Option<UsageWindow>,

    #[serde(rename = "sevenDayOpus")]
    pub seven_day_opus: Option<UsageWindow>,

    #[serde(rename = "extraUsage")]
    pub extra_usage: Option<ExtraUsage>,
}

/// A usage window from the OAuth API
#[derive(Debug, Deserialize)]
pub struct UsageWindow {
    pub utilization: Option<f64>,

    #[serde(rename = "resetsAt")]
    pub resets_at: Option<String>,
}

/// Extra usage (credits) info
#[derive(Debug, Deserialize)]
pub struct ExtraUsage {
    #[serde(rename = "isEnabled")]
    pub is_enabled: Option<bool>,

    #[serde(rename = "usedCredits")]
    pub used_credits: Option<f64>,

    #[serde(rename = "monthlyLimit")]
    pub monthly_limit: Option<f64>,

    pub currency: Option<String>,
}

/// Claude OAuth fetcher
pub struct ClaudeOAuthFetcher {
    client: Client,
}

impl ClaudeOAuthFetcher {
    const USAGE_URL: &'static str = "https://api.claude.ai/api/usage";

    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// Fetch usage data using OAuth credentials
    pub async fn fetch_usage(
        &self,
        credentials: &OAuthCredentials,
    ) -> Result<OAuthUsageResponse, ProviderError> {
        if credentials.is_expired() {
            return Err(ProviderError::OAuth(
                "OAuth token expired. Run `claude` to refresh.".to_string(),
            ));
        }

        // Check for required scope
        if !credentials.has_scope("user:profile") {
            return Err(ProviderError::OAuth(format!(
                "OAuth token missing 'user:profile' scope (has: {}). Run `claude setup-token` to regenerate.",
                credentials.scopes.join(", ")
            )));
        }

        let response = self
            .client
            .get(Self::USAGE_URL)
            .header("Authorization", format!("Bearer {}", credentials.access_token))
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status.as_u16() == 403 && body.contains("user:profile") {
                return Err(ProviderError::OAuth(
                    "OAuth token does not meet scope requirement 'user:profile'. Run `claude setup-token` to regenerate.".to_string(),
                ));
            }

            return Err(ProviderError::OAuth(format!(
                "API error {}: {}",
                status,
                body.chars().take(200).collect::<String>()
            )));
        }

        let usage: OAuthUsageResponse = response.json().await.map_err(|e| {
            ProviderError::Parse(format!("Failed to parse OAuth response: {}", e))
        })?;

        Ok(usage)
    }

    /// Convert OAuth usage window to RateWindow
    pub fn to_rate_window(window: &UsageWindow, window_minutes: Option<u32>) -> Option<RateWindow> {
        let utilization = window.utilization?;

        let resets_at = window
            .resets_at
            .as_ref()
            .and_then(|s| parse_iso8601_date(s));

        let reset_description = resets_at.map(format_reset_date);

        Some(RateWindow::with_details(
            utilization,
            window_minutes,
            resets_at,
            reset_description,
        ))
    }
}

impl Default for ClaudeOAuthFetcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse an ISO8601 date string
fn parse_iso8601_date(s: &str) -> Option<DateTime<Utc>> {
    // Try parsing with various formats
    DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.with_timezone(&Utc))
        .or_else(|| {
            // Try without timezone
            chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M:%S%.f")
                .ok()
                .map(|ndt| ndt.and_utc())
        })
}

/// Format a reset date for display
fn format_reset_date(date: DateTime<Utc>) -> String {
    date.format("%b %-d at %-I:%M%p").to_string()
}

/// Credential store location for Claude OAuth
pub struct ClaudeCredentialStore;

impl ClaudeCredentialStore {
    const SERVICE: &'static str = "codexbar-claude";
    const KEY: &'static str = "oauth";

    /// Load OAuth credentials from storage
    #[cfg(windows)]
    pub fn load() -> Result<OAuthCredentials, ProviderError> {
        use crate::core::{CredentialStore, WindowsCredentialStore};

        let store = WindowsCredentialStore::new();
        let json = store
            .get(Self::SERVICE, Self::KEY)
            .map_err(|e| ProviderError::OAuth(format!("Failed to load credentials: {}", e)))?;

        serde_json::from_str(&json)
            .map_err(|e| ProviderError::OAuth(format!("Invalid credential format: {}", e)))
    }

    /// Save OAuth credentials to storage
    #[cfg(windows)]
    pub fn save(credentials: &OAuthCredentials) -> Result<(), ProviderError> {
        use crate::core::{CredentialStore, WindowsCredentialStore};

        let store = WindowsCredentialStore::new();
        let json = serde_json::to_string(credentials)
            .map_err(|e| ProviderError::OAuth(format!("Failed to serialize credentials: {}", e)))?;

        store
            .set(Self::SERVICE, Self::KEY, &json)
            .map_err(|e| ProviderError::OAuth(format!("Failed to save credentials: {}", e)))
    }

    /// Delete stored credentials
    #[cfg(windows)]
    pub fn delete() -> Result<(), ProviderError> {
        use crate::core::{CredentialStore, WindowsCredentialStore};

        let store = WindowsCredentialStore::new();
        store
            .delete(Self::SERVICE, Self::KEY)
            .map_err(|e| ProviderError::OAuth(format!("Failed to delete credentials: {}", e)))
    }
}
