//! Notification service for agent completion notifications
//!
//! Provides simple API for OS notification delivery.

use super::backend::{NotificationRequest, PlatformBackend};
use super::content::NotificationContent;
use super::error::NotificationResult;

#[cfg(target_os = "macos")]
use super::macos_backend::MacOSBackend;

/// Notification service for agent responses
///
/// Handles platform-specific notification delivery with compile-time
/// backend selection.
pub struct NotificationService {
    backend: Box<dyn PlatformBackend>,
}

impl NotificationService {
    /// Create new notification service with platform backend
    pub fn new() -> Self {
        let backend: Box<dyn PlatformBackend> = {
            #[cfg(target_os = "macos")]
            {
                Box::new(MacOSBackend::new())
            }

            #[cfg(not(target_os = "macos"))]
            {
                compile_error!("Notifications only supported on macOS currently")
            }
        };

        Self { backend }
    }

    /// Send OS notification with title and body
    ///
    /// # Arguments
    /// * `title` - Notification title
    /// * `body` - Notification body text
    ///
    /// # Returns
    /// * `Ok(())` - Notification delivered successfully
    /// * `Err(NotificationError)` - Delivery failed
    ///
    /// # Example
    /// ```rust
    /// let service = NotificationService::new();
    /// service.send_notification(
    ///     "Agent Response",
    ///     "Your question has been answered..."
    /// ).await?;
    /// ```
    pub async fn send_notification(
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> NotificationResult<()> {
        let service = Self::new();

        // Create content and validate
        let content = NotificationContent::new(title, body);
        content
            .validate()
            .map_err(super::error::NotificationError::InvalidContent)?;

        // Generate unique notification ID
        let notification_id = uuid::Uuid::new_v4().to_string();
        let request = NotificationRequest::new(notification_id, content);

        // Deliver notification
        service.backend.deliver_notification(&request).await?;

        log::info!("OS notification delivered successfully");
        Ok(())
    }
}

impl Default for NotificationService {
    fn default() -> Self {
        Self::new()
    }
}
