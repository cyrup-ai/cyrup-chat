mod view;
pub mod mention_input;

pub use view::ChatComponent;

use chrono::{DateTime, Local};
use surrealdb_types::ToSql;

#[derive(Clone, Debug, PartialEq)]
pub struct ChatMessage {
    pub id: String,
    pub sender: MessageSender,
    pub content: String,
    pub timestamp: DateTime<Local>,
    pub in_reply_to: Option<String>,        // Parent message ID
    pub reply_to_author: Option<String>,    // Parent author name (for display)
    pub pinned: bool,                       // Pin to top of conversation (max 5)
    pub reactions: Vec<ReactionSummary>,    // Aggregated reaction data for display
    pub unread: bool,                       // Unread status for notification tracking
    pub is_error: bool,                     // Error message flag for distinct styling
}

#[derive(Clone, Debug, PartialEq)]
pub enum MessageSender {
    User,
    Cyrup,
    System,
    Tool,
}

/// Aggregated reaction data for display
#[derive(Clone, Debug, PartialEq)]
pub struct ReactionSummary {
    pub emoji: String,
    pub count: u32,
    pub user_reacted: bool, // Did current user react with this emoji?
}

impl ChatMessage {
    /// Create ChatMessage from database Message
    ///
    /// Uses database Message.id instead of generating random UUID.
    /// Converts Utc timestamp to Local for display.
    pub fn from_db_message(msg: crate::view_model::message::Message) -> Self {
        use crate::view_model::message::{AuthorType, MessageType};
        use chrono::TimeZone;

        let sender = match msg.author_type {
            AuthorType::Human => MessageSender::User,
            AuthorType::Agent => MessageSender::Cyrup,
            AuthorType::System => MessageSender::System,
            AuthorType::Tool => MessageSender::Tool,
        };

        // Convert Utc to Local timezone
        let local_timestamp = chrono::Local
            .timestamp_opt(msg.timestamp.timestamp(), 0)
            .single()
            .unwrap_or_else(chrono::Local::now);

        Self {
            id: msg.id.to_sql(), // Use database ID, not random UUID
            sender,
            content: msg.content,
            timestamp: local_timestamp,
            in_reply_to: msg.in_reply_to.as_ref().map(|id| id.to_sql()),
            reply_to_author: None,  // Will be populated when loading thread context
            pinned: msg.pinned,
            reactions: Vec::new(),  // Populated separately via LIVE QUERY
            unread: msg.unread,
            is_error: msg.message_type == MessageType::Error,
        }
    }

    #[allow(dead_code)] // Constructor for user messages - pending chat UI integration
    pub fn new_user(content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender: MessageSender::User,
            content,
            timestamp: Local::now(),
            in_reply_to: None,
            reply_to_author: None,
            pinned: false,
            reactions: Vec::new(),
            unread: false,
            is_error: false,
        }
    }

    #[allow(dead_code)] // Constructor for Cyrup AI messages - pending chat UI integration
    pub fn new_cyrup(content: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            sender: MessageSender::Cyrup,
            content,
            timestamp: Local::now(),
            in_reply_to: None,
            reply_to_author: None,
            pinned: false,
            reactions: Vec::new(),
            unread: false,
            is_error: false,
        }
    }
}
