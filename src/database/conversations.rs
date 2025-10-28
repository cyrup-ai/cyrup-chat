//! Conversation database operations
//!
//! Provides CRUD operations for conversation table and complex queries
//! joining with message table for summaries and unread counts.
//!
//! Aligns with src/database/schema.rs conversation table (lines 40-52)

use super::Database;
use crate::view_model::agent::AgentTemplateId;
use crate::view_model::conversation::{Conversation, ConversationId, ConversationSummary};
use crate::view_model::message::MessageId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;

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
    /// Inserts into conversation table with agent_session_id = None (lazy spawn pattern).
    /// SurrealDB auto-generates ID and sets created_at to current time.
    ///
    /// # Lazy Spawn Pattern
    /// New conversations start with agent_session_id = None.
    /// Call update_conversation_session() after spawning agent on first message.
    pub async fn create_conversation(&self, conversation: &Conversation) -> Result<String, String> {
        // Serialize conversation fields for database insertion
        #[derive(Serialize, SurrealValue)]
        struct ConversationInsert {
            title: String,
            template_id: String,
            summary: String,
            agent_session_id: Option<String>,
            last_message_at: DateTime<Utc>,
        }

        let insert_data = ConversationInsert {
            title: conversation.title.clone(),
            template_id: conversation.template_id.0.clone(),
            summary: conversation.summary.clone(),
            agent_session_id: conversation.agent_session_id.clone(),
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
            .map(|c| c.id.0)
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
            id: String,
            title: String,
            template_id: String,
            summary: String,
            agent_session_id: Option<String>,
            last_summarized_message_id: Option<String>,
            last_message_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
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
            template_id: AgentTemplateId(record.template_id),
            summary: record.summary,
            agent_session_id: record.agent_session_id,
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
    /// Graph query using SurrealDB's native traversal operators:
    /// - Uses `<-conversation_id<-message` to traverse from conversation to its messages
    /// - Filters and orders directly in traversal (no subqueries needed)
    /// - Orders by last_message_at DESC (newest first)
    pub async fn list_conversations(&self) -> Result<Vec<ConversationSummary>, String> {
        // Native SurrealDB graph traversal using record link
        // `<-conversation_id<-message` means: traverse backwards from conversation via conversation_id field to message records
        let query = r"
            SELECT 
                id,
                title,
                (<-conversation_id<-message WHERE deleted = false ORDER BY timestamp DESC LIMIT 1)[0].content AS last_message_preview,
                last_message_at AS last_message_timestamp,
                count(<-conversation_id<-message WHERE unread = true AND deleted = false) AS unread_count
            FROM conversation
            ORDER BY last_message_at DESC
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
            id: String,
            title: String,
            last_message_preview: Option<String>,
            last_message_timestamp: DateTime<Utc>,
            unread_count: u32,
        }

        let results: Vec<QueryResult> = response
            .take(0)
            .map_err(|e| format!("Failed to parse conversations: {}", e))?;

        // Map query results to ConversationSummary view models
        Ok(results
            .into_iter()
            .map(|r| ConversationSummary {
                id: ConversationId(r.id),
                title: r.title,
                last_message_preview: r
                    .last_message_preview
                    .unwrap_or_else(|| String::from("No messages yet")),
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
            WHERE id = $id
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

    /// Update conversation with spawned agent session ID
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID
    /// * `session_id` - Spawned agent session ID from MCP server
    ///
    /// # Returns
    /// * `Ok(())` - Update succeeded
    /// * `Err(String)` - Error if update fails
    ///
    /// # Database Operation
    /// UPDATE conversation SET agent_session_id = $session WHERE id = $id
    ///
    /// # Lazy Spawn Pattern
    /// This method implements the lazy spawn pattern:
    /// 1. Conversation created with agent_session_id = None
    /// 2. User sends first message → spawn agent via MCP
    /// 3. Call this method to store session_id
    /// 4. Future messages reuse this session_id
    pub async fn update_conversation_session(
        &self,
        conversation_id: &str,
        session_id: &str,
    ) -> Result<(), String> {
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            UPDATE conversation 
            SET agent_session_id = $session 
            WHERE id = $id
        ";

        self.client()
            .query(query)
            .bind(("id", conversation_id.to_string()))
            .bind(("session", session_id.to_string()))
            .await
            .map_err(|e| format!("Failed to update session: {}", e))?;

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
        timestamp: DateTime<Utc>,
    ) -> Result<(), String> {
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            UPDATE conversation 
            SET last_message_at = $timestamp 
            WHERE id = $id
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
