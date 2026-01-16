//! System notifications for CodexBar
//!
//! Provides Windows toast notifications for usage alerts

#![allow(dead_code)]

use crate::core::ProviderId;
use crate::settings::Settings;

/// Notification types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationType {
    /// Usage is approaching limit (high threshold)
    HighUsage,
    /// Usage is critical (critical threshold)
    CriticalUsage,
    /// Usage limit exhausted
    Exhausted,
    /// Provider status issue
    StatusIssue,
}

impl NotificationType {
    pub fn title(&self) -> &'static str {
        match self {
            NotificationType::HighUsage => "High Usage Warning",
            NotificationType::CriticalUsage => "Critical Usage Alert",
            NotificationType::Exhausted => "Usage Limit Reached",
            NotificationType::StatusIssue => "Provider Status Issue",
        }
    }

    pub fn icon(&self) -> &'static str {
        match self {
            NotificationType::HighUsage => "⚠️",
            NotificationType::CriticalUsage => "🔴",
            NotificationType::Exhausted => "🚫",
            NotificationType::StatusIssue => "⚡",
        }
    }
}

/// Notification manager
pub struct NotificationManager {
    /// Track which notifications have been sent to avoid spam
    sent_notifications: std::collections::HashSet<(ProviderId, NotificationType)>,
}

impl NotificationManager {
    pub fn new() -> Self {
        Self {
            sent_notifications: std::collections::HashSet::new(),
        }
    }

    /// Check usage and send notifications if thresholds are crossed
    pub fn check_and_notify(
        &mut self,
        provider: ProviderId,
        used_percent: f64,
        settings: &Settings,
    ) {
        if !settings.show_notifications {
            return;
        }

        let notification_type = if used_percent >= 100.0 {
            Some(NotificationType::Exhausted)
        } else if used_percent >= settings.critical_usage_threshold {
            Some(NotificationType::CriticalUsage)
        } else if used_percent >= settings.high_usage_threshold {
            Some(NotificationType::HighUsage)
        } else {
            // Reset notifications if usage dropped
            self.sent_notifications.retain(|(p, _)| *p != provider);
            None
        };

        if let Some(notif_type) = notification_type {
            let key = (provider, notif_type);
            if !self.sent_notifications.contains(&key) {
                self.send_notification(provider, used_percent, notif_type);
                self.sent_notifications.insert(key);
            }
        }
    }

    /// Send a notification for a status issue
    pub fn notify_status_issue(&mut self, provider: ProviderId, description: &str) {
        let key = (provider, NotificationType::StatusIssue);
        if !self.sent_notifications.contains(&key) {
            self.send_status_notification(provider, description);
            self.sent_notifications.insert(key);
        }
    }

    /// Clear status issue notification (when resolved)
    pub fn clear_status_issue(&mut self, provider: ProviderId) {
        self.sent_notifications.remove(&(provider, NotificationType::StatusIssue));
    }

    /// Send a Windows toast notification
    fn send_notification(&self, provider: ProviderId, used_percent: f64, notif_type: NotificationType) {
        let title = notif_type.title();
        let body = match notif_type {
            NotificationType::HighUsage => {
                format!("{} usage at {:.0}% - approaching limit", provider.display_name(), used_percent)
            }
            NotificationType::CriticalUsage => {
                format!("{} usage at {:.0}% - critically high!", provider.display_name(), used_percent)
            }
            NotificationType::Exhausted => {
                format!("{} usage limit exhausted ({:.0}%)", provider.display_name(), used_percent)
            }
            NotificationType::StatusIssue => {
                format!("{} is experiencing issues", provider.display_name())
            }
        };

        self.show_toast(title, &body);
    }

    fn send_status_notification(&self, provider: ProviderId, description: &str) {
        let title = NotificationType::StatusIssue.title();
        let body = format!("{}: {}", provider.display_name(), description);
        self.show_toast(title, &body);
    }

    #[cfg(target_os = "windows")]
    fn show_toast(&self, title: &str, body: &str) {
        use std::os::windows::process::CommandExt;
        use std::process::Command;

        // Use PowerShell to show a toast notification
        let script = format!(
            r#"
            [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
            [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null

            $template = @"
            <toast>
                <visual>
                    <binding template="ToastText02">
                        <text id="1">{}</text>
                        <text id="2">{}</text>
                    </binding>
                </visual>
            </toast>
"@

            $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
            $xml.LoadXml($template)
            $toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
            [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("CodexBar").Show($toast)
            "#,
            title.replace('"', "'"),
            body.replace('"', "'")
        );

        let _ = Command::new("powershell")
            .args(["-ExecutionPolicy", "Bypass", "-Command", &script])
            .creation_flags(0x08000000) // CREATE_NO_WINDOW
            .spawn();
    }

    #[cfg(not(target_os = "windows"))]
    fn show_toast(&self, title: &str, body: &str) {
        tracing::info!("Notification: {} - {}", title, body);
    }
}

impl Default for NotificationManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple notification function for one-off notifications
pub fn show_notification(title: &str, body: &str) {
    let manager = NotificationManager::new();
    manager.show_toast(title, body);
}
