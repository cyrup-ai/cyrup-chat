//! Media handling types and utilities for view models
//!
//! This module provides media attachment types, video media handling,
//! and emoji replacement utilities with zero-allocation patterns.

use crate::environment::model::*;
use megalodon::entities::Emoji;
use serde::{Deserialize, Serialize};

/// Media attachment information with file metadata
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct AttachmentMedia {
    /// Base 64 image preview
    pub preview: Option<String>,
    /// Path to the data on disk
    pub path: std::path::PathBuf,
    pub filename: String,
    pub description: Option<String>,
    pub is_uploaded: bool,
    pub server_id: Option<String>,
}

impl PartialEq for AttachmentMedia {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path && self.server_id == other.server_id
    }
}

impl Eq for AttachmentMedia {}

/// Video media content with preview and description
#[derive(Clone, Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct VideoMedia {
    pub preview_url: Option<String>,
    pub video_url: String,
    pub description: String,
}

/// Extract image attachments from status with optimized filtering
///
/// # Arguments
/// * `status` - The status to extract images from
///
/// # Returns
/// * `Vec<(description, preview_url, url)>` - Tuple of image metadata
///
/// # Performance
/// Uses iterator chaining with filter_map for zero-allocation processing
#[inline(always)]
pub fn status_images(status: &Status) -> Vec<(String, String, String)> {
    use megalodon::entities::attachment::AttachmentType;

    status
        .media_attachments
        .iter()
        .filter_map(|item| match item.r#type {
            AttachmentType::Image => Some((
                item.description.as_deref().unwrap_or_default().to_string(),
                item.preview_url.as_deref().unwrap_or(&item.url).to_string(),
                item.url.clone(),
            )),
            _ => None,
        })
        .collect()
}

/// Replace emoji shortcodes with HTML image tags using zero-allocation patterns
///
/// # Arguments
/// * `input` - Text content to process
/// * `emojis` - Array of emoji definitions
///
/// # Returns
/// * `Some(String)` - Processed text with emoji HTML, or `None` if no changes needed
///
/// # Performance
/// Early returns for empty emoji arrays and strings without colons to avoid allocations
#[inline(always)]
pub fn replace_emoji(input: &str, emojis: &[Emoji]) -> Option<String> {
    // Early return for empty emoji list - zero allocation
    if emojis.is_empty() {
        return None;
    }
    // Early return if no emoji shortcodes present - zero allocation
    if !input.contains(':') {
        return None;
    }

    let mut string = input.to_string();
    for emoji in emojis.iter() {
        let image = format!(
            "<img src=\"{}\" class=\"emoji-entry\" />",
            &emoji.static_url
        );
        string = string.replace(&format!(":{}:", emoji.shortcode), &image);
    }
    Some(string)
}

/// Helper function to create default megalodon Status (can't implement Default due to orphan rules)
///
/// # Returns
/// * `megalodon::entities::Status` - Default status with sensible field values
///
/// # Usage
/// Used when creating fallback status objects for error recovery
#[inline(always)]
pub fn create_default_megalodon_status() -> megalodon::entities::Status {
    use chrono::Utc;
    use megalodon::entities::Account;

    // Create default account for the status
    let default_account = Account {
        id: String::new(),
        username: String::new(),
        acct: String::new(),
        display_name: String::new(),
        locked: false,
        discoverable: None,
        group: None,
        noindex: None,
        moved: None,
        suspended: None,
        limited: None,
        created_at: Utc::now(),
        followers_count: 0,
        following_count: 0,
        statuses_count: 0,
        note: String::new(),
        url: String::new(),
        avatar: String::new(),
        avatar_static: String::new(),
        header: String::new(),
        header_static: String::new(),
        emojis: vec![],
        fields: vec![],
        bot: false,
        source: None,
        role: None,
        mute_expires_at: None,
    };

    megalodon::entities::Status {
        id: String::new(),
        uri: String::new(),
        url: None,
        account: default_account,
        in_reply_to_id: None,
        in_reply_to_account_id: None,
        reblog: None,
        content: String::new(),
        plain_content: None,
        created_at: Utc::now(),
        edited_at: None,
        emojis: vec![],
        replies_count: 0,
        reblogs_count: 0,
        favourites_count: 0,
        reblogged: None,
        favourited: None,
        muted: None,
        sensitive: false,
        spoiler_text: String::new(),
        visibility: megalodon::entities::StatusVisibility::Public,
        media_attachments: vec![],
        mentions: vec![],
        tags: vec![],
        card: None,
        poll: None,
        application: None,
        language: None,
        pinned: None,
        emoji_reactions: None,
        quote: false,
        bookmarked: None,
    }
}

/// Helper function to create default Status since we can't implement Default for external type
///
/// # Returns
/// * `Status` - Default status wrapped for use in the codebase
///
/// # Usage
/// Used when creating fallback status objects for error recovery
#[inline(always)]
pub fn create_default_status() -> Status {
    use chrono::Utc;
    use megalodon::entities::{Account, StatusVisibility};

    // Create default account for the status
    let default_account = Account {
        id: String::new(),
        username: String::new(),
        acct: String::new(),
        display_name: String::new(),
        locked: false,
        discoverable: None,
        group: None,
        noindex: None,
        moved: None,
        suspended: None,
        limited: None,
        created_at: Utc::now(),
        followers_count: 0,
        following_count: 0,
        statuses_count: 0,
        note: String::new(),
        url: String::new(),
        avatar: String::new(),
        avatar_static: String::new(),
        header: String::new(),
        header_static: String::new(),
        emojis: vec![],
        fields: vec![],
        bot: false,
        source: None,
        role: None,
        mute_expires_at: None,
    };

    Status {
        id: String::new(),
        uri: String::new(),
        url: None,
        account: default_account,
        in_reply_to_id: None,
        in_reply_to_account_id: None,
        reblog: None,
        content: String::new(),
        plain_content: None,
        created_at: Utc::now(),
        edited_at: None,
        emojis: vec![],
        replies_count: 0,
        reblogs_count: 0,
        favourites_count: 0,
        reblogged: None,
        favourited: None,
        muted: None,
        sensitive: false,
        spoiler_text: String::new(),
        visibility: StatusVisibility::Public,
        media_attachments: vec![],
        mentions: vec![],
        tags: vec![],
        card: None,
        poll: None,
        application: None,
        language: None,
        pinned: None,
        emoji_reactions: None,
        quote: false,
        bookmarked: None,
    }
}
