use super::core::{LinuxNotification, Platform};
use std::process::Command;

impl Platform {
    /// Update notification badge state
    pub fn update_notification_badge(&self, has_notifications: bool) {
        log::debug!("Updating Linux notification badge: {}", has_notifications);

        // Update badge using DBus LauncherEntry interface
        let badge_count = if has_notifications { "1" } else { "0" };
        let visible = if has_notifications { "true" } else { "false" };

        // Use dbus-send to update Unity launcher badge
        let unity_result = Command::new("dbus-send")
            .args(&[
                "--type=signal",
                "--dest=com.canonical.Unity.LauncherEntry",
                "/com/canonical/unity/launcherentry/cyrup_chat",
                "com.canonical.Unity.LauncherEntry.Update",
                &format!("string:application://cyrup-chat.desktop"),
                &format!(
                    "dict:string:variant:count,int64:{},visible,boolean:{}",
                    badge_count, visible
                ),
            ])
            .spawn();

        // Also try updating via desktop file badge count
        let desktop_result = Command::new("gdbus")
            .args(&[
                "call",
                "--session",
                "--dest",
                "org.freedesktop.Notifications",
                "--object-path",
                "/org/freedesktop/Notifications",
                "--method",
                "org.freedesktop.DBus.Properties.Set",
                "org.freedesktop.Notifications",
                "BadgeCount",
                &format!("<uint32 {}>", if has_notifications { 1 } else { 0 }),
            ])
            .spawn();

        match (unity_result, desktop_result) {
            (Ok(_), Ok(_)) => log::info!(
                "Notification badge updated via Unity and DBus: {}",
                has_notifications
            ),
            (Ok(_), Err(_)) => log::info!(
                "Notification badge updated via Unity: {}",
                has_notifications
            ),
            (Err(_), Ok(_)) => {
                log::info!("Notification badge updated via DBus: {}", has_notifications)
            }
            (Err(e1), Err(e2)) => {
                log::warn!(
                    "Failed to update notification badge - Unity: {}, DBus: {}",
                    e1,
                    e2
                );
            }
        }
    }

    /// Show native Linux notification using libnotify
    pub async fn show_notification(
        &self,
        message: &str,
        title: &str,
        icon: Option<&str>,
    ) -> Result<(), String> {
        log::debug!("Showing Linux notification: {} - {}", title, message);

        let notification_id = {
            let mut counter = self.notification_counter.lock().unwrap_or_else(|e| {
                log::error!("Failed to lock notification counter: {e}");
                panic!("Notification counter lock poisoned");
            });
            *counter += 1;
            *counter
        };

        // Create notification record
        let notification = LinuxNotification {
            id: notification_id,
            title: title.to_string(),
            message: message.to_string(),
            timestamp: std::time::SystemTime::now(),
        };

        // Store notification for tracking
        if let Ok(mut notifications) = self
            .windows
            .lock()
            .and_then(|windows| windows.values().next().map(|w| w.notifications.clone()))
        {
            if let Ok(mut notif_list) = notifications.lock() {
                notif_list.push(notification);

                // Keep only last 50 notifications to prevent memory bloat
                if notif_list.len() > 50 {
                    notif_list.drain(0..notif_list.len() - 50);
                }
            }
        }

        // Send notification via notify-send with proper error handling
        let mut cmd = Command::new("notify-send");
        cmd.args(&[title, message]);

        if let Some(icon_path) = icon {
            cmd.args(&["--icon", icon_path]);
        }

        // Set urgency level for important notifications
        cmd.args(&["--urgency", "normal"]);

        // Set timeout (5 seconds)
        cmd.args(&["--expire-time", "5000"]);

        match cmd.spawn() {
            Ok(_) => {
                log::info!(
                    "Linux notification sent: {} (ID: {})",
                    title,
                    notification_id
                );
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to send Linux notification: {e}");
                Err(format!("Notification failed: {e}"))
            }
        }
    }

    /// Clear all Linux notifications
    pub fn clear_notifications(&self) -> Result<(), String> {
        log::debug!("Clearing Linux notifications");

        // Clear stored notifications from all windows
        let mut cleared_count = 0;
        if let Ok(windows) = self.windows.lock() {
            for window in windows.values() {
                if let Ok(mut notifications) = window.notifications.lock() {
                    cleared_count += notifications.len();
                    notifications.clear();
                }
            }
        }

        // Clear system notifications via DBus
        let dbus_result = Command::new("gdbus")
            .args(&[
                "call",
                "--session",
                "--dest",
                "org.freedesktop.Notifications",
                "--object-path",
                "/org/freedesktop/Notifications",
                "--method",
                "org.freedesktop.Notifications.CloseNotification",
                "uint32:0", // 0 means close all notifications
            ])
            .spawn();

        // Also try clearing notification history in GNOME
        let gnome_result = Command::new("gsettings")
            .args(&[
                "set",
                "org.gnome.desktop.notifications",
                "show-banners",
                "false",
            ])
            .spawn()
            .and_then(|_| {
                Command::new("gsettings")
                    .args(&[
                        "set",
                        "org.gnome.desktop.notifications",
                        "show-banners",
                        "true",
                    ])
                    .spawn()
            });

        match (dbus_result, gnome_result) {
            (Ok(_), Ok(_)) => {
                log::info!(
                    "Cleared {} stored notifications and system notifications via DBus/GNOME",
                    cleared_count
                );
                Ok(())
            }
            (Ok(_), Err(_)) => {
                log::info!(
                    "Cleared {} stored notifications and system notifications via DBus",
                    cleared_count
                );
                Ok(())
            }
            (Err(e1), _) => {
                log::warn!("Failed to clear system notifications via DBus: {}", e1);
                log::info!("Cleared {} stored notifications only", cleared_count);
                Ok(()) // Still return success since we cleared stored notifications
            }
        }
    }
}
