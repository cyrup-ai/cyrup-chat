//! Error types for native OS notifications

use thiserror::Error;

/// Result type for notification operations
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Errors that can occur during notification operations
#[derive(Debug, Error)]
pub enum NotificationError {
    #[error("Platform error on {platform}: {message}")]
    PlatformError { platform: String, message: String },

    #[error("Authorization required for {platform}")]
    AuthorizationError { platform: String },

    #[error("Invalid notification content: {0}")]
    InvalidContent(String),

    #[error("Delivery failed: {0}")]
    DeliveryFailed(String),
}
