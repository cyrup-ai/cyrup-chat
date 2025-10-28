//! StatusViewModel constructor implementation
//!
//! This module contains the complex constructor logic for creating StatusViewModel
//! instances from raw Status data with comprehensive processing and formatting.

use super::core::StatusViewModel;
use crate::environment::model::Status;
use crate::helper::{clean_html, format_number};
use crate::view_model::account::AccountViewModel;
use crate::view_model::media::{VideoMedia, replace_emoji, status_images};
use crate::view_model::types::{StatusId, Visibility};
use megalodon::entities::attachment::AttachmentType;

impl StatusViewModel {
    /// Create StatusViewModel from Status with comprehensive processing
    ///
    /// # Arguments
    /// * `status` - Raw status data from the API
    ///
    /// # Returns
    /// * `StatusViewModel` - Processed view model with formatted fields and interaction states
    ///
    /// # Performance
    /// Uses efficient processing with conditional recursion for reblog handling
    pub fn new(status: &Status) -> Self {
        let (h, f) = crate::environment::platform::format_datetime(&status.created_at);
        let reblog_status = status
            .reblog
            .as_ref()
            .map(|status| Box::new(StatusViewModel::new(status)));

        let has_reblogged = status.reblogged.unwrap_or_default();
        let is_reblog = reblog_status.is_some();
        let is_favourited = status.favourited.unwrap_or_default();
        let is_bookmarked = status.bookmarked.unwrap_or_default();
        let is_reply = status
            .in_reply_to_id
            .as_ref()
            .or_else(|| {
                // replies to ourselves, for this app, are not considered replies.
                // maybe we need a better name for this (has replied self, has replied others)
                let s = status.in_reply_to_account_id.as_ref();
                if s == Some(&status.account.id) {
                    None
                } else {
                    s
                }
            })
            .is_some();

        let in_reply_to_id = status.in_reply_to_id.clone();

        let status_images = status_images(status);

        let mentions: Vec<_> = status
            .mentions
            .iter()
            .map(|e| format!("@{}", e.acct))
            .collect();

        let media: Vec<_> = status
            .media_attachments
            .iter()
            .filter(|a| matches!(a.r#type, AttachmentType::Video | AttachmentType::Gifv))
            .map(|attachment| VideoMedia {
                preview_url: attachment.preview_url.clone(),
                video_url: attachment.url.clone(),
                description: attachment.description.as_ref().cloned().unwrap_or_default(),
            })
            .collect();

        // if we replied to a conversation, or if we were replied to,
        // then we have a conversation that can be loaded
        let has_conversation = status
            .in_reply_to_id
            .as_ref()
            .map(|e| StatusId(e.clone()))
            .or((status.replies_count > 0).then(|| StatusId(status.id.clone())));

        let (text, content) = match replace_emoji(&status.content, &status.emojis) {
            Some(n) => clean_html(&n),
            None => clean_html(&status.content),
        };

        let replies_count = status.replies_count;
        let replies = format_number(replies_count as i64);
        let replies_title = format!("Replies ({})", replies_count);

        let reblog_count = status.reblogs_count;
        let reblog = format_number(reblog_count as i64);
        let reblog_title = if has_reblogged {
            "Reblog: You reblogged this".to_string()
        } else {
            "Reblog".to_string()
        };

        let favourited_count = status.favourites_count;
        let favourited = format_number(favourited_count as i64);
        let favourited_title = if is_favourited {
            "Favorites: You favourited this".to_string()
        } else {
            "Favorites".to_string()
        };

        let bookmarked_title = if is_bookmarked {
            "Bookmark: You bookmarked this".to_string()
        } else {
            "Bookmark".to_string()
        };

        let share_title = "Share".to_string();

        Self {
            id: StatusId(status.id.clone()),
            uri: status.uri.clone(),
            account: AccountViewModel::new(&status.account),
            status_images,
            created: status.created_at,
            created_human: h,
            created_full: f,
            reblog_status,
            content,
            card: status.card.clone(),
            replies,
            replies_title,
            replies_count,
            is_reply,
            in_reply_to_id,
            has_reblogged,
            is_reblog,
            reblog_count,
            reblog,
            reblog_title,
            is_favourited,
            favourited,
            favourited_count,
            favourited_title,
            bookmarked_title,
            is_bookmarked,
            share_title,
            mentions,
            has_conversation,
            text,
            media,
            visibility: Visibility::from(status.visibility.clone()),
            is_pinned: status.pinned.unwrap_or_default(),
        }
    }
}
