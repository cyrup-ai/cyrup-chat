//! Core StatusViewModel definition and trait implementations
//!
//! This module contains the main StatusViewModel struct definition and its
//! associated trait implementations for equality, debugging, and serialization.

use crate::environment::model::Status;
use crate::helper::HtmlItem;
use crate::view_model::account::AccountViewModel;
use crate::view_model::media::VideoMedia;
use crate::view_model::types::{StatusId, Visibility};
use chrono::{DateTime, Utc};
use megalodon::entities::Card;
use serde::{Deserialize, Serialize};

/// Comprehensive status view model with formatted display fields
#[derive(Default, Clone, Serialize, Deserialize)]
pub struct StatusViewModel {
    pub id: StatusId,
    pub uri: String,
    pub account: AccountViewModel,
    pub status_images: Vec<(String, String, String)>,
    pub created: DateTime<Utc>,
    pub created_human: String,
    pub created_full: String,
    pub reblog_status: Option<Box<StatusViewModel>>,
    pub content: Vec<HtmlItem>,
    pub card: Option<Card>,
    pub replies: String,
    pub replies_title: String,
    #[serde(default)]
    pub replies_count: u32,
    /// Is this a reply, except if it is a reply to ourselves
    pub is_reply: bool,
    /// The ID of the status this is replying to (None if not a reply)
    #[serde(default)]
    pub in_reply_to_id: Option<String>,
    /// Has the *current user* reblogged this
    #[serde(default)]
    pub has_reblogged: bool,
    /// Is this a reblog
    #[serde(default)]
    pub is_reblog: bool,
    #[serde(default)]
    pub reblog_count: u32,
    pub reblog: String,
    pub reblog_title: String,
    pub is_favourited: bool,
    pub favourited: String,
    #[serde(default)]
    pub favourited_count: u32,
    pub favourited_title: String,
    pub bookmarked_title: String,
    pub is_bookmarked: bool,
    pub share_title: String,
    pub mentions: Vec<String>,
    pub has_conversation: Option<StatusId>,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub media: Vec<VideoMedia>,
    #[serde(default)]
    pub visibility: Visibility,
    #[serde(default)]
    pub is_pinned: bool,
}

impl PartialEq for StatusViewModel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.account == other.account
            && self.is_bookmarked == other.is_bookmarked
            && self.is_favourited == other.is_favourited
            && self.is_reblog == other.is_reblog
            && self.is_reply == other.is_reply
            && self.in_reply_to_id == other.in_reply_to_id
            && self.replies_count == other.replies_count
            && self.reblog_status == other.reblog_status
            && self.reblog == other.reblog
            && self.favourited_count == other.favourited_count
            && self.favourited == other.favourited
    }
}

impl Eq for StatusViewModel {}

impl std::fmt::Debug for StatusViewModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatusViewModel")
            .field("id", &self.id)
            .field("user", &self.account.username)
            .finish()
    }
}

/// Implement From trait for Status to StatusViewModel conversion
impl From<Status> for StatusViewModel {
    #[inline(always)]
    fn from(status: Status) -> Self {
        StatusViewModel::new(&status)
    }
}
