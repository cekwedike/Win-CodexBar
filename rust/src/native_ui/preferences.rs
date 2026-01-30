//! Preferences window for CodexBar
//!
//! A refined settings interface inspired by Linear and Apple Settings.
//! Design principle: Precision Calm - clear hierarchy, generous spacing, subtle depth.

use eframe::egui::{self, Color32, RichText, Rounding, Stroke, Vec2, Rect};

use super::theme::{provider_color, provider_icon, FontSize, Radius, Spacing, Theme};
use crate::settings::{ApiKeys, ManualCookies, Settings, get_api_key_providers};
use crate::core::ProviderId;

/// Which preferences tab is active
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PreferencesTab {
    #[default]
    General,
    Providers,
    ApiKeys,
    Cookies,
    Advanced,
    About,
}

impl PreferencesTab {
    fn label(&self) -> &'static str {
        match self {
            PreferencesTab::General => "General",
            PreferencesTab::Providers => "Providers",
            PreferencesTab::ApiKeys => "API Keys",
            PreferencesTab::Cookies => "Cookies",
            PreferencesTab::Advanced => "Advanced",
            PreferencesTab::About => "About",
        }
    }
}

/// Preferences window state
pub struct PreferencesWindow {
    pub is_open: bool,
    pub active_tab: PreferencesTab,
    pub settings: Settings,
    pub settings_changed: bool,
    cookies: ManualCookies,
    new_cookie_provider: String,
    new_cookie_value: String,
    cookie_status_msg: Option<(String, bool)>,
    api_keys: ApiKeys,
    new_api_key_provider: String,
    new_api_key_value: String,
    show_api_key_input: bool,
    api_key_status_msg: Option<(String, bool)>,
}

impl Default for PreferencesWindow {
    fn default() -> Self {
        Self {
            is_open: false,
            active_tab: PreferencesTab::General,
            settings: Settings::load(),
            settings_changed: false,
            cookies: ManualCookies::load(),
            new_cookie_provider: String::new(),
            new_cookie_value: String::new(),
            cookie_status_msg: None,
            api_keys: ApiKeys::load(),
            new_api_key_provider: String::new(),
            new_api_key_value: String::new(),
            show_api_key_input: false,
            api_key_status_msg: None,
        }
    }
}

