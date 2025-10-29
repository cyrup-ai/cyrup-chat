//! Conversation database operations
//!
//! Provides CRUD operations for conversation table and complex queries
//! joining with message table for summaries and unread counts.
//!
//! Aligns with src/database/schema.rs conversation table (lines 39-55)

use super::Database;
use crate::view_model::agent::AgentTemplateId;
use crate::view_model::conversation::{Conversation, ConversationId, ConversationSummary};
use crate::view_model::message::MessageId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb_types::{Datetime, RecordId, SurrealValue, ToSql};

impl Database {
    /// Create a new conversation in the database
    ///
    /// # Arguments
    /// * `conversation` - Conversation to create (id field ignored, DB generates ID)
    ///
    /// # Returns
    /// * `Ok(String)` - Database-generated conversation ID
    /// * `Err(String)` - Error message if creation fails
    ///
    /// # Database Operation
    /// Inserts into conversation table with agent_sessions = {} (lazy spawn pattern).
    /// SurrealDB auto-generates ID and sets created_at to current time.
    ///
    /// # Lazy Spawn Pattern
    /// New conversations start with agent_sessions = empty HashMap.
    /// Call update_agent_session() after spawning agents on first message.
    pub async fn create_conversation(&self, conversation: &Conversation) -> Result<String, String> {
        // Validate participants not empty (follows pattern from notifications/content.rs:28)
        if conversation.participants.is_empty() {
            return Err("Conversation must have at least one participant".to_string());
        }

        // Serialize conversation fields for database insertion
        #[derive(Serialize, SurrealValue)]
        struct ConversationInsert {
            title: String,
            participants: Vec<String>,
            summary: String,
            agent_sessions: HashMap<String, String>,
            last_message_at: Datetime,
        }

        let insert_data = ConversationInsert {
            title: conversation.title.clone(),
            participants: conversation.participants.iter().map(|p| p.0.to_sql()).collect(),
            summary: conversation.summary.clone(),
            agent_sessions: conversation.agent_sessions.clone(),
            last_message_at: conversation.last_message_at,
        };

        // .create() returns Option<T>, not Vec<Thing>
        let result: Option<Conversation> = self
            .client()
            .create("conversation")
            .content(insert_data)
            .await
            .map_err(|e| format!("Failed to create conversation: {}", e))?;

        // Extract ID from created record
        result
            .map(|c| c.id.0.to_sql())
            .ok_or_else(|| "Create returned empty result".to_string())
    }

    /// Retrieve a single conversation by ID
    ///
    /// # Arguments
    /// * `id` - Conversation record ID (e.g., "conversation:ulid")
    ///
    /// # Returns
    /// * `Ok(Conversation)` - Found conversation with all fields
    /// * `Err(String)` - Error if not found or query fails
    ///
    /// # Database Operation
    /// SELECT * FROM conversation WHERE id = $id
    pub async fn get_conversation(&self, id: &str) -> Result<Conversation, String> {
        // Define response struct matching database schema
        #[derive(Deserialize, SurrealValue)]
        struct ConversationRecord {
            id: RecordId,
            title: String,
            participants: Vec<RecordId>,
            summary: String,
            agent_sessions: HashMap<String, String>,
            last_summarized_message_id: Option<RecordId>,
            last_message_at: Datetime,
            created_at: Datetime,
        }

        let record: Option<ConversationRecord> =
            self.client()
                .select(("conversation", id))
                .await
                .map_err(|e| format!("Failed to get conversation: {}", e))?;

        let record = record.ok_or_else(|| format!("Conversation not found: {}", id))?;

        // Map database record to Conversation view model
        Ok(Conversation {
            id: ConversationId(record.id),
            title: record.title,
            participants: record.participants.into_iter().map(AgentTemplateId).collect(),
            summary: record.summary,
            agent_sessions: record.agent_sessions,
            last_summarized_message_id: record.last_summarized_message_id.map(MessageId),
            last_message_at: record.last_message_at,
            created_at: record.created_at,
        })
    }

