//! Platform backend trait for notification delivery
//!
//! Adapted from ecs-notifications platform.rs with Bevy removed.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

use super::content::NotificationContent;
use super::error::NotificationResult;

/// Notification delivery request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRequest {
    /// Unique notification identifier
    pub notification_id: String,
    /// Notification content
    pub content: NotificationContent,
}

impl NotificationRequest {
    pub fn new(notification_id: impl Into<String>, content: NotificationContent) -> Self {
        Self {
            notification_id: notification_id.into(),
            content,
        }
    }
}

/// Notification delivery receipt
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryReceipt {
    /// Notification ID that was delivered
    pub notification_id: String,
    /// Platform where notification was delivered
    pub platform: String,
    /// Delivery timestamp
    pub delivered_at: SystemTime,
    /// Success status
    pub success: bool,
}

/// Platform-specific notification backend trait
///
/// Implemented by platform backends (MacOSBackend, WindowsBackend, LinuxBackend)
#[async_trait]
pub trait PlatformBackend: Send + Sync {
    /// Deliver a notification to the platform
    async fn deliver_notification(
        &self,
        request: &NotificationRequest,
    ) -> NotificationResult<DeliveryReceipt>;

    /// Check if platform is authorized to show notifications
    async fn check_authorization(&self) -> NotificationResult<bool>;

    /// Get platform name for debugging
    fn platform_name(&self) -> &'static str;
}
