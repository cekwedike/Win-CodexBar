//! Claude provider implementation

mod oauth;
mod web_api;

use async_trait::async_trait;

use crate::core::{
    FetchContext, Provider, ProviderError, ProviderFetchResult, ProviderId, ProviderMetadata,
    RateWindow, SourceMode, UsageSnapshot,
};

pub use web_api::ClaudeWebApiFetcher;

/// Claude provider implementation
pub struct ClaudeProvider {
    metadata: ProviderMetadata,
    web_fetcher: ClaudeWebApiFetcher,
}

impl ClaudeProvider {
    pub fn new() -> Self {
        Self {
            metadata: ProviderMetadata {
                id: ProviderId::Claude,
                display_name: "Claude",
                session_label: "Session (5h)",
                weekly_label: "Weekly",
                supports_opus: true,
                supports_credits: true,
                default_enabled: true,
                is_primary: true,
                dashboard_url: Some("https://claude.ai/settings/usage"),
                status_page_url: Some("https://status.anthropic.com"),
            },
            web_fetcher: ClaudeWebApiFetcher::new(),
        }
    }
}

impl Default for ClaudeProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Provider for ClaudeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Claude
    }

    fn metadata(&self) -> &ProviderMetadata {
        &self.metadata
    }

    async fn fetch_usage(&self, ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError> {
        match ctx.source_mode {
            SourceMode::Auto => {
                // Try OAuth first, then Web, then CLI
                if let Ok(result) = self.fetch_via_oauth(ctx).await {
                    return Ok(result);
                }
                if let Ok(result) = self.fetch_via_web(ctx).await {
                    return Ok(result);
                }
                self.fetch_via_cli(ctx).await
            }
            SourceMode::OAuth => self.fetch_via_oauth(ctx).await,
            SourceMode::Web => self.fetch_via_web(ctx).await,
            SourceMode::Cli => self.fetch_via_cli(ctx).await,
        }
    }

    fn available_sources(&self) -> Vec<SourceMode> {
        vec![
            SourceMode::Auto,
            SourceMode::OAuth,
            SourceMode::Web,
            SourceMode::Cli,
        ]
    }

    fn supports_oauth(&self) -> bool {
        true
    }

    fn supports_web(&self) -> bool {
        true
    }

    fn supports_cli(&self) -> bool {
        true
    }

    fn detect_version(&self) -> Option<String> {
        detect_claude_version()
    }
}

impl ClaudeProvider {
    async fn fetch_via_oauth(&self, _ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError> {
        tracing::debug!("Attempting OAuth fetch for Claude");

        // TODO: Implement OAuth fetching with stored credentials
        // For now, return error to fall through to web API
        Err(ProviderError::OAuth(
            "OAuth credentials not available".to_string(),
        ))
    }

    async fn fetch_via_web(&self, ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError> {
        tracing::debug!("Attempting Web API fetch for Claude");

        // Check for manual cookie header first
        if let Some(ref cookie_header) = ctx.manual_cookie_header {
            tracing::debug!("Using manual cookie header");
            return self.web_fetcher.fetch_with_cookie_header(cookie_header).await;
        }

        // Otherwise, try to extract cookies from browser
        self.web_fetcher.fetch_with_cookies().await
    }

    async fn fetch_via_cli(&self, _ctx: &FetchContext) -> Result<ProviderFetchResult, ProviderError> {
        tracing::debug!("Attempting CLI probe for Claude");

        // Check if claude CLI exists
        if which_claude().is_none() {
            return Err(ProviderError::NotInstalled(
                "Claude CLI not found. Install from https://docs.claude.ai/claude-code".to_string(),
            ));
        }

        // TODO: Actually run the CLI and parse output
        // For now, return placeholder data
        let usage = UsageSnapshot::new(RateWindow::new(0.0))
            .with_secondary(RateWindow::new(0.0))
            .with_login_method("Claude (CLI)");

        Ok(ProviderFetchResult::new(usage, "cli"))
    }
}

/// Try to find the claude CLI binary
fn which_claude() -> Option<std::path::PathBuf> {
    // Check common locations on Windows
    let possible_paths = [
        // In PATH
        which::which("claude").ok(),
        // AppData locations
        dirs::data_local_dir().map(|p| p.join("Programs").join("claude").join("claude.exe")),
        // npm global install
        dirs::data_dir().map(|p| p.join("npm").join("claude.cmd")),
    ];

    possible_paths.into_iter().flatten().find(|p| p.exists())
}

/// Detect the version of the claude CLI
fn detect_claude_version() -> Option<String> {
    let claude_path = which_claude()?;

    let output = std::process::Command::new(claude_path)
        .args(["--version"])
        .output()
        .ok()?;

    if output.status.success() {
        let version_str = String::from_utf8_lossy(&output.stdout);
        extract_version(&version_str)
    } else {
        None
    }
}

/// Extract version number from a string like "claude 1.2.3"
fn extract_version(s: &str) -> Option<String> {
    let re = regex_lite::Regex::new(r"(\d+(?:\.\d+)+)").ok()?;
    re.find(s).map(|m| m.as_str().to_string())
}