impl PreferencesWindow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self) {
        self.is_open = true;
        self.settings = Settings::load();
        self.cookies = ManualCookies::load();
        self.api_keys = ApiKeys::load();
        self.settings_changed = false;
        self.cookie_status_msg = None;
        self.api_key_status_msg = None;
        self.new_api_key_value.clear();
        self.show_api_key_input = false;
    }

    pub fn close(&mut self) {
        if self.settings_changed {
            let _ = self.settings.save();
        }
        self.is_open = false;
    }

    /// Show the preferences window
    pub fn show(&mut self, ctx: &egui::Context) {
        if !self.is_open {
            return;
        }

        let screen_rect = ctx.screen_rect();

        egui::Area::new(egui::Id::new("settings_overlay"))
            .fixed_pos(screen_rect.min)
            .order(egui::Order::Foreground)
            .show(ctx, |ui| {
                // Solid background
                ui.painter().rect_filled(screen_rect, 0.0, Theme::BG_PRIMARY);

                let content_width = (screen_rect.width() - 48.0).min(440.0);
                let side_padding = (screen_rect.width() - content_width) / 2.0;

                let content_rect = Rect::from_min_max(
                    egui::pos2(side_padding, 20.0),
                    egui::pos2(screen_rect.width() - side_padding, screen_rect.height() - 20.0),
                );

                ui.allocate_ui_at_rect(content_rect, |ui| {
                    ui.vertical(|ui| {
                        // ═══════════════════════════════════════════════════════════
                        // HEADER - Minimal, clean
                        // ═══════════════════════════════════════════════════════════
                        ui.horizontal(|ui| {
                            // Back button - subtle circle
                            let back_response = ui.add(
                                egui::Button::new(
                                    RichText::new("←")
                                        .size(FontSize::LG)
                                        .color(Theme::TEXT_SECONDARY)
                                )
                                .fill(Color32::TRANSPARENT)
                                .stroke(Stroke::NONE)
                                .min_size(Vec2::new(36.0, 36.0))
                            );

                            if back_response.clicked() {
                                self.close();
                            }

                            if back_response.hovered() {
                                ui.painter().rect_filled(
                                    back_response.rect,
                                    Rounding::same(18.0),
                                    Theme::hover_overlay(),
                                );
                            }

                            ui.add_space(8.0);

                            ui.label(
                                RichText::new("Settings")
                                    .size(FontSize::XL)
                                    .color(Theme::TEXT_PRIMARY)
                                    .strong()
                            );
                        });

                        ui.add_space(Spacing::LG);

                        // ═══════════════════════════════════════════════════════════
                        // TAB BAR - Underline style, more spacious
                        // ═══════════════════════════════════════════════════════════
                        ui.horizontal(|ui| {
                            let tabs = [
                                PreferencesTab::General,
                                PreferencesTab::Providers,
                                PreferencesTab::ApiKeys,
                                PreferencesTab::Cookies,
                                PreferencesTab::Advanced,
                                PreferencesTab::About,
                            ];

                            for tab in tabs {
                                let is_selected = self.active_tab == tab;
                                let text_color = if is_selected {
                                    Theme::TEXT_PRIMARY
                                } else {
                                    Theme::TEXT_MUTED
                                };

                                let response = ui.add(
                                    egui::Button::new(
                                        RichText::new(tab.label())
                                            .size(FontSize::SM)
                                            .color(text_color)
                                    )
                                    .fill(Color32::TRANSPARENT)
                                    .stroke(Stroke::NONE)
                                    .min_size(Vec2::new(0.0, 32.0))
                                );

                                // Underline indicator for selected tab
                                if is_selected {
                                    let rect = response.rect;
                                    let indicator_rect = Rect::from_min_size(
                                        egui::pos2(rect.min.x, rect.max.y - 2.0),
                                        Vec2::new(rect.width(), 2.0),
                                    );
                                    ui.painter().rect_filled(
                                        indicator_rect,
                                        Rounding::same(1.0),
                                        Theme::ACCENT_PRIMARY,
                                    );
                                }

                                if response.clicked() {
                                    self.active_tab = tab;
                                }

                                ui.add_space(4.0);
                            }
                        });

                        // Separator line
                        ui.add_space(1.0);
                        let separator_rect = Rect::from_min_size(
                            ui.cursor().min,
                            Vec2::new(ui.available_width(), 1.0),
                        );
                        ui.painter().rect_filled(separator_rect, 0.0, Theme::SEPARATOR);
                        ui.add_space(1.0);

                        ui.add_space(Spacing::LG);

                        // ═══════════════════════════════════════════════════════════
                        // TAB CONTENT - Scrollable
                        // ═══════════════════════════════════════════════════════════
                        let scroll_height = ui.available_height() - Spacing::MD;

                        egui::ScrollArea::vertical()
                            .max_height(scroll_height)
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                ui.set_min_width(content_width - Spacing::SM);

                                match self.active_tab {
                                    PreferencesTab::General => self.show_general_tab(ui),
                                    PreferencesTab::Providers => self.show_providers_tab(ui),
                                    PreferencesTab::ApiKeys => self.show_api_keys_tab(ui),
                                    PreferencesTab::Cookies => self.show_cookies_tab(ui),
                                    PreferencesTab::Advanced => self.show_advanced_tab(ui),
                                    PreferencesTab::About => self.show_about_tab(ui),
                                }

                                ui.add_space(Spacing::XL);
                            });
                    });
                });
            });
    }

    fn show_general_tab(&mut self, ui: &mut egui::Ui) {
        // STARTUP section
        section_header(ui, "Startup");

        settings_card(ui, |ui| {
            let mut start_at_login = self.settings.start_at_login;
            if setting_toggle(ui, "Start at login", "Launch CodexBar when you log in", &mut start_at_login) {
                if let Err(e) = self.settings.set_start_at_login(start_at_login) {
                    tracing::error!("Failed to set start at login: {}", e);
                } else {
                    self.settings_changed = true;
                }
            }

            setting_divider(ui);

            let mut start_minimized = self.settings.start_minimized;
            if setting_toggle(ui, "Start minimized", "Start in the system tray", &mut start_minimized) {
                self.settings.start_minimized = start_minimized;
                self.settings_changed = true;
            }
        });

        ui.add_space(Spacing::LG);

        // NOTIFICATIONS section
        section_header(ui, "Notifications");

        settings_card(ui, |ui| {
            let mut show_notifications = self.settings.show_notifications;
            if setting_toggle(ui, "Show notifications", "Alert when usage thresholds are reached", &mut show_notifications) {
                self.settings.show_notifications = show_notifications;
                self.settings_changed = true;
            }

            setting_divider(ui);

            // High warning threshold
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("High warning").size(FontSize::MD).color(Theme::TEXT_PRIMARY));
                    ui.label(RichText::new("Show warning at this usage level").size(FontSize::SM).color(Theme::TEXT_MUTED));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut threshold = self.settings.high_usage_threshold as i32;
                    ui.add(
                        egui::Slider::new(&mut threshold, 50..=95)
                            .suffix("%")
                            .show_value(true)
                    );
                    if threshold as f64 != self.settings.high_usage_threshold {
                        self.settings.high_usage_threshold = threshold as f64;
                        self.settings_changed = true;
                    }
                });
            });

            setting_divider(ui);

            // Critical alert threshold
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Critical alert").size(FontSize::MD).color(Theme::TEXT_PRIMARY));
                    ui.label(RichText::new("Show critical alert at this level").size(FontSize::SM).color(Theme::TEXT_MUTED));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let mut threshold = self.settings.critical_usage_threshold as i32;
                    ui.add(
                        egui::Slider::new(&mut threshold, 80..=100)
                            .suffix("%")
                            .show_value(true)
                    );
                    if threshold as f64 != self.settings.critical_usage_threshold {
                        self.settings.critical_usage_threshold = threshold as f64;
                        self.settings_changed = true;
                    }
                });
            });
        });
    }

    fn show_providers_tab(&mut self, ui: &mut egui::Ui) {
        section_header(ui, "Enabled Providers");

        ui.label(
            RichText::new("Select which AI providers to track. Disabled providers won't be fetched.")
                .size(FontSize::SM)
                .color(Theme::TEXT_MUTED),
        );

        ui.add_space(Spacing::MD);

        settings_card(ui, |ui| {
            let providers = ProviderId::all();
            let len = providers.len();

            for (i, provider_id) in providers.iter().enumerate() {
                let provider_name = provider_id.cli_name();
                let display_name = provider_id.display_name();
                let is_enabled = self.settings.enabled_providers.contains(provider_name);
                let icon = provider_icon(provider_name);
                let color = provider_color(provider_name);

                ui.horizontal(|ui| {
                    // Icon with brand color
                    ui.label(RichText::new(icon).size(FontSize::LG).color(color));
                    ui.add_space(Spacing::SM);

                    // Provider name
                    ui.label(RichText::new(display_name).size(FontSize::MD).color(Theme::TEXT_PRIMARY));

                    // Checkbox on the right
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        let mut enabled = is_enabled;
                        if ui.checkbox(&mut enabled, "").changed() {
                            if enabled {
                                self.settings.enabled_providers.insert(provider_name.to_string());
                            } else {
                                self.settings.enabled_providers.remove(provider_name);
                            }
                            self.settings_changed = true;
                        }
                    });
                });

                if i < len - 1 {
                    setting_divider(ui);
                }
            }
        });
    }

    fn show_api_keys_tab(&mut self, ui: &mut egui::Ui) {
        section_header(ui, "API Keys");

        ui.label(
            RichText::new("Configure access tokens for providers that require authentication.")
                .size(FontSize::SM)
                .color(Theme::TEXT_MUTED),
        );

        ui.add_space(Spacing::MD);

        // Status message
        if let Some((msg, is_error)) = &self.api_key_status_msg {
            status_message(ui, msg, *is_error);
            ui.add_space(Spacing::SM);
        }

        // Provider cards - one per provider
        let api_key_providers = get_api_key_providers();

        for provider_info in &api_key_providers {
            let provider_id = provider_info.id.cli_name();
            let has_key = self.api_keys.has_key(provider_id);
            let is_enabled = self.settings.enabled_providers.contains(provider_id);
            let icon = provider_icon(provider_id);
            let color = provider_color(provider_id);

            // Card with left accent bar
            let accent_color = if has_key { Theme::GREEN } else if is_enabled { Theme::ORANGE } else { Theme::BG_TERTIARY };

            egui::Frame::none()
                .fill(Theme::BG_SECONDARY)
                .rounding(Rounding::same(Radius::MD))
                .inner_margin(egui::Margin::same(0.0))
                .show(ui, |ui| {
                    ui.horizontal(|ui| {
                        // Left accent bar
                        let bar_rect = Rect::from_min_size(
                            ui.cursor().min,
                            Vec2::new(3.0, 72.0),
                        );
                        ui.painter().rect_filled(
                            bar_rect,
                            Rounding {
                                nw: Radius::MD,
                                sw: Radius::MD,
                                ne: 0.0,
                                se: 0.0,
                            },
                            accent_color,
                        );
                        ui.add_space(3.0);

                        // Content
                        ui.vertical(|ui| {
                            ui.add_space(Spacing::SM);

                            // Row 1: Icon, Name, Status badge
                            ui.horizontal(|ui| {
                                ui.add_space(Spacing::SM);
                                ui.label(RichText::new(icon).size(FontSize::XL).color(color));
                                ui.add_space(Spacing::XS);
                                ui.label(
                                    RichText::new(provider_info.name)
                                        .size(FontSize::MD)
                                        .color(Theme::TEXT_PRIMARY)
                                        .strong()
                                );

                                ui.add_space(Spacing::XS);

                                if has_key {
                                    badge(ui, "✓ Set", Theme::GREEN);
                                } else if is_enabled {
                                    badge(ui, "Needs key", Theme::ORANGE);
                                }
                            });

                            // Row 2: Help text + env var
                            ui.horizontal(|ui| {
                                ui.add_space(Spacing::SM);
                                if let Some(env_var) = provider_info.api_key_env_var {
                                    ui.label(
                                        RichText::new(format!("Env: {}", env_var))
                                            .size(FontSize::XS)
                                            .color(Theme::TEXT_MUTED)
                                            .monospace()
                                    );
                                }
                            });

                            ui.add_space(Spacing::XS);

                            // Row 3: Actions
                            ui.horizontal(|ui| {
                                ui.add_space(Spacing::SM);

                                if has_key {
                                    // Show masked key
                                    if let Some(key_info) = self.api_keys.get_all_for_display()
                                        .iter()
                                        .find(|k| k.provider_id == provider_id)
                                    {
                                        ui.label(
                                            RichText::new(&key_info.masked_key)
                                                .size(FontSize::SM)
                                                .color(Theme::TEXT_MUTED)
                                                .monospace()
                                        );
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(Spacing::SM);
                                        if small_button(ui, "Remove", Theme::RED) {
                                            self.api_keys.remove(provider_id);
                                            let _ = self.api_keys.save();
                                            self.api_key_status_msg = Some((
                                                format!("Removed API key for {}", provider_info.name),
                                                false,
                                            ));
                                        }
                                    });
                                } else {
                                    if let Some(url) = provider_info.dashboard_url {
                                        if text_button(ui, "Get key →", Theme::ACCENT_PRIMARY) {
                                            let _ = open::that(url);
                                        }
                                    }

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.add_space(Spacing::SM);
                                        if primary_button(ui, "+ Add Key") {
                                            self.new_api_key_provider = provider_id.to_string();
                                            self.show_api_key_input = true;
                                            self.new_api_key_value.clear();
                                        }
                                    });
                                }
                            });

                            ui.add_space(Spacing::SM);
                        });
                    });
                });

            ui.add_space(Spacing::SM);
        }

        // API Key input modal
        if self.show_api_key_input {
            ui.add_space(Spacing::MD);

            let provider_name = ProviderId::from_cli_name(&self.new_api_key_provider)
                .map(|id| id.display_name())
                .unwrap_or(&self.new_api_key_provider);

            egui::Frame::none()
                .fill(Theme::BG_TERTIARY)
                .stroke(Stroke::new(1.0, Theme::ACCENT_PRIMARY.gamma_multiply(0.4)))
                .rounding(Rounding::same(Radius::LG))
                .inner_margin(Spacing::LG)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new(format!("Enter API Key for {}", provider_name))
                            .size(FontSize::MD)
                            .color(Theme::TEXT_PRIMARY)
                            .strong()
                    );

                    ui.add_space(Spacing::SM);

                    let text_edit = egui::TextEdit::singleline(&mut self.new_api_key_value)
                        .password(true)
                        .desired_width(ui.available_width())
                        .hint_text("Paste your API key here...");
                    ui.add(text_edit);

                    ui.add_space(Spacing::MD);

                    ui.horizontal(|ui| {
                        let can_save = !self.new_api_key_value.trim().is_empty();

                        if ui.add_enabled(
                            can_save,
                            egui::Button::new(
                                RichText::new("Save")
                                    .size(FontSize::SM)
                                    .color(Color32::WHITE)
                            )
                            .fill(if can_save { Theme::GREEN } else { Theme::BG_TERTIARY })
                            .rounding(Rounding::same(Radius::SM))
                            .min_size(Vec2::new(80.0, 32.0))
                        ).clicked() {
                            self.api_keys.set(
                                &self.new_api_key_provider,
                                self.new_api_key_value.trim(),
                                None,
                            );
                            if let Err(e) = self.api_keys.save() {
                                self.api_key_status_msg = Some((format!("Failed to save: {}", e), true));
                            } else {
                                self.api_key_status_msg = Some((
                                    format!("API key saved for {}", provider_name),
                                    false,
                                ));
                                self.show_api_key_input = false;
                                self.new_api_key_value.clear();
                            }
                        }

                        ui.add_space(Spacing::XS);

                        if ui.add(
                            egui::Button::new(
                                RichText::new("Cancel")
                                    .size(FontSize::SM)
                                    .color(Theme::TEXT_MUTED)
                            )
                            .fill(Color32::TRANSPARENT)
                            .stroke(Stroke::new(1.0, Theme::BORDER_SUBTLE))
                            .rounding(Rounding::same(Radius::SM))
                        ).clicked() {
                            self.show_api_key_input = false;
                            self.new_api_key_value.clear();
                        }
                    });
                });
        }
    }

    fn show_cookies_tab(&mut self, ui: &mut egui::Ui) {
        section_header(ui, "Browser Cookies");

        ui.label(
            RichText::new("Cookies are automatically extracted from Chrome, Edge, Brave, and Firefox.")
                .size(FontSize::SM)
                .color(Theme::TEXT_MUTED),
        );

        ui.add_space(Spacing::MD);

        // Status message
        if let Some((msg, is_error)) = &self.cookie_status_msg {
            status_message(ui, msg, *is_error);
            ui.add_space(Spacing::SM);
        }

        // Saved cookies
        let saved_cookies = self.cookies.get_all_for_display();

        if !saved_cookies.is_empty() {
            section_header(ui, "Saved Cookies");

            settings_card(ui, |ui| {
                let mut to_remove: Option<String> = None;
                let len = saved_cookies.len();

                for (i, cookie_info) in saved_cookies.iter().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label(
                            RichText::new(&cookie_info.provider)
                                .size(FontSize::MD)
                                .color(Theme::TEXT_PRIMARY)
                        );
                        ui.label(
                            RichText::new(format!("· {}", &cookie_info.saved_at))
                                .size(FontSize::SM)
                                .color(Theme::TEXT_MUTED)
                        );

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if small_button(ui, "Remove", Theme::RED) {
                                to_remove = Some(cookie_info.provider_id.clone());
                            }
                        });
                    });

                    if i < len - 1 {
                        setting_divider(ui);
                    }
                }

                if let Some(provider_id) = to_remove {
                    self.cookies.remove(&provider_id);
                    let _ = self.cookies.save();
                    self.cookie_status_msg = Some((format!("Removed cookie for {}", provider_id), false));
                }
            });

            ui.add_space(Spacing::LG);
        }

        // Add manual cookie
        section_header(ui, "Add Manual Cookie");

        settings_card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new("Provider").size(FontSize::MD).color(Theme::TEXT_PRIMARY));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    egui::ComboBox::from_id_salt("cookie_provider")
                        .selected_text(if self.new_cookie_provider.is_empty() {
                            "Select..."
                        } else {
                            &self.new_cookie_provider
                        })
                        .show_ui(ui, |ui| {
                            let web_providers = ["claude", "cursor", "kimi"];
                            for provider_name in web_providers {
                                if let Some(id) = ProviderId::from_cli_name(provider_name) {
                                    if ui.selectable_label(
                                        self.new_cookie_provider == provider_name,
                                        id.display_name(),
                                    ).clicked() {
                                        self.new_cookie_provider = provider_name.to_string();
                                    }
                                }
                            }
                        });
                });
            });

            setting_divider(ui);

            ui.label(RichText::new("Cookie header").size(FontSize::MD).color(Theme::TEXT_PRIMARY));
            ui.add_space(Spacing::XS);

            let text_edit = egui::TextEdit::multiline(&mut self.new_cookie_value)
                .desired_width(ui.available_width())
                .desired_rows(3)
                .hint_text("Paste cookie header from browser dev tools");
            ui.add(text_edit);

            ui.add_space(Spacing::MD);

            let can_save = !self.new_cookie_provider.is_empty() && !self.new_cookie_value.is_empty();

            if ui.add_enabled(
                can_save,
                egui::Button::new(
                    RichText::new("Save Cookie")
                        .size(FontSize::SM)
                        .color(Color32::WHITE)
                )
                .fill(if can_save { Theme::ACCENT_PRIMARY } else { Theme::BG_TERTIARY })
                .rounding(Rounding::same(Radius::SM))
            ).clicked() {
                self.cookies.set(&self.new_cookie_provider, &self.new_cookie_value);
                if let Err(e) = self.cookies.save() {
                    self.cookie_status_msg = Some((format!("Failed to save: {}", e), true));
                } else {
                    let provider_name = ProviderId::from_cli_name(&self.new_cookie_provider)
                        .map(|id| id.display_name().to_string())
                        .unwrap_or_else(|| self.new_cookie_provider.clone());
                    self.cookie_status_msg = Some((format!("Cookie saved for {}", provider_name), false));
                    self.new_cookie_provider.clear();
                    self.new_cookie_value.clear();
                }
            }
        });
    }

    fn show_advanced_tab(&mut self, ui: &mut egui::Ui) {
        section_header(ui, "Refresh");

        settings_card(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label(RichText::new("Auto-refresh interval").size(FontSize::MD).color(Theme::TEXT_PRIMARY));
                    ui.label(RichText::new("How often to fetch usage data").size(FontSize::SM).color(Theme::TEXT_MUTED));
                });

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let intervals = [
                        (0, "Manual"),
                        (60, "1 min"),
                        (120, "2 min"),
                        (300, "5 min"),
                        (600, "10 min"),
                        (900, "15 min"),
                    ];

                    egui::ComboBox::from_id_salt("refresh_interval")
                        .selected_text(
                            intervals
                                .iter()
                                .find(|(secs, _)| *secs == self.settings.refresh_interval_secs)
                                .map(|(_, label)| *label)
                                .unwrap_or("5 min"),
                        )
                        .show_ui(ui, |ui| {
                            for (secs, label) in intervals {
                                if ui.selectable_value(
                                    &mut self.settings.refresh_interval_secs,
                                    secs,
                                    label,
                                ).changed() {
                                    self.settings_changed = true;
                                }
                            }
                        });
                });
            });
        });

        ui.add_space(Spacing::LG);

        section_header(ui, "Display");

        settings_card(ui, |ui| {
            let mut merge_icons = self.settings.merge_tray_icons;
            if setting_toggle(ui, "Merge tray icons", "Show all providers in a single tray icon", &mut merge_icons) {
                self.settings.merge_tray_icons = merge_icons;
                self.settings_changed = true;
            }

            setting_divider(ui);

            let mut show_as_used = self.settings.show_as_used;
            if setting_toggle(ui, "Show as used", "Show usage as percentage used (vs remaining)", &mut show_as_used) {
                self.settings.show_as_used = show_as_used;
                self.settings_changed = true;
            }
        });

        ui.add_space(Spacing::LG);

        section_header(ui, "Animations");

        settings_card(ui, |ui| {
            let mut enable_animations = self.settings.enable_animations;
            if setting_toggle(ui, "Enable animations", "Animate charts and UI transitions", &mut enable_animations) {
                self.settings.enable_animations = enable_animations;
                self.settings_changed = true;
            }
        });

        ui.add_space(Spacing::LG);

        section_header(ui, "Fun");

        settings_card(ui, |ui| {
            let mut surprise = self.settings.surprise_animations;
            if setting_toggle(ui, "Surprise me", "Random animations on tray icon", &mut surprise) {
                self.settings.surprise_animations = surprise;
                self.settings_changed = true;
            }
        });
    }

    fn show_about_tab(&mut self, ui: &mut egui::Ui) {
        ui.add_space(Spacing::XL);

        // App branding
        ui.vertical_centered(|ui| {
            // Logo placeholder
            egui::Frame::none()
                .fill(Theme::ACCENT_PRIMARY)
                .rounding(Rounding::same(16.0))
                .inner_margin(Spacing::MD)
                .show(ui, |ui| {
                    ui.label(
                        RichText::new("C")
                            .size(32.0)
                            .color(Color32::WHITE)
                            .strong()
                    );
                });

            ui.add_space(Spacing::MD);

            ui.label(
                RichText::new("CodexBar")
                    .size(FontSize::XXL)
                    .color(Theme::TEXT_PRIMARY)
                    .strong()
            );

            ui.label(
                RichText::new(format!("Version {}", env!("CARGO_PKG_VERSION")))
                    .size(FontSize::SM)
                    .color(Theme::TEXT_MUTED)
            );
        });

        ui.add_space(Spacing::XL);

        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("A Windows port of the macOS CodexBar app.")
                    .size(FontSize::MD)
                    .color(Theme::TEXT_SECONDARY)
            );
            ui.label(
                RichText::new("Track your AI provider usage from the system tray.")
                    .size(FontSize::MD)
                    .color(Theme::TEXT_SECONDARY)
            );
        });

        ui.add_space(Spacing::XL);

        ui.vertical_centered(|ui| {
            ui.horizontal(|ui| {
                if ui.link("GitHub Repository").clicked() {
                    let _ = open::that("https://github.com/Finesssee/Win-CodexBar");
                }
                ui.label(RichText::new("·").color(Theme::TEXT_DIM));
                if ui.link("Original macOS Version").clicked() {
                    let _ = open::that("https://github.com/steipete/CodexBar");
                }
            });
        });

        ui.add_space(Spacing::LG);

        ui.vertical_centered(|ui| {
            if ui.add(
                egui::Button::new(
                    RichText::new("Check for Updates")
                        .size(FontSize::SM)
                        .color(Theme::TEXT_PRIMARY)
                )
                .fill(Theme::BG_SECONDARY)
                .stroke(Stroke::new(1.0, Theme::BORDER_SUBTLE))
                .rounding(Rounding::same(Radius::SM))
            ).clicked() {
                let _ = open::that("https://github.com/Finesssee/Win-CodexBar/releases");
            }
        });

        ui.add_space(Spacing::XXL);

        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new("Built with Rust + egui")
                    .size(FontSize::XS)
                    .color(Theme::TEXT_DIM)
            );
        });
    }
}

