//! Tauri application module
//!
//! Provides the Tauri-based GUI for CodexBar with system tray integration

mod commands;

use std::sync::Mutex;
use tauri::{
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle, Emitter, Manager, Runtime, WindowEvent,
};

use crate::core::{FetchContext, Provider, ProviderId};
use crate::notifications::NotificationManager;
use crate::providers::{
    AntigravityProvider, AugmentProvider, ClaudeProvider, CodexProvider,
    CopilotProvider, CursorProvider, FactoryProvider, GeminiProvider,
    KiroProvider, MiniMaxProvider, VertexAIProvider, ZaiProvider,
};
use crate::settings::Settings;
use crate::status::{fetch_provider_status, StatusLevel};
use crate::tray::icon::{BadgeType, UsageLevel};

/// Shared application state
pub struct AppState {
    pub providers: Mutex<Vec<ProviderInfo>>,
    pub settings: Mutex<Settings>,
    pub refresh_interval: Mutex<u64>,
    pub notifications: Mutex<NotificationManager>,
    pub status_badge: Mutex<BadgeType>,
}

impl Default for AppState {
    fn default() -> Self {
        let settings = Settings::load();
        let refresh_interval = settings.refresh_interval_secs;

        // Initialize providers based on enabled settings
        let provider_infos: Vec<ProviderInfo> = settings
            .get_enabled_provider_ids()
            .iter()
            .map(|id| ProviderInfo::new(id.display_name(), *id))
            .collect();

        Self {
            providers: Mutex::new(provider_infos),
            settings: Mutex::new(settings),
            refresh_interval: Mutex::new(refresh_interval),
            notifications: Mutex::new(NotificationManager::new()),
            status_badge: Mutex::new(BadgeType::None),
        }
    }
}

/// Provider information for the frontend
#[derive(Debug, Clone, serde::Serialize)]
pub struct ProviderInfo {
    pub name: String,
    pub id: String,
    pub percent: Option<f64>,
    pub used: Option<u64>,
    pub limit: Option<u64>,
    pub unit: Option<String>,
    pub reset_time: Option<String>,
    pub error: Option<String>,
}

impl ProviderInfo {
    pub fn new(name: &str, id: ProviderId) -> Self {
        Self {
            name: name.to_string(),
            id: id.cli_name().to_string(),
            percent: None,
            used: None,
            limit: None,
            unit: None,
            reset_time: None,
            error: None,
        }
    }
}

/// Create a Tauri image from usage level with optional badge
fn create_tauri_icon(level: UsageLevel, badge: BadgeType) -> Option<tauri::image::Image<'static>> {
    let (r, g, b) = level.color();
    let size = 32u32;
    let mut rgba = Vec::with_capacity((size * size * 4) as usize);

    let letter_color = (255u8, 255u8, 255u8, 255u8);
    let bg_color = (r, g, b, 255u8);
    let border_color = (r / 2, g / 2, b / 2, 255u8);

    // Badge colors
    let badge_color = match badge {
        BadgeType::Warning => (255u8, 193u8, 7u8, 255u8),   // Yellow
        BadgeType::Incident => (244u8, 67u8, 54u8, 255u8),  // Red
        BadgeType::None => (0u8, 0u8, 0u8, 0u8),
    };
    let badge_border_color = match badge {
        BadgeType::Warning => (200u8, 150u8, 5u8, 255u8),
        BadgeType::Incident => (200u8, 50u8, 40u8, 255u8),
        BadgeType::None => (0u8, 0u8, 0u8, 0u8),
    };

    // Badge position (bottom-right corner)
    let badge_cx = 25u32;
    let badge_cy = 25u32;
    let badge_radius = 5u32;

    let cx = size / 2;
    let cy = size / 2;
    let outer_radius = 10u32;
    let inner_radius = 5u32;

    for y in 0..size {
        for x in 0..size {
            let margin = 2u32;
            let border = 1u32;

            let in_bounds = x >= margin && x < size - margin && y >= margin && y < size - margin;
            let in_border = in_bounds && (
                x < margin + border
                || x >= size - margin - border
                || y < margin + border
                || y >= size - margin - border
            );

            let dx = (x as i32 - cx as i32).abs() as u32;
            let dy = (y as i32 - cy as i32).abs() as u32;
            let dist_sq = dx * dx + dy * dy;
            let in_ring = dist_sq >= inner_radius * inner_radius && dist_sq <= outer_radius * outer_radius;
            let is_gap = x > cx && dy < dx / 2 + 2;
            let is_c = in_ring && !is_gap;

            // Check if in badge area (circle in bottom-right)
            let badge_dx = (x as i32 - badge_cx as i32).abs() as u32;
            let badge_dy = (y as i32 - badge_cy as i32).abs() as u32;
            let badge_dist_sq = badge_dx * badge_dx + badge_dy * badge_dy;
            let in_badge = badge != BadgeType::None && badge_dist_sq <= badge_radius * badge_radius;
            let in_badge_border = badge != BadgeType::None && badge_dist_sq <= (badge_radius + 1) * (badge_radius + 1) && !in_badge;

            let pixel = if in_badge_border {
                badge_border_color
            } else if in_badge {
                badge_color
            } else if !in_bounds {
                (0u8, 0u8, 0u8, 0u8)
            } else if in_border {
                border_color
            } else if is_c {
                letter_color
            } else {
                bg_color
            };

            rgba.extend_from_slice(&[pixel.0, pixel.1, pixel.2, pixel.3]);
        }
    }

    Some(tauri::image::Image::new_owned(rgba, size, size))
}

