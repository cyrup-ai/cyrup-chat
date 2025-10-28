//! Notification and UI state management for macOS Platform
//!
//! This module handles platform-native notifications and UI state updates.

use super::core::Platform;
use dioxus::prelude::spawn;

impl Platform {
    /// Show platform-native notification
    #[inline(always)]
    pub async fn show_notification(
        &self,
        message: &str,
        title: &str,
        _icon: Option<&str>,
    ) -> Result<(), String> {
        log::debug!("Showing notification: {} - {}", title, message);

        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let _ = Command::new("osascript")
                .args([
                    "-e",
                    &format!(
                        r#"display notification "{}" with title "{}""#,
                        message, title
                    ),
                ])
                .spawn();
        }

        #[cfg(target_os = "windows")]
        {
            log::debug!("Windows notification: {} - {}", title, message);
        }

        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            let _ = Command::new("notify-send").args(&[title, message]).spawn();
        }

        Ok(())
    }

    /// Update sidebar visibility state
    #[inline(always)]
    pub async fn update_sidebar_visibility(&self, visible: bool) -> Result<(), String> {
        log::debug!("Updating sidebar visibility: {}", visible);

        #[cfg(target_os = "macos")]
        {
            let script = if visible {
                "document.querySelector('.sidebar')?.style.setProperty('display', 'block', 'important'); return 'visible';"
            } else {
                "document.querySelector('.sidebar')?.style.setProperty('display', 'none', 'important'); return 'hidden';"
            };

            match self.execute_javascript(script).await {
                Ok(_) => {
                    log::debug!("Applied sidebar visibility script: visible={}", visible);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to update sidebar visibility: {}", e);
                    Err(format!("Sidebar visibility update failed: {}", e))
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::debug!("Sidebar visibility update not supported on this platform");
            Ok(())
        }
    }

    /// Set text size for the application
    #[inline(always)]
    pub async fn set_text_size(&self, size: f32) -> Result<(), String> {
        log::debug!("Setting text size: {}", size);

        #[cfg(target_os = "macos")]
        {
            let script = format!(
                "document.documentElement.style.setProperty('--base-font-size', '{}px', 'important'); return 'set';",
                size
            );

            match self.execute_javascript(&script).await {
                Ok(_) => {
                    log::debug!("Applied text size CSS: {}px", size);
                    Ok(())
                }
                Err(e) => {
                    log::error!("Failed to set text size: {}", e);
                    Err(format!("Text size update failed: {}", e))
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::debug!("Text size update not supported on this platform");
            Ok(())
        }
    }

    /// Clear all notifications using UNUserNotificationCenter
    #[inline(always)]
    pub fn clear_notifications(&self) -> Result<(), String> {
        log::debug!("Clearing notifications");

        #[cfg(target_os = "macos")]
        {
            use objc2_user_notifications::UNUserNotificationCenter;

            let center = UNUserNotificationCenter::currentNotificationCenter();
            center.removeAllDeliveredNotifications();
            center.removeAllPendingNotificationRequests();

            log::debug!("Successfully cleared all notifications");
            Ok(())
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::debug!("Notification clearing not supported on this platform");
            Ok(())
        }
    }

    /// Check if auto-reload is enabled using NSUserDefaults
    #[inline(always)]
    pub fn should_auto_reload(&self) -> Result<bool, String> {
        #[cfg(target_os = "macos")]
        {
            use objc2_foundation::{NSString, NSUserDefaults};

            let defaults = NSUserDefaults::standardUserDefaults();
            let key = NSString::from_str("CyrupAutoReload");
            let auto_reload = defaults.boolForKey(&key);

            log::debug!("Auto-reload setting: {}", auto_reload);
            Ok(auto_reload)
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::debug!("Auto-reload check not supported on this platform, defaulting to true");
            Ok(true)
        }
    }

    /// Update text size for the application (synchronous wrapper)
    #[inline(always)]
    pub fn update_text_size(&self, size: f32) {
        log::debug!("Updating text size: {}", size);

        // Use spawn_local for non-Send futures (UI operations must run on main thread)
        let platform = self.clone();
        spawn(async move {
            match platform.set_text_size(size).await {
                Ok(_) => log::debug!("Successfully updated text size to {}px", size),
                Err(e) => log::error!("Failed to update text size: {}", e),
            }
        });
    }
}
