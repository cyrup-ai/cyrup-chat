//! SurrealDB schema definitions for agent chat
//!
//! Defines 5 tables:
//! 1. agent_template - AI agent configurations (model, system prompt, etc.)
//! 2. conversation - Unified 1:N agent conversations (supports single or multi-agent)
//! 3. message - All messages (user + agent responses)
//! 4. bookmark - Saved messages
//! 5. reaction - Emoji reactions on messages

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
    // Used for: Unified 1:N conversations with agents (single or multi-agent)
    db.query(
        r#"
        DEFINE TABLE conversation SCHEMAFULL;
        DEFINE FIELD title ON conversation TYPE string;
        DEFINE FIELD participants ON conversation TYPE array<string> ASSERT array::len($value) > 0 AND array::len($value) <= 50;
        DEFINE FIELD summary ON conversation TYPE string DEFAULT "";
        DEFINE FIELD agent_sessions ON conversation TYPE object DEFAULT {};
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
        DEFINE FIELD conversation_id ON message TYPE record<conversation> REFERENCE ON DELETE CASCADE;
        DEFINE FIELD author ON message TYPE string;
        DEFINE FIELD author_type ON message TYPE string;
        DEFINE FIELD content ON message TYPE string ASSERT string::len($value) > 0;
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
        -- Author filtering for notifications (DEFECT_008)
        DEFINE INDEX idx_message_author_unread ON message COLUMNS author_type, unread, timestamp;
    "#,
    )
    .await
    .map_err(|e| format!("Schema init failed (message): {}", e))?;

    // Table 4: Bookmarks
    // Used for: User-saved messages (global across conversations)
    db.query(
        r"
        DEFINE TABLE bookmark SCHEMAFULL;
        DEFINE FIELD user_id ON bookmark TYPE string;
        DEFINE FIELD message_id ON bookmark TYPE record<message> REFERENCE ON DELETE CASCADE;
        DEFINE FIELD created_at ON bookmark TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_bookmark_user ON bookmark COLUMNS user_id, created_at;
        -- Bookmark existence check optimization (DEFECT_008)
        DEFINE INDEX idx_bookmark_lookup ON bookmark COLUMNS user_id, message_id;
    ",
    )
    .await
    .map_err(|e| format!("Schema init failed (bookmark): {}", e))?;

    // Table 5: Reactions
    // Used for: Emoji reactions on messages (👍 👎 ❤️ 🎯)
    db.query(
        r"
        DEFINE TABLE reaction SCHEMAFULL;
        DEFINE FIELD message_id ON reaction TYPE record<message> REFERENCE ON DELETE CASCADE;
        DEFINE FIELD user_id ON reaction TYPE string;
        DEFINE FIELD emoji ON reaction TYPE string ASSERT string::len($value) > 0 AND string::len($value) <= 10;
        DEFINE FIELD created_at ON reaction TYPE datetime DEFAULT time::now();
        DEFINE INDEX idx_reaction_msg ON reaction COLUMNS message_id;
        DEFINE INDEX idx_reaction_unique ON reaction COLUMNS message_id, user_id, emoji UNIQUE;
        -- Reaction grouping optimization (DEFECT_008)
        DEFINE INDEX idx_reaction_msg_emoji ON reaction COLUMNS message_id, emoji;
    ",
    )
    .await
    .map_err(|e| format!("Schema init failed (reaction): {}", e))?;

    // Table 6: Schema Version Tracking
    db.query(
        r"
        DEFINE TABLE schema_version SCHEMAFULL;
        DEFINE FIELD version ON schema_version TYPE int;
        DEFINE FIELD applied_at ON schema_version TYPE datetime DEFAULT time::now();
    ",
    )
    .await
    .map_err(|e| format!("Schema init failed (schema_version): {}", e))?;

    Ok(())
}
