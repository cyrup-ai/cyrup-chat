//! macOS notification backend using UserNotifications framework
//!
//! Adapted from ecs-notifications macos.rs with Bevy removed.

#[cfg(target_os = "macos")]
use objc2::rc::Retained;
#[cfg(target_os = "macos")]
use objc2_foundation::NSString;
#[cfg(target_os = "macos")]
use objc2_user_notifications::{
    UNAuthorizationStatus, UNMutableNotificationContent, UNNotificationRequest,
    UNTimeIntervalNotificationTrigger, UNUserNotificationCenter,
};

use async_trait::async_trait;
use std::time::SystemTime;

use super::backend::{DeliveryReceipt, NotificationRequest, PlatformBackend};
use super::error::{NotificationError, NotificationResult};

/// macOS notification backend using UserNotifications framework
pub struct MacOSBackend;

impl MacOSBackend {
    pub fn new() -> Self {
        Self
    }

    #[cfg(target_os = "macos")]
    #[allow(dead_code)] // Helper method for notification system - pending full notification integration
    fn get_notification_center() -> Retained<UNUserNotificationCenter> {
        UNUserNotificationCenter::currentNotificationCenter()
    }
}

#[async_trait]
impl PlatformBackend for MacOSBackend {
    async fn deliver_notification(
        &self,
        request: &NotificationRequest,
    ) -> NotificationResult<DeliveryReceipt> {
        #[cfg(target_os = "macos")]
        {
            // Check authorization first
            if !self.check_authorization().await? {
                return Err(NotificationError::AuthorizationError {
                    platform: "macOS".to_string(),
                });
            }

            // Clone data for use in blocking task
            let notification_id = request.notification_id.clone();
            let title = request.content.title.clone();
            let body = request.content.body.clone();

            // Clone notification_id again for use in closure
            let notification_id_for_closure = notification_id.clone();

            // Use spawn_blocking for all objc2 operations to avoid Send issues
            let result = tokio::task::spawn_blocking(move || {
                let (tx, rx) = std::sync::mpsc::channel();

                let content = UNMutableNotificationContent::new();
                let title_ns = NSString::from_str(&title);
                let body_ns = NSString::from_str(&body);

                content.setTitle(&title_ns);
                content.setBody(&body_ns);

                // Set default sound
                let default_sound = objc2_user_notifications::UNNotificationSound::defaultSound();
                content.setSound(Some(&default_sound));

                // Create notification request
                let notification_id_ns = NSString::from_str(&notification_id_for_closure);
                let trigger =
                    UNTimeIntervalNotificationTrigger::triggerWithTimeInterval_repeats(0.1, false);
                let un_request = UNNotificationRequest::requestWithIdentifier_content_trigger(
                    &notification_id_ns,
                    &content,
                    Some(&trigger),
                );

                // Deliver notification
                let center = UNUserNotificationCenter::currentNotificationCenter();

                // Create completion handler block
                let block =
                    block2::StackBlock::new(move |error: *mut objc2_foundation::NSError| {
                        let success = error.is_null();
                        let _ = tx.send(success);
                    });
                let block = block.copy();

                center.addNotificationRequest_withCompletionHandler(&un_request, Some(&block));

                // Wait for completion (blocking)
                rx.recv_timeout(std::time::Duration::from_secs(5))
            })
            .await
            .map_err(|e| NotificationError::DeliveryFailed(format!("Task join error: {}", e)))?
            .map_err(|e| NotificationError::DeliveryFailed(format!("Timeout: {}", e)))?;

            if !result {
                return Err(NotificationError::DeliveryFailed(
                    "macOS delivery failed".to_string(),
                ));
            }

            Ok(DeliveryReceipt {
                notification_id,
                platform: "macOS".to_string(),
                delivered_at: SystemTime::now(),
                success: true,
            })
        }

        #[cfg(not(target_os = "macos"))]
        {
            Err(NotificationError::PlatformError {
                platform: "macOS".to_string(),
                message: "macOS backend not available on this platform".to_string(),
            })
        }
    }

    async fn check_authorization(&self) -> NotificationResult<bool> {
        #[cfg(target_os = "macos")]
        {
            // Use spawn_blocking for objc2 operations
            let result = tokio::task::spawn_blocking(|| {
                let (tx, rx) = std::sync::mpsc::channel();

                unsafe {
                    let center = UNUserNotificationCenter::currentNotificationCenter();

                    let block = block2::StackBlock::new(
                        move |settings: std::ptr::NonNull<
                            objc2_user_notifications::UNNotificationSettings,
                        >| {
                            let auth_status = settings.as_ref().authorizationStatus();
                            let is_authorized = matches!(
                                auth_status,
                                UNAuthorizationStatus::Authorized
                                    | UNAuthorizationStatus::Provisional
                            );
                            let _ = tx.send(is_authorized);
                        },
                    );
                    let block = block.copy();

                    center.getNotificationSettingsWithCompletionHandler(&block);
                }

                // Wait for completion (blocking)
                rx.recv_timeout(std::time::Duration::from_secs(5))
            })
            .await
            .map_err(|e| NotificationError::PlatformError {
                platform: "macOS".to_string(),
                message: format!("Task join error: {}", e),
            })?
            .map_err(|e| NotificationError::PlatformError {
                platform: "macOS".to_string(),
                message: format!("Authorization check timeout: {}", e),
            })?;

            Ok(result)
        }

        #[cfg(not(target_os = "macos"))]
        Ok(false)
    }

    fn platform_name(&self) -> &'static str {
        "macOS"
    }
}

impl Default for MacOSBackend {
    fn default() -> Self {
        Self::new()
    }
}
