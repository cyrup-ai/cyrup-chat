//! Conversation types for agent chat
//!
//! Aligns with:
//! - src/database/schema.rs conversation table (lines 39-55)

use super::agent::AgentTemplateId;
use super::message::MessageId;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use surrealdb_types::{Datetime, RecordId, SurrealValue, ToSql};
use std::collections::HashMap;

/// Conversation ID newtype wrapper
#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ConversationId(pub RecordId);

impl Default for ConversationId {
    fn default() -> Self {
        ConversationId(RecordId::new("conversation", "default"))
    }
}

impl std::fmt::Display for ConversationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_sql())
    }
}

impl From<RecordId> for ConversationId {
    fn from(r: RecordId) -> Self {
        ConversationId(r)
    }
}

impl From<String> for ConversationId {
    fn from(s: String) -> Self {
        ConversationId(RecordId::parse_simple(&s).unwrap_or_else(|_| RecordId::new("conversation", s)))
    }
}

impl From<&str> for ConversationId {
    fn from(s: &str) -> Self {
        ConversationId(RecordId::parse_simple(s).unwrap_or_else(|_| RecordId::new("conversation", s)))
    }
}


/// Full conversation data structure (unified 1:N agent support)
///
/// Database mapping (src/database/schema.rs:39-55):
/// - title → title (string)
/// - participants → participants (array<string>) ← NEW: 1 or more agent template IDs
/// - summary → summary (string, default "")
/// - agent_sessions → agent_sessions (object) ← NEW: HashMap<agent_id, session_id>
/// - last_summarized_message_id → last_summarized_message_id (option<record<message>>)
/// - last_message_at → last_message_at (datetime)
/// - created_at → created_at (datetime)
///
/// Design:
/// - Supports 1:N agents via participants Vec
/// - Lazy spawn: agent_sessions HashMap is empty until agents spawn
/// - Single agent: participants.len() == 1
/// - Multi-agent: participants.len() > 1
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Conversation {
    pub id: ConversationId,
    pub title: String,
    /// Agent template IDs participating in this conversation (1 or more)
    pub participants: Vec<AgentTemplateId>,
    pub summary: String,
    /// Lazy spawn pattern: Maps agent_id → session_id for active agents
    pub agent_sessions: HashMap<String, String>,
    pub last_summarized_message_id: Option<MessageId>,
    pub last_message_at: Datetime,
    pub created_at: Datetime,
}

/// Lightweight conversation summary for list views
///
/// Used in sidebar to show conversation history without loading full message thread.
/// Supports both single and multi-agent conversations.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConversationSummary {
    pub id: ConversationId,
    pub title: String,
    /// All agent participants (1 for single-agent, N for multi-agent)
    pub participants: Vec<AgentTemplateId>,
    pub last_message_preview: String,
    pub last_message_timestamp: Datetime,
    pub agent_avatar: Option<String>,
    /// Unread message count
    pub unread_count: u32,
}

impl Default for Conversation {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            id: ConversationId::default(),
            title: "New Conversation".to_string(),
            participants: vec![AgentTemplateId::default()],
            summary: String::new(),
            agent_sessions: HashMap::new(), // ← Lazy spawn: starts empty
            last_summarized_message_id: None,
            last_message_at: now.into(),
            created_at: now.into(),
        }
    }
}
