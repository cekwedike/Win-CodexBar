//! Tauri commands exposed to the frontend

use tauri::{AppHandle, Manager, Runtime};

use super::{AppState, ProviderInfo};
use crate::settings::{
    get_refresh_interval_options, ManualCookies, ProviderStatus, RefreshIntervalOption,
    SavedCookieInfo,
};

/// Get all provider data
#[tauri::command]
pub async fn get_providers<R: Runtime>(app: AppHandle<R>) -> Result<Vec<ProviderInfo>, String> {
    // Trigger a refresh
    let app_clone = app.clone();
    tokio::task::spawn_blocking(move || {
        super::refresh_providers_internal(&app_clone);
    })
    .await
    .map_err(|e| e.to_string())?;

    // Return current state
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(providers) = state.providers.lock() {
            return Ok(providers.clone());
        }
    }

    Ok(Vec::new())
}

/// Get current settings
#[tauri::command]
pub async fn get_settings<R: Runtime>(app: AppHandle<R>) -> Result<SettingsResponse, String> {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(settings) = state.settings.lock() {
            return Ok(SettingsResponse {
                providers: settings.get_all_providers_status(),
                refresh_interval_secs: settings.refresh_interval_secs,
                refresh_options: get_refresh_interval_options(),
                show_notifications: settings.show_notifications,
                high_usage_threshold: settings.high_usage_threshold,
                critical_usage_threshold: settings.critical_usage_threshold,
            });
        }
    }

    Err("Could not read settings".to_string())
}

/// Update provider enabled status
#[tauri::command]
pub async fn toggle_provider<R: Runtime>(
    app: AppHandle<R>,
    provider_id: String,
) -> Result<bool, String> {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut settings) = state.settings.lock() {
            if let Some(id) = crate::core::ProviderId::from_cli_name(&provider_id) {
                let enabled = settings.toggle_provider(id);
                let _ = settings.save();
                return Ok(enabled);
            }
        }
    }

    Err("Could not toggle provider".to_string())
}

/// Update refresh interval
#[tauri::command]
pub async fn set_refresh_interval<R: Runtime>(
    app: AppHandle<R>,
    interval_secs: u64,
) -> Result<(), String> {
    if let Some(state) = app.try_state::<AppState>() {
        if let Ok(mut settings) = state.settings.lock() {
            settings.refresh_interval_secs = interval_secs;
            let _ = settings.save();

            if let Ok(mut interval) = state.refresh_interval.lock() {
                *interval = interval_secs;
            }

            return Ok(());
        }
    }

    Err("Could not update refresh interval".to_string())
}

/// Open settings panel
#[tauri::command]
pub async fn open_settings<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    tracing::info!("Settings requested");

    if let Some(window) = app.get_webview_window("settings") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _settings_window = tauri::WebviewWindowBuilder::new(
            &app,
            "settings",
            tauri::WebviewUrl::App("settings.html".into()),
        )
        .title("CodexBar Settings")
        .inner_size(400.0, 500.0)
        .resizable(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Open About dialog
#[tauri::command]
pub async fn open_about<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    tracing::info!("About requested");

    if let Some(window) = app.get_webview_window("about") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _about_window = tauri::WebviewWindowBuilder::new(
            &app,
            "about",
            tauri::WebviewUrl::App("about.html".into()),
        )
        .title("About CodexBar")
        .inner_size(360.0, 480.0)
        .resizable(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Open manual cookie input dialog
#[tauri::command]
pub async fn open_cookie_input<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    tracing::info!("Cookie input requested");

    if let Some(window) = app.get_webview_window("cookies") {
        let _ = window.show();
        let _ = window.set_focus();
    } else {
        let _cookie_window = tauri::WebviewWindowBuilder::new(
            &app,
            "cookies",
            tauri::WebviewUrl::App("cookies.html".into()),
        )
        .title("Manual Cookie Input")
        .inner_size(450.0, 650.0)
        .resizable(false)
        .center()
        .build()
        .map_err(|e| e.to_string())?;
    }

    Ok(())
}

/// Get app info for About dialog
#[tauri::command]
pub async fn get_app_info() -> Result<AppInfo, String> {
    Ok(AppInfo {
        version: env!("CARGO_PKG_VERSION").to_string(),
        name: "CodexBar".to_string(),
        description: "Monitor AI provider usage limits".to_string(),
    })
}

/// Get saved manual cookies
#[tauri::command]
pub async fn get_manual_cookies() -> Result<Vec<SavedCookieInfo>, String> {
    let cookies = ManualCookies::load();
    Ok(cookies.get_all_for_display())
}

/// Save a manual cookie
#[tauri::command]
pub async fn save_manual_cookie(provider_id: String, cookie_header: String) -> Result<(), String> {
    let mut cookies = ManualCookies::load();
    cookies.set(&provider_id, &cookie_header);
    cookies.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Delete a manual cookie
#[tauri::command]
pub async fn delete_manual_cookie(provider_id: String) -> Result<(), String> {
    let mut cookies = ManualCookies::load();
    cookies.remove(&provider_id);
    cookies.save().map_err(|e| e.to_string())?;
    Ok(())
}

/// Quit the application
#[tauri::command]
pub async fn quit_app<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    tracing::info!("Quit requested");
    app.exit(0);
    Ok(())
}

/// Settings response for the frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct SettingsResponse {
    pub providers: Vec<ProviderStatus>,
    pub refresh_interval_secs: u64,
    pub refresh_options: Vec<RefreshIntervalOption>,
    pub show_notifications: bool,
    pub high_usage_threshold: f64,
    pub critical_usage_threshold: f64,
}

/// App info for About dialog
#[derive(Debug, Clone, serde::Serialize)]
pub struct AppInfo {
    pub version: String,
    pub name: String,
    pub description: String,
}