// ════════════════════════════════════════════════════════════════════════════════
// HELPER COMPONENTS - Refined, reusable UI elements
// ════════════════════════════════════════════════════════════════════════════════

/// Section header - subtle, uppercase
fn section_header(ui: &mut egui::Ui, text: &str) {
    ui.label(
        RichText::new(text.to_uppercase())
            .size(FontSize::XS)
            .color(Theme::TEXT_SECTION)
    );
    ui.add_space(Spacing::SM);
}

/// Settings card container - grouped settings with rounded corners
fn settings_card(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
    egui::Frame::none()
        .fill(Theme::BG_SECONDARY)
        .rounding(Rounding::same(Radius::LG))
        .inner_margin(Spacing::MD)
        .show(ui, content);
}

/// Divider line between settings in a card
fn setting_divider(ui: &mut egui::Ui) {
    ui.add_space(Spacing::SM);
    let rect = Rect::from_min_size(
        ui.cursor().min,
        Vec2::new(ui.available_width(), 1.0),
    );
    ui.painter().rect_filled(rect, 0.0, Theme::SEPARATOR);
    ui.add_space(Spacing::SM + 1.0);
}

/// Toggle setting row - title, subtitle, and toggle on right
fn setting_toggle(ui: &mut egui::Ui, title: &str, subtitle: &str, value: &mut bool) -> bool {
    let mut changed = false;

    ui.horizontal(|ui| {
        ui.vertical(|ui| {
            ui.label(RichText::new(title).size(FontSize::MD).color(Theme::TEXT_PRIMARY));
            ui.label(RichText::new(subtitle).size(FontSize::SM).color(Theme::TEXT_MUTED));
        });

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.checkbox(value, "").changed() {
                changed = true;
            }
        });
    });

    changed
}

