//! Native OS notifications for agent responses
//!
//! Adapted from ecs-notifications with Bevy ECS removed.
//! Provides simple notification delivery for macOS using
//! UserNotifications framework.
//!
//! # Design Decision Q29
//! Triggers native OS notification when agent finishes responding.

pub mod backend;
pub mod content;
pub mod error;
pub mod service;

#[cfg(target_os = "macos")]
pub mod macos_backend;

// Re-export public API
pub use error::{NotificationError, NotificationResult};
pub use service::NotificationService;