/// Create a provider instance by ID
fn create_provider(id: ProviderId) -> Box<dyn Provider> {
    match id {
        ProviderId::Claude => Box::new(ClaudeProvider::new()),
        ProviderId::Codex => Box::new(CodexProvider::new()),
        ProviderId::Cursor => Box::new(CursorProvider::new()),
        ProviderId::Gemini => Box::new(GeminiProvider::new()),
        ProviderId::Copilot => Box::new(CopilotProvider::new()),
        ProviderId::Antigravity => Box::new(AntigravityProvider::new()),
        ProviderId::Factory => Box::new(FactoryProvider::new()),
        ProviderId::Zai => Box::new(ZaiProvider::new()),
        ProviderId::Kiro => Box::new(KiroProvider::new()),
        ProviderId::VertexAI => Box::new(VertexAIProvider::new()),
        ProviderId::Augment => Box::new(AugmentProvider::new()),
        ProviderId::MiniMax => Box::new(MiniMaxProvider::new()),
    }
}

/// Run the Tauri application
pub fn run() -> anyhow::Result<()> {
    tracing::info!("Starting CodexBar Tauri application");

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(AppState::default())
        .setup(|app| {
            // Create tray icon
            let tray_icon_image = create_tauri_icon(UsageLevel::Unknown, BadgeType::None)
                .ok_or_else(|| anyhow::anyhow!("Failed to create tray icon"))?;

            let tray = TrayIconBuilder::new()
                .icon(tray_icon_image)
                .tooltip("CodexBar")
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        toggle_window(app);
                    }
                })
                .build(app)?;

            // Store tray icon handle
            app.manage(tray);

            // Set up window close behavior (hide instead of close)
            let window = app.get_webview_window("main").unwrap();
            let window_clone = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    let _ = window_clone.hide();
                }
            });

            // Initial provider refresh
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                refresh_providers_internal(&handle);
            });

            // Start background refresh thread with dynamic interval
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                loop {
                    // Get current refresh interval from state
                    let interval = if let Some(state) = handle.try_state::<AppState>() {
                        state.refresh_interval.lock().map(|i| *i).unwrap_or(300)
                    } else {
                        300
                    };

                    std::thread::sleep(std::time::Duration::from_secs(interval));
                    refresh_providers_internal(&handle);
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_providers,
            commands::get_settings,
            commands::toggle_provider,
            commands::set_refresh_interval,
            commands::open_settings,
            commands::open_about,
            commands::open_cookie_input,
            commands::get_app_info,
            commands::get_manual_cookies,
            commands::save_manual_cookie,
            commands::delete_manual_cookie,
            commands::quit_app,
        ])
        .run(tauri::generate_context!())?;

    Ok(())
}