/// Status message banner
fn status_message(ui: &mut egui::Ui, msg: &str, is_error: bool) {
    let (bg_color, text_color, icon) = if is_error {
        (Color32::from_rgba_unmultiplied(239, 68, 68, 15), Theme::RED, "✕")
    } else {
        (Color32::from_rgba_unmultiplied(34, 197, 94, 15), Theme::GREEN, "✓")
    };

    egui::Frame::none()
        .fill(bg_color)
        .rounding(Rounding::same(Radius::SM))
        .inner_margin(Spacing::SM)
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(icon).size(FontSize::SM).color(text_color));
                ui.add_space(Spacing::XS);
                ui.label(RichText::new(msg).size(FontSize::SM).color(text_color));
            });
        });
}

/// Small badge
fn badge(ui: &mut egui::Ui, text: &str, color: Color32) {
    egui::Frame::none()
        .fill(color.gamma_multiply(0.15))
        .rounding(Rounding::same(Radius::XS))
        .inner_margin(egui::Margin::symmetric(Spacing::XS, 2.0))
        .show(ui, |ui| {
            ui.label(
                RichText::new(text)
                    .size(FontSize::XS)
                    .color(color)
            );
        });
}

/// Small text button
fn small_button(ui: &mut egui::Ui, text: &str, color: Color32) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .size(FontSize::SM)
                .color(color)
        )
        .fill(color.gamma_multiply(0.1))
        .rounding(Rounding::same(Radius::SM))
    ).clicked()
}

/// Text-only button (no background)
fn text_button(ui: &mut egui::Ui, text: &str, color: Color32) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .size(FontSize::SM)
                .color(color)
        )
        .fill(Color32::TRANSPARENT)
        .stroke(Stroke::NONE)
    ).clicked()
}

/// Primary action button
fn primary_button(ui: &mut egui::Ui, text: &str) -> bool {
    ui.add(
        egui::Button::new(
            RichText::new(text)
                .size(FontSize::SM)
                .color(Color32::WHITE)
        )
        .fill(Theme::ACCENT_PRIMARY)
        .rounding(Rounding::same(Radius::SM))
    ).clicked()
}
