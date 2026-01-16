//! Kiro provider implementation
//!
//! Fetches usage data from AWS Kiro (Amazon's AI coding assistant)
//! Kiro uses AWS credentials for authentication

use async_trait::async_trait;
use std::path::PathBuf;

use crate::core::{
    FetchContext, Provider, ProviderId, ProviderError, ProviderFetchResult,
    ProviderMetadata, RateWindow, SourceMode, UsageSnapshot,
};

/// Kiro provider (AWS AI assistant)
pub struct KiroProvider {
    metadata: ProviderMetadata,
}

impl KiroProvider {
    pub fn new() -> Self {
        Self {
            metadata: ProviderMetadata {
                id: ProviderId::Kiro,
                display_name: "Kiro",
                session_label: "Session",
                weekly_label: "Monthly",
                supports_opus: false,
                supports_credits: true,
                default_enabled: false,
                is_primary: false,
                dashboard_url: Some("https://kiro.dev/account"),
                status_page_url: Some("https://health.aws.amazon.com"),
            },
        }
    }

    /// Get Kiro config directory
    fn get_kiro_config_path() -> Option<PathBuf> {
        #[cfg(target_os = "windows")]
        {
            dirs::config_dir().map(|p| p.join("Kiro"))
        }
        #[cfg(not(target_os = "windows"))]
        {
            dirs::home_dir().map(|p| p.join(".kiro"))
        }
    }

    /// Find Kiro CLI binary
    fn which_kiro() -> Option<PathBuf> {
        let possible_paths = [
            which::which("kiro").ok(),
            #[cfg(target_os = "windows")]
            dirs::data_local_dir().map(|p| p.join("Programs").join("Kiro").join("kiro.exe")),
            #[cfg(target_os = "windows")]
            Some(PathBuf::from("C:\\Program Files\\Kiro\\kiro.exe")),
            #[cfg(not(target_os = "windows"))]
            None,
        ];

        possible_paths.into_iter().flatten().find(|p| p.exists())
    }

    /// Read AWS/Kiro credentials
    async fn read_credentials(&self) -> Result<(String, String), ProviderError> {
        // Check Kiro-specific config first
        let kiro_config = Self::get_kiro_config_path()
            .ok_or_else(|| ProviderError::NotInstalled("Kiro config not found".to_string()))?;

        let creds_file = kiro_config.join("credentials");
        if creds_file.exists() {
            let content = tokio::fs::read_to_string(&creds_file).await
                .map_err(|e| ProviderError::Other(e.to_string()))?;

            // Parse INI-style credentials file
            let mut access_key = None;
            let mut secret_key = None;

            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("aws_access_key_id") {
                    access_key = line.split('=').nth(1).map(|s| s.trim().to_string());
                } else if line.starts_with("aws_secret_access_key") {
                    secret_key = line.split('=').nth(1).map(|s| s.trim().to_string());
                }
            }

            if let (Some(ak), Some(sk)) = (access_key, secret_key) {
                return Ok((ak, sk));
            }
        }

        // Fall back to AWS credentials
        let aws_creds = dirs::home_dir()
            .map(|p| p.join(".aws").join("credentials"));

        if let Some(aws_file) = aws_creds {
            if aws_file.exists() {
                let content = tokio::fs::read_to_string(&aws_file).await
                    .map_err(|e| ProviderError::Other(e.to_string()))?;

                let mut access_key = None;
                let mut secret_key = None;

                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("aws_access_key_id") {
                        access_key = line.split('=').nth(1).map(|s| s.trim().to_string());
                    } else if line.starts_with("aws_secret_access_key") {
                        secret_key = line.split('=').nth(1).map(|s| s.trim().to_string());
                    }
                }

                if let (Some(ak), Some(sk)) = (access_key, secret_key) {
                    return Ok((ak, sk));
                }
            }
        }

        Err(ProviderError::AuthRequired)
    }

    /// Fetch usage via Kiro API
    async fn fetch_via_web(&self) -> Result<UsageSnapshot, ProviderError> {
        let (_access_key, _secret_key) = self.read_credentials().await?;

        // Kiro API requires AWS SigV4 signing
        // For now, we'll just check if credentials exist
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| ProviderError::Other(e.to_string()))?;

        // Kiro usage endpoint (placeholder - actual endpoint TBD)
        let resp = client
            .get("https://api.kiro.dev/v1/usage")
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => {
                let json: serde_json::Value = r.json().await
                    .map_err(|e| ProviderError::Parse(e.to_string()))?;
                self.parse_usage_response(&json)
            }
            _ => {
                // Return placeholder if API is not available
                let usage = UsageSnapshot::new(RateWindow::new(0.0))
                    .with_login_method("Kiro (credentials found)");
                Ok(usage)
            }
        }
    }

    fn parse_usage_response(&self, json: &serde_json::Value) -> Result<UsageSnapshot, ProviderError> {
        let used = json.get("used")
            .or_else(|| json.get("usage"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let limit = json.get("limit")
            .or_else(|| json.get("quota"))
            .and_then(|v| v.as_f64())
            .unwrap_or(100.0);

        let used_percent = if limit > 0.0 {
            (used / limit) * 100.0
        } else {
            0.0
        };

        let plan = json.get("plan")
            .or_else(|| json.get("tier"))
            .and_then(|v| v.as_str())
            .unwrap_or("Kiro");

        let usage = UsageSnapshot::new(RateWindow::new(used_percent))
            .with_login_method(plan);

        Ok(usage)
    }

    /// Probe CLI for detection
    async fn probe_cli(&self) -> Result<UsageSnapshot, ProviderError> {
        let kiro_path = Self::which_kiro().ok_or_else(|| {
            ProviderError::NotInstalled("Kiro not found. Install from https://kiro.dev".to_string())
        })?;

        if kiro_path.exists() {
            let usage = UsageSnapshot::new(RateWindow::new(0.0))
                .with_login_method("Kiro (installed)");
            Ok(usage)
        } else {
            Err(ProviderError::NotInstalled("Kiro not found".to_string()))
        }
    }
}

impl Default for KiroProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for KiroProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Kiro
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn fetch_usage(&self, ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError> {
        tracing::debug!("Fetching Kiro usage");

        match ctx.source_mode {
            SourceMode::Auto => {
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