    /// List all conversations with summaries for sidebar display
    ///
    /// # Returns
    /// * `Ok(Vec<ConversationSummary>)` - Conversation list ordered by last_message_at DESC
    /// * `Err(String)` - Error if query fails
    ///
    /// # Database Operation
    /// Uses FOR loop pattern for correlated subqueries since SurrealDB 3.0
    /// doesn't support $parent variable. Iterates over conversations and computes
    /// related message data using LET statements with $conv.id reference.
    pub async fn list_conversations(&self) -> Result<Vec<ConversationSummary>, String> {
        // Query using FOR loop pattern for correlated subqueries
        let query = r"
            FOR $conv IN (SELECT id, title, participants, last_message_at 
                          FROM conversation 
                          ORDER BY last_message_at DESC) {
                LET $last_msg_record = SELECT content, timestamp 
                                        FROM message 
                                        WHERE conversation_id = $conv.id 
                                          AND deleted = false 
                                        ORDER BY timestamp DESC 
                                        LIMIT 1;
                LET $unread = SELECT VALUE count() 
                               FROM message 
                               WHERE conversation_id = $conv.id 
                                 AND unread = true 
                                 AND deleted = false;
                LET $preview = IF $last_msg_record[0] != NONE THEN $last_msg_record[0].content ELSE 'No messages yet' END;
                RETURN {
                    id: $conv.id,
                    title: $conv.title,
                    participants: $conv.participants,
                    last_message_preview: $preview,
                    last_message_timestamp: $conv.last_message_at,
                    unread_count: $unread[0] ?? 0
                };
            }
        ";

        // Execute query and extract result set
        let mut response = self
            .client()
            .query(query)
            .await
            .map_err(|e| format!("Failed to list conversations: {}", e))?;

        // Define response struct matching query result
        #[derive(Deserialize, SurrealValue)]
        struct QueryResult {
            id: RecordId,
            title: String,
            participants: Vec<RecordId>,
            last_message_preview: String,
            last_message_timestamp: Datetime,
            unread_count: u32,
        }

        let results: Option<Vec<QueryResult>> = response
            .take(0)
            .map_err(|e| format!("Failed to parse conversations: {}", e))?;

        // Map query results to ConversationSummary view models
        Ok(results
            .unwrap_or_default()
            .into_iter()
            .map(|r| ConversationSummary {
                id: ConversationId(r.id),
                title: r.title,
                participants: r.participants.into_iter().map(AgentTemplateId).collect(),
                last_message_preview: r.last_message_preview,
                last_message_timestamp: r.last_message_timestamp,
                agent_avatar: None,
                unread_count: r.unread_count,
            })
            .collect())
    }

    /// Update conversation summary and title
    ///
    /// # Arguments
    /// * `id` - Conversation ID to update
    /// * `summary` - New summary text (generated by summarizer service)
    /// * `title` - New title (extracted from first messages or generated)
    ///
    /// # Returns
    /// * `Ok(())` - Update succeeded
    /// * `Err(String)` - Error if update fails
    ///
    /// # Database Operation
    /// UPDATE conversation SET summary = $summary, title = $title WHERE id = $id
    pub async fn update_conversation_summary(
        &self,
        id: &str,
        summary: &str,
        title: &str,
    ) -> Result<(), String> {
        // Use .bind() with tuples, NOT HashMap with sql::Value
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            UPDATE conversation 
            SET summary = $summary, title = $title 
            WHERE id = type::record('conversation', $id)
        ";

        self.client()
            .query(query)
            .bind(("id", id.to_string()))
            .bind(("summary", summary.to_string()))
            .bind(("title", title.to_string()))
            .await
            .map_err(|e| format!("Failed to update summary: {}", e))?;

        Ok(())
    }

