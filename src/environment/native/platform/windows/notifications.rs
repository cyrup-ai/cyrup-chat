//! Windows notification system implementation
//!
//! This module handles Windows Toast notifications and notification management
//! using Windows Runtime APIs and PowerShell integration.

use super::core::{Platform, WindowsNotification};
use std::process::Command;

impl Platform {
    /// Show native Windows Toast notification
    pub async fn show_notification(
        &self,
        message: &str,
        title: &str,
        icon: Option<&str>,
    ) -> Result<(), String> {
        log::debug!("Showing Windows notification: {} - {}", title, message);

        let notification_id = {
            let mut counter = self.notification_counter.lock().unwrap_or_else(|e| {
                log::error!("Failed to lock notification counter: {e}");
                panic!("Notification counter lock poisoned");
            });
            *counter += 1;
            *counter
        };

        // Generate unique toast ID for Windows Toast notifications
        let toast_id = format!("cyrup-toast-{}", notification_id);

        // Create notification record
        let notification = WindowsNotification {
            id: notification_id,
            title: title.to_string(),
            message: message.to_string(),
            timestamp: std::time::SystemTime::now(),
            toast_id: Some(toast_id.clone()),
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

        // Use Windows Runtime APIs for native toast notifications
        #[cfg(target_os = "windows")]
        {
            self.show_native_toast_notification(title, message, &toast_id, icon)
                .await
        }

        #[cfg(not(target_os = "windows"))]
        {
            log::warn!("Windows notifications only available on Windows platform");
            Err("Windows notifications not supported on this platform".to_string())
        }
    }

    #[cfg(target_os = "windows")]
    async fn show_native_toast_notification(
        &self,
        title: &str,
        message: &str,
        toast_id: &str,
        icon: Option<&str>,
    ) -> Result<(), String> {
        use windows::Data::Xml::Dom::*;
        use windows::UI::Notifications::*;
        use windows::core::*;

        // Create XML template for toast notification
        let xml_template = if let Some(icon_path) = icon {
            format!(
                r#"<toast>
                    <visual>
                        <binding template="ToastGeneric">
                            <image placement="appLogoOverride" src="{}"/>
                            <text>{}</text>
                            <text>{}</text>
                        </binding>
                    </visual>
                </toast>"#,
                icon_path,
                html_escape::encode_text(title),
                html_escape::encode_text(message)
            )
        } else {
            format!(
                r#"<toast>
                    <visual>
                        <binding template="ToastGeneric">
                            <text>{}</text>
                            <text>{}</text>
                        </binding>
                    </visual>
                </toast>"#,
                html_escape::encode_text(title),
                html_escape::encode_text(message)
            )
        };

        match self.create_and_show_toast(&xml_template, toast_id).await {
            Ok(_) => {
                log::info!("Windows Toast notification displayed: {}", title);
                Ok(())
            }
            Err(e) => {
                log::warn!("Native toast failed, trying PowerShell fallback: {}", e);
                self.show_powershell_toast_fallback(title, message, toast_id)
                    .await
            }
        }
    }

    #[cfg(target_os = "windows")]
    async fn create_and_show_toast(
        &self,
        xml_content: &str,
        toast_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use windows::Data::Xml::Dom::*;
        use windows::UI::Notifications::*;
        use windows::core::*;

        // Create XML document
        let xml_doc = XmlDocument::new()?;
        xml_doc.LoadXml(&HSTRING::from(xml_content))?;

        // Create toast notification
        let toast = ToastNotification::CreateToastNotification(&xml_doc)?;
        toast.SetTag(&HSTRING::from(toast_id))?;

        // Get toast notifier for the application
        let notifier =
            ToastNotificationManager::CreateToastNotifierWithId(&HSTRING::from("CYRUP"))?;

        // Show the toast
        notifier.Show(&toast)?;

        Ok(())
    }

    #[cfg(target_os = "windows")]
    async fn show_powershell_toast_fallback(
        &self,
        title: &str,
        message: &str,
        toast_id: &str,
    ) -> Result<(), String> {
        let powershell_script = format!(
            r#"
            try {{
                [Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
                [Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
                
                $template = @"
                <toast>
                    <visual>
                        <binding template="ToastGeneric">
                            <text>{}</text>
                            <text>{}</text>
                        </binding>
                    </visual>
                </toast>
                "@
                
                $xml = New-Object Windows.Data.Xml.Dom.XmlDocument
                $xml.LoadXml($template)
                
                $toast = [Windows.UI.Notifications.ToastNotification]::new($xml)
                $toast.Tag = "{}"
                
                $notifier = [Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier("CYRUP")
                $notifier.Show($toast)
                
                Write-Output "Toast notification sent successfully"
            }} catch {{
                Write-Error "Failed to show toast: $_"
                exit 1
            }}
            "#,
            title.replace('"', "\"\""),
            message.replace('"', "\"\""),
            toast_id
        );

        match Command::new("powershell")
            .args(&[
                "-NoProfile",
                "-ExecutionPolicy",
                "Bypass",
                "-Command",
                &powershell_script,
            ])
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    log::info!("PowerShell toast notification sent: {}", title);
                    Ok(())
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    log::error!("PowerShell toast failed: {}", error);
                    Err(format!("PowerShell notification failed: {}", error))
                }
            }
            Err(e) => {
                log::error!("Failed to execute PowerShell: {}", e);
                Err(format!("PowerShell execution failed: {}", e))
            }
        }
    }

    pub fn clear_notifications(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use windows::UI::Notifications::*;

            log::debug!("Clearing Windows toast notifications");

            // Clear all notifications for this app
            match ToastNotificationManager::CreateToastNotifierWithId(
                &windows::core::HSTRING::from("CYRUP"),
            ) {
                Ok(notifier) => {
                    // Get notification history and clear all
                    if let Ok(history) = notifier.History() {
                        if let Err(e) = history.Clear() {
                            log::warn!("Failed to clear notification history: {:?}", e);
                        } else {
                            log::info!("Windows notification history cleared");
                        }
                    }
                }
                Err(e) => {
                    log::error!("Failed to get toast notifier for clearing: {:?}", e);
                    return Err(format!("Failed to clear notifications: {:?}", e));
                }
            }

            // Clear our internal notification tracking
            if let Ok(windows) = self.windows.lock() {
                for window in windows.values() {
                    if let Ok(mut notifications) = window.notifications.lock() {
                        notifications.clear();
                        log::debug!("Internal notification list cleared");
                    }
                }
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("Windows notification clearing only available on Windows".to_string())
        }
    }
}
