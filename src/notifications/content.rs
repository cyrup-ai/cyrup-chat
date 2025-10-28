//! Simplified notification content types
//!
//! Adapted from ecs-notifications with Bevy components removed
//! and focus on simple text notifications only.

use serde::{Deserialize, Serialize};

/// Notification content with title and body
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationContent {
    /// Notification title (required)
    pub title: String,
    /// Notification body text
    pub body: String,
}

impl NotificationContent {
    /// Create new notification with title and body
    pub fn new(title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
        }
    }

    /// Validate content meets basic requirements
    pub fn validate(&self) -> Result<(), String> {
        if self.title.is_empty() {
            return Err("Title cannot be empty".to_string());
        }

        if self.body.is_empty() {
            return Err("Body cannot be empty".to_string());
        }

        // Truncate to platform limits if needed
        if self.title.len() > 256 {
            return Err("Title exceeds 256 characters".to_string());
        }

        if self.body.len() > 2048 {
            return Err("Body exceeds 2048 characters".to_string());
        }

        Ok(())
    }
}
