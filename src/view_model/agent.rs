//! Agent template types for agent chat
//!
//! Aligns with src/database/schema.rs agent_template table (lines 24-36)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::{RecordId, SurrealValue, ToSql};

/// Agent template ID newtype wrapper following pattern from src/view_model/types.rs
#[derive(Debug, Eq, PartialEq, Hash, Clone, Serialize, Deserialize, SurrealValue)]
pub struct AgentTemplateId(pub RecordId);

impl Default for AgentTemplateId {
    fn default() -> Self {
        AgentTemplateId(RecordId::new("agent_template", "default"))
    }
}

impl std::fmt::Display for AgentTemplateId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.to_sql())
    }
}

impl From<RecordId> for AgentTemplateId {
    fn from(r: RecordId) -> Self {
        AgentTemplateId(r)
    }
}

impl From<String> for AgentTemplateId {
    fn from(s: String) -> Self {
        AgentTemplateId(RecordId::parse_simple(&s).unwrap_or_else(|_| RecordId::new("agent_template", s)))
    }
}

impl From<&str> for AgentTemplateId {
    fn from(s: &str) -> Self {
        AgentTemplateId(RecordId::parse_simple(s).unwrap_or_else(|_| RecordId::new("agent_template", s)))
    }
}

/// Agent template configuration
///
/// Database mapping (src/database/schema.rs:24-36):
/// - name → name (string)
/// - system_prompt → system_prompt (string)
/// - model → model (string: "sonnet", "haiku", "opus")
/// - max_turns → max_turns (int, default 50)
/// - icon → icon (option<string>)
/// - color → color (option<string>)
/// - created_at → created_at (datetime)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SurrealValue)]
pub struct AgentTemplate {
    pub id: AgentTemplateId,
    pub name: String,
    pub system_prompt: String,
    pub model: AgentModel,
    pub max_turns: u32,
    pub icon: Option<String>,
    pub color: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Claude model variants
///
/// Serializes to lowercase strings for database storage:
/// - Sonnet → "sonnet"
/// - Haiku → "haiku"
/// - Opus → "opus"
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, SurrealValue, Default)]
#[serde(rename_all = "lowercase")]
#[surreal(untagged, lowercase)]
pub enum AgentModel {
    #[default]
    Sonnet, // Claude 3.5 Sonnet - balanced performance
    Haiku, // Claude 3.5 Haiku - fast, lightweight
    Opus,  // Claude 3 Opus - most capable
}

impl std::fmt::Display for AgentModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AgentModel::Sonnet => write!(f, "Claude 3.5 Sonnet"),
            AgentModel::Haiku => write!(f, "Claude 3 Haiku"),
            AgentModel::Opus => write!(f, "Claude 3 Opus"),
        }
    }
}

impl Default for AgentTemplate {
    fn default() -> Self {
        Self {
            id: AgentTemplateId::default(),
            name: "Default Agent".to_string(),
            system_prompt: "You are a helpful AI assistant.".to_string(),
            model: AgentModel::default(),
            max_turns: 50,
            icon: None,
            color: None,
            created_at: chrono::Utc::now(),
        }
    }
}