    /// Update conversation with spawned agent session ID for a specific agent
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID
    /// * `agent_id` - Agent template ID
    /// * `session_id` - Spawned agent session ID from MCP server
    ///
    /// # Returns
    /// * `Ok(())` - Update succeeded
    /// * `Err(String)` - Error if update fails
    ///
    /// # Database Operation
    /// UPDATE conversation SET agent_sessions[$agent_id] = $session WHERE id = $id
    ///
    /// Update agent session ID for a specific agent in a conversation
    ///
    /// Uses SurrealDB bracket notation for safe dynamic field access.
    /// The syntax `agent_sessions[$agent_id]` leverages Part::Value(Expr)
    /// from the SurrealDB parser to allow parameterized field names.
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation record ID
    /// * `agent_id` - Agent template ID (validated against agent_template table)
    /// * `session_id` - MCP session ID to store
    ///
    /// # Returns
    /// * `Ok(())` - Update succeeded
    /// * `Err(String)` - Error if validation fails or database operation fails
    ///
    /// # Security
    /// - Zero string interpolation
    /// - All values properly parameterized via .bind()
    /// - Agent validation via get_template() before modification
    /// - Bracket notation prevents SQL injection for dynamic field names
    ///
    /// # Lazy Spawn Pattern
    /// This method implements the lazy spawn pattern:
    /// 1. Conversation created with agent_sessions = {}
    /// 2. User sends first message → spawn agent(s) via MCP
    /// 3. Call this method to store session_id for each agent
    /// 4. Future messages reuse these session_ids
    pub async fn update_agent_session(
        &self,
        conversation_id: &str,
        agent_id: &str,
        session_id: &str,
    ) -> Result<(), String> {
        // Validate agent_id exists in templates table
        self.get_template(agent_id)
            .await
            .map_err(|_| format!("Invalid agent_id: {}", agent_id))?;

        // ✅ SAFE: Bracket notation with parameterized field name
        // The syntax agent_sessions[$agent_id] uses Part::Value(Expr)
        // which allows the parameter to be safely evaluated as a dynamic key
        let query = r"
            UPDATE conversation
            SET agent_sessions[$agent_id] = $session
            WHERE id = type::record('conversation', $id)
        ";

        self.client()
            .query(query)
            .bind(("agent_id", agent_id.to_string()))      // ✅ Field name parameter
            .bind(("session", session_id.to_string()))      // ✅ Field value parameter
            .bind(("id", conversation_id.to_string()))      // ✅ Record ID parameter
            .await
            .map_err(|e| format!("Failed to update agent session: {}", e))?;

        Ok(())
    }

    /// Add a new participant to an existing conversation
    ///
    /// Uses array::union() to prevent duplicates automatically.
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID
    /// * `agent_id` - Agent template ID to add
    ///
    /// # Returns
    /// * `Ok(())` - Update succeeded
    /// * `Err(String)` - Error if update fails
    ///
    /// # Database Operation
    /// UPDATE conversation SET participants = array::union(participants, [$agent_id]) WHERE id = $id
    pub async fn add_participant(
        &self,
        conversation_id: &str,
        agent_id: &str,
    ) -> Result<(), String> {
        // Validate agent exists (reuses pattern from DEFECT 1 fix)
        self.get_template(agent_id)
            .await
            .map_err(|_| format!("Invalid agent_id: {}", agent_id))?;

        // Use array::union() for automatic deduplication
        let query = r"
            UPDATE conversation
            SET participants = array::union(participants, [$agent_id])
            WHERE id = type::record('conversation', $id)
        ";

        self.client()
            .query(query)
            .bind(("id", conversation_id.to_string()))
            .bind(("agent_id", agent_id.to_string()))
            .await
            .map_err(|e| format!("Failed to add participant: {}", e))?;

        Ok(())
    }

    /// Update last_message_at timestamp
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID
    /// * `timestamp` - New last message timestamp
    ///
    /// # Returns
    /// * `Ok(())` - Update succeeded
    /// * `Err(String)` - Error if update fails
    ///
    /// # Database Operation
    /// UPDATE conversation SET last_message_at = $timestamp WHERE id = $id
    ///
    /// # Usage
    /// Called after inserting new message to keep conversation list sorted correctly.
    /// The idx_conv_updated index uses last_message_at DESC for efficient list queries.
    pub async fn update_last_message_at(
        &self,
        conversation_id: &str,
        timestamp: Datetime,
    ) -> Result<(), String> {
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            UPDATE conversation 
            SET last_message_at = $timestamp 
            WHERE id = type::record('conversation', $id)
        ";

        self.client()
            .query(query)
            .bind(("id", conversation_id.to_string()))
            .bind(("timestamp", timestamp))
            .await
            .map_err(|e| format!("Failed to update last_message_at: {}", e))?;

        Ok(())
    }
}