/// Toggle the main window visibility
fn toggle_window<R: Runtime>(app: &AppHandle<R>) {
    if let Some(window) = app.get_webview_window("main") {
        if window.is_visible().unwrap_or(false) {
            let _ = window.hide();
        } else {
            position_window_near_tray(&window);
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}

/// Position window near the system tray
fn position_window_near_tray<R: Runtime>(window: &tauri::WebviewWindow<R>) {
    if let Ok(pos) = window.cursor_position() {
        let x = (pos.x as i32 - 160).max(0);
        let y = (pos.y as i32 - 420).max(0);
        let _ = window.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
    }
}

/// Refresh provider data (internal helper)
fn refresh_providers_internal<R: Runtime>(app: &AppHandle<R>) {
    // Get enabled providers from settings
    let enabled_providers: Vec<ProviderId> = if let Some(state) = app.try_state::<AppState>() {
        if let Ok(settings) = state.settings.lock() {
            settings.get_enabled_provider_ids()
        } else {
            vec![ProviderId::Claude, ProviderId::Codex]
        }
    } else {
        vec![ProviderId::Claude, ProviderId::Codex]
    };

    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let ctx = FetchContext::default();

        let mut futures = Vec::new();
        for id in &enabled_providers {
            let provider = create_provider(*id);
            let id_copy = *id;
            let ctx_copy = ctx.clone();
            futures.push(async move {
                let result = provider.fetch_usage(&ctx_copy).await;
                (id_copy, result)
            });
        }

        let results = futures::future::join_all(futures).await;

        let mut provider_infos = Vec::new();
        let mut max_usage = 0.0f64;
        let mut usage_data: Vec<(ProviderId, f64)> = Vec::new();

        for (id, result) in results {
            let mut info = ProviderInfo::new(id.display_name(), id);

            match result {
                Ok(fetch_result) => {
                    let usage = fetch_result.usage;
                    info.percent = Some(usage.primary.used_percent);

                    if let Some(desc) = &usage.primary.reset_description {
                        info.reset_time = Some(desc.clone());
                    } else if let Some(reset) = usage.primary.resets_at {
                        info.reset_time = Some(format_reset_time(reset));
                    }

                    if usage.primary.used_percent > max_usage {
                        max_usage = usage.primary.used_percent;
                    }

                    // Collect usage data for notifications
                    usage_data.push((id, usage.primary.used_percent));
                }
                Err(e) => {
                    info.error = Some(e.to_string());
                }
            }

            provider_infos.push(info);
        }

        // Update state and check notifications
        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(mut providers) = state.providers.lock() {
                *providers = provider_infos.clone();
            }

            // Check usage thresholds and send notifications
            if let (Ok(mut notifications), Ok(settings)) = (
                state.notifications.lock(),
                state.settings.lock(),
            ) {
                for (provider_id, used_percent) in usage_data {
                    notifications.check_and_notify(provider_id, used_percent, &settings);
                }
            }
        }

        // Fetch status for enabled providers and determine badge
        let mut status_badge = BadgeType::None;
        for id in &enabled_providers {
            if let Some(status) = fetch_provider_status(id.cli_name()).await {
                match status.level {
                    StatusLevel::Major => {
                        status_badge = BadgeType::Incident;
                        break; // Major is worst, no need to check more
                    }
                    StatusLevel::Partial | StatusLevel::Degraded => {
                        if status_badge != BadgeType::Incident {
                            status_badge = BadgeType::Warning;
                        }
                    }
                    _ => {}
                }
            }
        }

        // Store badge in state
        if let Some(state) = app.try_state::<AppState>() {
            if let Ok(mut badge) = state.status_badge.lock() {
                *badge = status_badge;
            }
        }

        // Update tray icon based on max usage and status badge
        update_tray_icon(app, max_usage, status_badge);

        // Emit event to frontend
        let _ = app.emit("providers-updated", provider_infos);
    });
}

/// Update the tray icon based on usage level and badge
fn update_tray_icon<R: Runtime>(app: &AppHandle<R>, max_usage: f64, badge: BadgeType) {
    let level = if max_usage > 0.0 {
        UsageLevel::from_percent(max_usage)
    } else {
        UsageLevel::Unknown
    };

    if let Some(image) = create_tauri_icon(level, badge) {
        if let Some(tray) = app.try_state::<tauri::tray::TrayIcon<R>>() {
            let _ = tray.set_icon(Some(image));
            let mut tooltip = if max_usage > 0.0 {
                format!("CodexBar - {:.0}% used", max_usage)
            } else {
                "CodexBar".to_string()
            };
            if badge == BadgeType::Incident {
                tooltip.push_str(" (Status Issue)");
            }
            let _ = tray.set_tooltip(Some(&tooltip));
        }
    }
}

/// Format reset time for display
fn format_reset_time(reset: chrono::DateTime<chrono::Utc>) -> String {
    let now = chrono::Utc::now();
    let duration = reset.signed_duration_since(now);

    if duration.num_hours() > 24 {
        format!("in {} days", duration.num_days())
    } else if duration.num_hours() > 0 {
        format!("in {}h", duration.num_hours())
    } else if duration.num_minutes() > 0 {
        format!("in {}m", duration.num_minutes())
    } else {
        "soon".to_string()
    }
}
