//! Conversation and room types for agent chat
//!
//! Aligns with:
//! - src/database/schema.rs conversation table (lines 41-52)
//! - src/database/schema.rs room table (lines 79-88)

use super::agent::AgentTemplateId;
use super::message::MessageId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;

/// Conversation ID newtype wrapper
#[derive(Debug, Eq, PartialEq, Hash, Clone, Default, Serialize, Deserialize, SurrealValue)]
pub struct ConversationId(pub String);

impl std::fmt::Display for ConversationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("ConversationID:{}", self.0))
    }
}

impl From<String> for ConversationId {
    fn from(s: String) -> Self {
        ConversationId(s)
    }
}

impl From<&str> for ConversationId {
    fn from(s: &str) -> Self {
        ConversationId(s.to_string())
    }
}

/// Room ID newtype wrapper for multi-agent conversations
#[derive(Debug, Eq, PartialEq, Hash, Clone, Default, Serialize, Deserialize, SurrealValue)]
pub struct RoomId(pub String);

impl std::fmt::Display for RoomId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("RoomID:{}", self.0))
    }
}

impl From<String> for RoomId {
    fn from(s: String) -> Self {
        RoomId(s)
    }
}

/// Full conversation data structure
///
/// Database mapping (src/database/schema.rs:41-52):
/// - title → title (string)
/// - template_id → template_id (record<agent_template>)
/// - summary → summary (string, default "")
/// - agent_session_id → agent_session_id (option<string>) ← LAZY SPAWN
/// - last_summarized_message_id → last_summarized_message_id (option<record<message>>)
/// - last_message_at → last_message_at (datetime)
/// - created_at → created_at (datetime)
///
/// Design: Q17 from MASTODON_ROSETTA_STONE.md
/// - Lazy spawn: agent_session_id is None until first user message sent
/// - This allows creating conversations without immediately spawning agents
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Conversation {
    pub id: ConversationId,
    pub title: String,
    pub template_id: AgentTemplateId,
    pub summary: String,
    /// Lazy spawn pattern: None = not spawned, Some(id) = active session
    pub agent_session_id: Option<String>,
    pub last_summarized_message_id: Option<MessageId>,
    pub last_message_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Lightweight conversation summary for list views
///
/// Used in sidebar to show conversation history without loading full message thread.
/// Similar pattern to AccountUpdateViewModel in src/view_model/account.rs:168-215
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationSummary {
    pub id: ConversationId,
    pub title: String,
    pub last_message_preview: String,
    pub last_message_timestamp: DateTime<Utc>,
    pub agent_avatar: Option<String>,
    /// Unread message count (Q30 from MASTODON_ROSETTA_STONE.md)
    pub unread_count: u32,
}

/// Multi-agent conversation room (Phase 7 feature)
///
/// Database mapping (src/database/schema.rs:79-88):
/// - title → title (string)
/// - participants → participants (array<string>)
/// - summary → summary (string, default "")
/// - last_summarized_message_id → last_summarized_message_id (option<record<message>>)
/// - last_message_at → last_message_at (datetime)
/// - created_at → created_at (datetime)
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Room {
    pub id: RoomId,
    pub title: String,
    /// Agent template IDs participating in this room
    pub participants: Vec<AgentTemplateId>,
    pub summary: String,
    pub last_summarized_message_id: Option<MessageId>,
    pub last_message_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Lightweight room summary for list views
///
/// Mirrors ConversationSummary pattern but for multi-agent rooms.
/// Used in RoomListProvider to display room history without loading full messages.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RoomSummary {
    pub id: RoomId,
    pub title: String,
    pub participants: Vec<AgentTemplateId>,  // Multi-agent participant list
    pub last_message_preview: String,
    pub last_message_timestamp: DateTime<Utc>,
}

impl Default for Conversation {
    fn default() -> Self {
        let now = chrono::Utc::now();
        Self {
            id: ConversationId::default(),
            title: "New Conversation".to_string(),
            template_id: AgentTemplateId::default(),
            summary: String::new(),
            agent_session_id: None, // ← Lazy spawn: starts unspawned
            last_summarized_message_id: None,
            last_message_at: now,
            created_at: now,
        }
    }
}
