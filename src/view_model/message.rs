//! Message types for agent chat
//!
//! Aligns with src/database/schema.rs message table (lines 57-74)

use super::conversation::ConversationId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use surrealdb_types::{Datetime, RecordId, SurrealValue, ToSql};

/// Message ID newtype wrapper
#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, SurrealValue)]
pub struct MessageId(pub RecordId);

impl Default for MessageId {
    fn default() -> Self {
        MessageId(RecordId::new("message", "default"))
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_sql())
    }
}

impl From<RecordId> for MessageId {
    fn from(r: RecordId) -> Self {
        MessageId(r)
    }
}

impl From<String> for MessageId {
    fn from(s: String) -> Self {
        MessageId(RecordId::parse_simple(&s).unwrap_or_else(|_| RecordId::new("message", s)))
    }
}

impl From<&str> for MessageId {
    fn from(s: &str) -> Self {
        MessageId(RecordId::parse_simple(s).unwrap_or_else(|_| RecordId::new("message", s)))
    }
}

/// Message in an agent conversation
///
/// Database mapping (src/database/schema.rs:57-74):
/// - conversation_id → conversation_id (record<conversation>)
/// - author → author (string)
/// - author_type → author_type (string: "human", "agent", "system")
/// - content → content (string)
/// - timestamp → timestamp (datetime)
/// - in_reply_to → in_reply_to (option<record<message>>)
/// - attachments → attachments (array, default [])
/// - message_type → message_type (string: "normal", "error", "system")
/// - unread → unread (bool, default false) ← Q30: Unread tracking
/// - deleted → deleted (bool, default false) ← Q35: Soft delete
/// - pinned → pinned (bool, default false) ← Q37: Pin messages
///
/// Design decisions:
/// - Q30: unread field tracks if user has seen this message (for notification badge)
/// - Q35: deleted=true hides from UI but preserves in database (soft delete)
/// - Q37: pinned=true shows message at top of conversation (max 5 pins)
/// - Q3: attachments stores file paths as Vec<String>
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, SurrealValue)]
pub struct Message {
    pub id: MessageId,
    pub conversation_id: ConversationId,
    pub author: String,
    pub author_type: AuthorType,
    pub content: String,
    pub timestamp: Datetime,
    pub in_reply_to: Option<MessageId>,
    pub message_type: MessageType,
    /// File paths to attachments (Q3 from MASTODON_ROSETTA_STONE.md)
    pub attachments: Vec<String>,
    /// Unread status for notification counts (Q30)
    pub unread: bool,
    /// Soft delete flag - hides from UI but keeps in DB (Q35)
    pub deleted: bool,
    /// Pin to top of conversation (Q37 - max 5 per conversation)
    pub pinned: bool,
}

/// Who authored this message
///
/// Serializes to lowercase for database: "human", "agent", "system"
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, SurrealValue)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum AuthorType {
    /// User sent this message
    #[default]
    Human,
    /// Agent (Claude) sent this message
    Agent,
    /// System-generated message (e.g., "Agent session started")
    System,
    /// Tool execution message (tool calls and results)
    Tool,
}

/// Message type classification
///
/// Serializes to lowercase for database: "normal", "error", "system"
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, SurrealValue)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum MessageType {
    /// Regular user or agent message
    #[default]
    Normal,
    /// Error message (e.g., agent failed to respond)
    Error,
    /// System notification message
    System,
    /// Tool use message (tool calls and results)
    Tool,
}

impl Default for Message {
    fn default() -> Self {
        Self {
            id: MessageId::default(),
            conversation_id: ConversationId::default(),
            author: String::new(),
            author_type: AuthorType::Human,
            content: String::new(),
            timestamp: Utc::now().into(),
            in_reply_to: None,
            message_type: MessageType::Normal,
            attachments: Vec::new(),
            unread: false,  // ← Default: message is read
            deleted: false, // ← Default: message is not deleted
            pinned: false,  // ← Default: message is not pinned
        }
    }
}

impl std::fmt::Display for AuthorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthorType::Human => write!(f, "User"),
            AuthorType::Agent => write!(f, "Assistant"),
            AuthorType::System => write!(f, "System"),
            AuthorType::Tool => write!(f, "Tool"),
        }
    }
}
