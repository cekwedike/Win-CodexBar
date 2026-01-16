//! Antigravity provider implementation
//!
//! Fetches usage data from Antigravity's local language server probe
//! No external authentication required

use async_trait::async_trait;

use crate::core::{
    FetchContext, Provider, ProviderId, ProviderError, ProviderFetchResult,
    ProviderMetadata, RateWindow, SourceMode, UsageSnapshot,
};

/// Antigravity provider
pub struct AntigravityProvider {
    metadata: ProviderMetadata,
}

impl AntigravityProvider {
    pub fn new() -> Self {
        Self {
            metadata: ProviderMetadata {
                id: ProviderId::Antigravity,
                display_name: "Antigravity",
                session_label: "Session",
                weekly_label: "Weekly",
                supports_opus: false,
                supports_credits: false,
                default_enabled: false,
                is_primary: false,
                dashboard_url: None,
                status_page_url: None,
            },
        }
    }

    /// Try to probe the local language server for usage info
    async fn probe_local_server(&self) -> Result<UsageSnapshot, ProviderError> {
        // Antigravity uses a local language server probe
        // Check common socket paths or ports

        // Try to connect to local LSP
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .map_err(|e| ProviderError::Other(e.to_string()))?;

        // Common Antigravity LSP endpoints
        let endpoints = [
            "http://127.0.0.1:7865/usage",
            "http://127.0.0.1:7866/api/usage",
        ];

        for endpoint in endpoints {
            match client.get(endpoint).send().await {
                Ok(resp) if resp.status().is_success() => {
                    if let Ok(json) = resp.json::<serde_json::Value>().await {
                        return self.parse_usage_response(&json);
                    }
                }
                _ => continue,
            }
        }

        Err(ProviderError::NotInstalled(
            "Antigravity language server not running".to_string()
        ))
    }

    fn parse_usage_response(&self, json: &serde_json::Value) -> Result<UsageSnapshot, ProviderError> {
        let used_percent = json.get("used_percent")
            .or_else(|| json.get("usage_percent"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let primary = RateWindow::new(used_percent);
        Ok(UsageSnapshot::new(primary).with_login_method("Local LSP"))
    }
}

impl Default for AntigravityProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for AntigravityProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Antigravity
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn fetch_usage(&self, _ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError> {
        tracing::debug!("Fetching Antigravity usage via local probe");

        match self.probe_local_server().await {
            Ok(usage) => Ok(ProviderFetchResult::new(usage, "local")),
            Err(e) => {
                tracing::warn!("Antigravity probe failed: {}", e);
                Err(e)
            }
        }
    }

    fn available_sources(&self) -> Vec<SourceMode> {
        vec![SourceMode::Auto, SourceMode::Cli]
    }

    fn supports_cli(&self) -> bool {
        true
    }
}
