//! Factory (Windsurf) provider implementation
//!
//! Fetches usage data from Windsurf/Codeium's local state or API
//! Windsurf stores usage info in local configuration files

use async_trait::async_trait;
use std::path::PathBuf;

use crate::core::{
    FetchContext, Provider, ProviderId, ProviderError, ProviderFetchResult,
    ProviderMetadata, RateWindow, SourceMode, UsageSnapshot,
};

/// Factory (Windsurf) provider
pub struct FactoryProvider {
    metadata: ProviderMetadata,
}

impl FactoryProvider {
    pub fn new() -> Self {
        Self {
            metadata: ProviderMetadata {
                id: ProviderId::Factory,
                display_name: "Windsurf",
                session_label: "Session",
                weekly_label: "Weekly",
                supports_opus: false,
                supports_credits: true,
                default_enabled: false,
                is_primary: false,
                dashboard_url: Some("https://codeium.com/account"),
                status_page_url: Some("https://status.codeium.com"),
            },
        }
    }

    /// Try to read usage from Windsurf local config
    fn get_windsurf_config_path() -> Option<PathBuf> {
        // Windsurf stores config in AppData on Windows
        #[cfg(target_os = "windows")]
        {
            dirs::config_dir().map(|p| p.join("Windsurf").join("User").join("globalStorage").join("codeium.codeium"))
        }
        #[cfg(not(target_os = "windows"))]
        {
            dirs::config_dir().map(|p| p.join("Windsurf").join("User").join("globalStorage").join("codeium.codeium"))
        }
    }

    /// Try to find the windsurf CLI
    fn which_windsurf() -> Option<PathBuf> {
        // Check common locations
        let possible_paths = [
            which::which("windsurf").ok(),
            // Windows-specific paths
            #[cfg(target_os = "windows")]
            dirs::data_local_dir().map(|p| p.join("Programs").join("Windsurf").join("windsurf.exe")),
            #[cfg(not(target_os = "windows"))]
            None,
        ];

        possible_paths.into_iter().flatten().find(|p| p.exists())
    }

    /// Probe the Windsurf CLI for usage info
    async fn probe_cli(&self) -> Result<UsageSnapshot, ProviderError> {
        let windsurf_path = Self::which_windsurf().ok_or_else(|| {
            ProviderError::NotInstalled("Windsurf not found. Install from https://codeium.com/windsurf".to_string())
        })?;

        // Windsurf CLI doesn't have a direct usage command yet
        // Check if the binary exists to confirm installation
        if windsurf_path.exists() {
            // Return placeholder - Windsurf doesn't expose usage via CLI yet
            let usage = UsageSnapshot::new(RateWindow::new(0.0))
                .with_login_method("Windsurf (installed)");
            Ok(usage)
        } else {
            Err(ProviderError::NotInstalled("Windsurf CLI not found".to_string()))
        }
    }

    /// Try to read usage from local config files
    async fn read_local_config(&self) -> Result<UsageSnapshot, ProviderError> {
        let config_path = Self::get_windsurf_config_path().ok_or_else(|| {
            ProviderError::NotInstalled("Windsurf config directory not found".to_string())
        })?;

        // Check for usage/state file
        let state_file = config_path.join("state.json");
        if state_file.exists() {
            if let Ok(content) = tokio::fs::read_to_string(&state_file).await {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    return self.parse_state_json(&json);
                }
            }
        }

        Err(ProviderError::NotInstalled("Windsurf state file not found".to_string()))
    }

    fn parse_state_json(&self, json: &serde_json::Value) -> Result<UsageSnapshot, ProviderError> {
        // Parse Windsurf state format
        let used_percent = json.get("usage")
            .and_then(|u| u.get("percent"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let email = json.get("user")
            .and_then(|u| u.get("email"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut usage = UsageSnapshot::new(RateWindow::new(used_percent))
            .with_login_method("Windsurf");

        if let Some(email) = email {
            usage = usage.with_email(email);
        }

        Ok(usage)
    }

    /// Fetch usage via Codeium web API
    async fn fetch_via_web(&self) -> Result<UsageSnapshot, ProviderError> {
        // Codeium API requires authentication token from local config
        let config_path = Self::get_windsurf_config_path()
            .ok_or_else(|| ProviderError::NoCookies)?;

        let api_key_file = config_path.join("api_key");
        if !api_key_file.exists() {
            return Err(ProviderError::AuthRequired);
        }

        let api_key = tokio::fs::read_to_string(&api_key_file).await
            .map_err(|e| ProviderError::Other(e.to_string()))?
            .trim()
            .to_string();

        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::Other(e.to_string()))?;

        let resp = client
            .get("https://api.codeium.com/user/usage")
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(ProviderError::AuthRequired);
        }

        let json: serde_json::Value = resp.json().await
            .map_err(|e| ProviderError::Parse(e.to_string()))?;

        self.parse_api_response(&json)
    }

    fn parse_api_response(&self, json: &serde_json::Value) -> Result<UsageSnapshot, ProviderError> {
        let used = json.get("used_credits")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let limit = json.get("credit_limit")
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);

        let used_percent = if limit > 0.0 { (used / limit) * 100.0 } else { 0.0 };

        let email = json.get("email")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let plan = json.get("plan")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let mut usage = UsageSnapshot::new(RateWindow::new(used_percent));

        if let Some(email) = email {
            usage = usage.with_email(email);
        }
        if let Some(plan) = plan {
            usage = usage.with_login_method(&plan);
        }

        Ok(usage)
    }
}

impl Default for FactoryProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for FactoryProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Factory
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn fetch_usage(&self, ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError> {
        tracing::debug!("Fetching Windsurf usage");

        match ctx.source_mode {
            SourceMode::Auto => {
                // Try local config first, then web API, then CLI
                if let Ok(usage) = self.read_local_config().await {
                    return Ok(ProviderFetchResult::new(usage, "local"));
                }
                if let Ok(usage) = self.fetch_via_web().await {
                    return Ok(ProviderFetchResult::new(usage, "web"));
                }
                let usage = self.probe_cli().await?;
                Ok(ProviderFetchResult::new(usage, "cli"))
            }
            SourceMode::Web => {
                let usage = self.fetch_via_web().await?;
                Ok(ProviderFetchResult::new(usage, "web"))
            }
            SourceMode::Cli => {
                let usage = self.probe_cli().await?;
                Ok(ProviderFetchResult::new(usage, "cli"))
            }
            SourceMode::OAuth => {
                Err(ProviderError::UnsupportedSource(SourceMode::OAuth))
            }
        }
    }

    fn available_sources(&self) -> Vec<SourceMode> {
        vec![SourceMode::Auto, SourceMode::Web, SourceMode::Cli]
    }

    fn supports_web(&self) -> bool {
        true
    }

    fn supports_cli(&self) -> bool {
        true
    }
}
