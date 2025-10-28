//! SurrealDB schema definitions for agent chat
//!
//! Defines 6 tables:
//! 1. agent_template - AI agent configurations (model, system prompt, etc.)
//! 2. conversation - 1:1 agent conversations
//! 3. message - All messages (user + agent responses)
//! 4. room - Multi-agent conversations (Phase 7 feature)
//! 5. bookmark - Saved messages
//! 6. reaction - Emoji reactions on messages

use surrealdb::Surreal;
use surrealdb::engine::local::Db;

/// Initialize database schema with tables and indexes
///
/// Creates all tables in SCHEMAFULL mode with typed fields and indexes.
/// Safe to call multiple times - SurrealDB handles "IF NOT EXISTS" internally.
///
/// # Errors
/// Returns error if schema queries fail
pub async fn init_schema(db: &Surreal<Db>) -> Result<(), String> {
    // Table 1: Agent Templates
    // Used for: Configurable AI agent personalities and models
    db.query(
        r"
        DEFINE TABLE agent_template SCHEMAFULL;
        DEFINE FIELD name ON agent_template TYPE string;
        DEFINE FIELD system_prompt ON agent_template TYPE string;
        DEFINE FIELD model ON agent_template TYPE string;
        DEFINE FIELD max_turns ON agent_template TYPE int DEFAULT 50;
        DEFINE FIELD icon ON agent_template TYPE option<string>;
        DEFINE FIELD color ON agent_template TYPE option<string>;
        DEFINE FIELD created_at ON agent_template TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_template_name ON agent_template COLUMNS name;
    ",
    )
    .await
    .map_err(|e| format!("Schema init failed (agent_template): {}", e))?;

    // Table 2: Conversations
    // Used for: 1:1 conversations with agents (lazy spawn pattern)
    db.query(
        r#"
        DEFINE TABLE conversation SCHEMAFULL;
        DEFINE FIELD title ON conversation TYPE string;
        DEFINE FIELD template_id ON conversation TYPE record<agent_template>;
        DEFINE FIELD summary ON conversation TYPE string DEFAULT "";
        DEFINE FIELD agent_session_id ON conversation TYPE option<string>;
        DEFINE FIELD last_summarized_message_id ON conversation TYPE option<record<message>>;
        DEFINE FIELD last_message_at ON conversation TYPE datetime;
        DEFINE FIELD created_at ON conversation TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_conv_updated ON conversation COLUMNS last_message_at;
    "#,
    )
    .await
    .map_err(|e| format!("Schema init failed (conversation): {}", e))?;

    // Table 3: Messages
    // Used for: All conversation messages with unread/deleted/pinned state
    db.query(
        r#"
        DEFINE TABLE message SCHEMAFULL;
        DEFINE FIELD conversation_id ON message TYPE record<conversation>;
        DEFINE FIELD author ON message TYPE string;
        DEFINE FIELD author_type ON message TYPE string;
        DEFINE FIELD content ON message TYPE string;
        DEFINE FIELD timestamp ON message TYPE datetime DEFAULT time::now();
        DEFINE FIELD in_reply_to ON message TYPE option<record<message>>;
        DEFINE FIELD attachments ON message TYPE array DEFAULT [];
        DEFINE FIELD message_type ON message TYPE string DEFAULT "normal";
        DEFINE FIELD unread ON message TYPE bool DEFAULT false;
        DEFINE FIELD deleted ON message TYPE bool DEFAULT false;
        DEFINE FIELD pinned ON message TYPE bool DEFAULT false;
        DEFINE INDEX idx_msg_conv ON message COLUMNS conversation_id, timestamp;
        DEFINE INDEX idx_msg_unread ON message COLUMNS conversation_id, unread;
        DEFINE INDEX idx_msg_pinned ON message COLUMNS conversation_id, pinned;
    "#,
    )
    .await
    .map_err(|e| format!("Schema init failed (message): {}", e))?;

    // Table 4: Rooms
    // Used for: Multi-agent conversations (Phase 7 feature)
    db.query(
        r#"
        DEFINE TABLE room SCHEMAFULL;
        DEFINE FIELD title ON room TYPE string;
        DEFINE FIELD participants ON room TYPE array<string>;
        DEFINE FIELD summary ON room TYPE string DEFAULT "";
        DEFINE FIELD last_summarized_message_id ON room TYPE option<record<message>>;
        DEFINE FIELD last_message_at ON room TYPE datetime;
        DEFINE FIELD created_at ON room TYPE datetime DEFAULT time::now();
    "#,
    )
    .await
    .map_err(|e| format!("Schema init failed (room): {}", e))?;

    // Table 5: Bookmarks
    // Used for: User-saved messages (global across conversations)
    db.query(
        r"
        DEFINE TABLE bookmark SCHEMAFULL;
        DEFINE FIELD user_id ON bookmark TYPE string;
        DEFINE FIELD message_id ON bookmark TYPE record<message>;
        DEFINE FIELD created_at ON bookmark TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_bookmark_user ON bookmark COLUMNS user_id, created_at;
    ",
    )
    .await
    .map_err(|e| format!("Schema init failed (bookmark): {}", e))?;

    // Table 6: Reactions
    // Used for: Emoji reactions on messages (👍 👎 ❤️ 🎯)
    db.query(
        r"
        DEFINE TABLE reaction SCHEMAFULL;
        DEFINE FIELD message_id ON reaction TYPE record<message>;
        DEFINE FIELD user_id ON reaction TYPE string;
        DEFINE FIELD emoji ON reaction TYPE string;
        DEFINE FIELD created_at ON reaction TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_reaction_msg ON reaction COLUMNS message_id;
    ",
    )
    .await
    .map_err(|e| format!("Schema init failed (reaction): {}", e))?;

    Ok(())
}
