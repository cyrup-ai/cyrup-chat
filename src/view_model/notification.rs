//! Notification view models and related functionality
//!
//! This module provides notification-related view models for displaying
//! user notifications with zero-allocation patterns.

use super::status::StatusViewModel;
use crate::helper::clean_html;
use chrono::{DateTime, Utc};
use megalodon::entities::{Notification, notification::NotificationType};
use serde::{Deserialize, Serialize};

/// Notification view model with formatted message and status context
#[derive(Debug, Eq, PartialEq, Clone, Default, Serialize, Deserialize)]
pub struct NotificationViewModel {
    pub id: String,
    pub message: String,
    pub status: StatusViewModel,
    pub date: DateTime<Utc>,
}

impl NotificationViewModel {
    /// Create NotificationViewModel from Notification with message formatting
    ///
    /// # Arguments
    /// * `notification` - Raw notification data from the API
    ///
    /// # Returns
    /// * `Some(NotificationViewModel)` - Processed notification with formatted message
    /// * `None` - If notification type is not supported or status is missing
    ///
    /// # Performance
    /// Uses efficient string processing with conditional content truncation
    pub fn new(notification: &Notification) -> Option<Self> {
        let status = notification.status.as_ref()?;
        let mut content = status
            .plain_content
            .clone()
            .unwrap_or_else(|| clean_html(&status.content).0);

        // Efficient content truncation with ellipsis
        if content.len() > 140 {
            content = content.chars().take(140).collect();
            content.push('…');
        }

        let username = notification
            .account
            .as_ref()
            .map(|a| a.username.as_str())
            .unwrap_or("Unknown");

        let message = match notification.r#type {
            NotificationType::Mention => {
                format!("{} mentioned you: {content}", username)
            }
            NotificationType::Status => {
                format!("{} shared an update: {content}", username)
            }
            NotificationType::Reblog => {
                format!("{} boosted your post: {content}", username)
            }
            NotificationType::Follow => {
                format!("{} started following you", username)
            }
            NotificationType::FollowRequest => {
                format!("{} requested to follow you", username)
            }
            NotificationType::Favourite => {
                format!("{} favourited your post: {content}", username)
            }
            NotificationType::PollVote => {
                format!("{} voted in your poll: {content}", username)
            }
            NotificationType::PollExpired => {
                format!("A poll you voted in has ended: {content}")
            }
            NotificationType::Update => {
                format!("{} edited a post: {content}", username)
            }
            NotificationType::AdminSignup => {
                format!("New user {} signed up", username)
            }
            NotificationType::AdminReport => {
                format!("New report from {}: {content}", username)
            }
            NotificationType::Reaction => {
                format!("{} reacted to your post: {content}", username)
            }
            NotificationType::Move => {
                format!("{} moved to a new account", username)
            }
            NotificationType::GroupInvited => {
                format!("{} invited you to a group", username)
            }
            NotificationType::App => {
                format!("App notification: {content}")
            }
            NotificationType::Unknown => {
                format!("Unknown notification from {}: {content}", username)
            }
        };

        let status = StatusViewModel::new(status);
        let id = notification.id.clone();

        Some(Self {
            id,
            message,
            status,
            date: notification.created_at,
        })
    }
}
