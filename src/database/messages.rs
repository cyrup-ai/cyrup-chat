//! Message database operations
//!
//! Provides CRUD operations for message table including insert, retrieve,
//! search, mark read, delete, and pin operations.
//!
//! Aligns with src/database/schema.rs message table (lines 57-74)

use super::Database;
use crate::view_model::message::{AuthorType, Message, MessageType};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::types::RecordId;
use surrealdb_types::{SurrealValue, ToSql};

impl Database {
    /// Insert a new message and update conversation timestamp
    ///
    /// # Arguments
    /// * `message` - Message to insert (id field ignored, DB generates ID)
    ///
    /// # Returns
    /// * `Ok(RecordId)` - Database-generated message ID
    /// * `Err(String)` - Error message if insertion fails
    ///
    /// # Database Operations
    /// 1. Inserts message into message table
    /// 2. Updates conversation.last_message_at to message timestamp
    ///
    /// # Design Note
    /// Both operations must succeed together for conversation sorting accuracy.
    pub async fn insert_message(&self, message: &Message) -> Result<RecordId, String> {
        // Serialize message fields for database insertion
        #[derive(Serialize, SurrealValue)]
        struct MessageInsert {
            conversation_id: RecordId,
            author: String,
            author_type: AuthorType,
            content: String,
            timestamp: DateTime<Utc>,
            in_reply_to: Option<RecordId>,
            message_type: MessageType,
            attachments: Vec<String>,
            unread: bool,
            deleted: bool,
            pinned: bool,
        }

        let insert_data = MessageInsert {
            conversation_id: message.conversation_id.clone(),
            author: message.author.clone(),
            author_type: message.author_type,
            content: message.content.clone(),
            timestamp: *message.timestamp,
            in_reply_to: message.in_reply_to.clone(),
            message_type: message.message_type,
            attachments: message.attachments.clone(),
            unread: message.unread,
            deleted: message.deleted,
            pinned: message.pinned,
        };

        // Insert message into database
        let result: Option<Message> = self
            .client()
            .create("message")
            .content(insert_data)
            .await
            .map_err(|e| format!("Failed to insert message: {}", e))?;

        let message_id = result
            .as_ref()
            .map(|m| m.id.clone())
            .ok_or_else(|| "Insert returned empty result".to_string())?;

        // Update conversation last_message_at timestamp
        let update_query = r"
            UPDATE conversation 
            SET last_message_at = $timestamp 
            WHERE id = $conversation_id
        ";

        self.client()
            .query(update_query)
            .bind(("conversation_id", message.conversation_id.clone()))
            .bind(("timestamp", message.timestamp))
            .await
            .map_err(|e| format!("Failed to update conversation timestamp: {}", e))?;

        Ok(message_id)
    }

    /// Get recent messages for agent context window
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID to retrieve messages from
    ///
    /// # Returns
    /// * `Ok(Vec<Message>)` - Messages in chronological order (oldest first)
    /// * `Err(String)` - Error message if retrieval fails
    ///
    /// # Design Note
    /// Returns oldest-first ordering for agent context window.
    /// Filters out soft-deleted messages (deleted=false).
    /// Uses token-aware dynamic limit based on configured token budget.
    pub async fn get_recent_messages(&self, conversation_id: &RecordId) -> Result<Vec<Message>, String> {
        // Calculate dynamic limit based on token budget
        let message_limit = self.token_budget_config.calculate_message_limit();

        // Build query with dynamic limit
        let query = format!(
            r#"
            SELECT *
            FROM message
            WHERE conversation_id = $conversation_id
              AND deleted = false
            ORDER BY timestamp ASC
            LIMIT {}
            "#,
            message_limit
        );

        let mut response = self
            .client()
            .query(&query)
            .bind(("conversation_id", conversation_id.clone()))
            .await
            .map_err(|e| format!("Failed to get recent messages: {}", e))?;

        let messages: Vec<Message> = response
            .take(0)
            .map_err(|e| format!("Failed to parse messages: {}", e))?;

        Ok(messages)
    }

    /// Get all messages in conversation history
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID to retrieve messages from
    ///
    /// # Returns
    /// * `Ok(Vec<Message>)` - All messages in chronological order (oldest first)
    /// * `Err(String)` - Error message if retrieval fails
    ///
    /// # Design Note
    /// Returns oldest-first ordering for chat display.
    /// Filters out soft-deleted messages (deleted=false).
    pub async fn get_all_messages(&self, conversation_id: &RecordId) -> Result<Vec<Message>, String> {
        let query = r"
            SELECT *
            FROM message
            WHERE conversation_id = $conversation_id
              AND deleted = false
            ORDER BY timestamp ASC
        ";

        let mut response = self
            .client()
            .query(query)
            .bind(("conversation_id", conversation_id.clone()))
            .await
            .map_err(|e| format!("Failed to get all messages: {}", e))?;

        let messages: Vec<Message> = response
            .take(0)
            .map_err(|e| format!("Failed to parse messages: {}", e))?;

        Ok(messages)
    }

    /// Search messages by content within a conversation
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID to search within
    /// * `search_term` - Text to search for in message content
    ///
    /// # Returns
    /// * `Ok(Vec<Message>)` - Messages containing search term, chronological order
    /// * `Err(String)` - Error message if search fails
    ///
    /// # Design Note
    /// Uses SurrealDB CONTAINS operator for full-text search.
    /// Filters out soft-deleted messages (deleted=false).
    pub async fn search_messages(
        &self,
        conversation_id: &RecordId,
        search_term: &str,
    ) -> Result<Vec<Message>, String> {
        let query = r"
            SELECT *
            FROM message
            WHERE conversation_id = $conversation_id
              AND content CONTAINS $search_term
              AND deleted = false
            ORDER BY timestamp ASC
        ";

        let mut response = self
            .client()
            .query(query)
            .bind(("conversation_id", conversation_id.clone()))
            .bind(("search_term", search_term.to_string()))
            .await
            .map_err(|e| format!("Failed to search messages: {}", e))?;

        let messages: Vec<Message> = response
            .take(0)
            .map_err(|e| format!("Failed to parse search results: {}", e))?;

        Ok(messages)
    }

    /// Mark all messages in a conversation as read
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID to mark messages as read
    ///
    /// # Returns
    /// * `Ok(())` - All messages marked read successfully
    /// * `Err(String)` - Error message if update fails
    ///
    /// # Design Note (Q30)
    /// Called when user opens conversation. Sets unread=false for all messages.
    /// Used for notification badge clearing.
    pub async fn mark_messages_read(&self, conversation_id: &RecordId) -> Result<(), String> {
        let query = r"
            UPDATE message
            SET unread = false
            WHERE conversation_id = $conversation_id
        ";

        self.client()
            .query(query)
            .bind(("conversation_id", conversation_id.clone()))
            .await
            .map_err(|e| format!("Failed to mark messages read: {}", e))?;

        Ok(())
    }

    /// Get count of unread messages in a conversation
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID to count unread messages
    ///
    /// # Returns
    /// * `Ok(u32)` - Number of unread messages
    /// * `Err(String)` - Error message if count fails
    ///
    /// # Design Note (Q30)
    /// Used for notification badge counts. Excludes soft-deleted messages.
    pub async fn get_unread_count(&self, conversation_id: &RecordId) -> Result<u32, String> {
        let query = r"
            SELECT count() AS count
            FROM message
            WHERE conversation_id = $conversation_id
              AND unread = true
              AND deleted = false
        ";

        let mut response = self
            .client()
            .query(query)
            .bind(("conversation_id", conversation_id.clone()))
            .await
            .map_err(|e| format!("Failed to get unread count: {}", e))?;

        #[derive(Deserialize, SurrealValue)]
        struct CountResult {
            count: u32,
        }

        let results: Vec<CountResult> = response
            .take(0)
            .map_err(|e| format!("Failed to parse unread count: {}", e))?;

        Ok(results.first().map(|r| r.count).unwrap_or(0))
    }

    /// Soft delete a message
    ///
    /// # Arguments
    /// * `message_id` - Message ID to delete
    ///
    /// # Returns
    /// * `Ok(())` - Message deleted successfully
    /// * `Err(String)` - Error message if deletion fails
    ///
    /// # Design Note (Q35)
    /// Sets deleted=true, does NOT remove from database.
    /// Preserves message for agent context but hides from UI.
    pub async fn delete_message(&self, message_id: &RecordId) -> Result<(), String> {
        let query = r"
            UPDATE message
            SET deleted = true
            WHERE id = $message_id
        ";

        self.client()
            .query(query)
            .bind(("message_id", message_id.clone()))
            .await
            .map_err(|e| format!("Failed to delete message: {}", e))?;

        Ok(())
    }

    /// Toggle pin status for a message
    ///
    /// # Arguments
    /// * `message_id` - Message ID to toggle pin
    /// * `conversation_id` - Conversation ID (for pin limit check)
    ///
    /// # Returns
    /// * `Ok(bool)` - New pin state (true=pinned, false=unpinned)
    /// * `Err(String)` - Error message if toggle fails or pin limit exceeded
    ///
    /// # Design Note (Q37)
    /// Enforces maximum 5 pinned messages per conversation.
    /// Checks current pin count before allowing new pin.
    pub async fn toggle_pin_message(
        &self,
        message_id: &RecordId,
        conversation_id: &RecordId,
    ) -> Result<bool, String> {
        // Get current message state
        let current_query = r"
            SELECT pinned
            FROM message
            WHERE id = $message_id
        ";

        let mut current_response = self
            .client()
            .query(current_query)
            .bind(("message_id", message_id.clone()))
            .await
            .map_err(|e| format!("Failed to get message state: {}", e))?;

        #[derive(Deserialize, SurrealValue)]
        struct MessageState {
            pinned: bool,
        }

        let current_state: Vec<MessageState> = current_response
            .take(0)
            .map_err(|e| format!("Failed to parse message state: {}", e))?;

        let currently_pinned = current_state
            .first()
            .map(|s| s.pinned)
            .ok_or_else(|| "Message not found".to_string())?;

        // If trying to pin (currently unpinned), check pin limit
        if !currently_pinned {
            let count_query = r"
                SELECT count() AS count
                FROM message
                WHERE conversation_id = $conversation_id
                  AND pinned = true
            ";

            let mut count_response = self
                .client()
                .query(count_query)
                .bind(("conversation_id", conversation_id.clone()))
                .await
                .map_err(|e| format!("Failed to check pin count: {}", e))?;

            #[derive(Deserialize, SurrealValue)]
            struct CountResult {
                count: u32,
            }

            let pin_counts: Vec<CountResult> = count_response
                .take(0)
                .map_err(|e| format!("Failed to parse pin count: {}", e))?;

            let current_pin_count = pin_counts.first().map(|r| r.count).unwrap_or(0);

            if current_pin_count >= 5 {
                return Err("Maximum 5 pinned messages per conversation".to_string());
            }
        }

        // Toggle pin state
        let new_state = !currently_pinned;
        let update_query = r"
            UPDATE message
            SET pinned = $pinned
            WHERE id = $message_id
        ";

        self.client()
            .query(update_query)
            .bind(("message_id", message_id.clone()))
            .bind(("pinned", new_state))
            .await
            .map_err(|e| format!("Failed to toggle pin: {}", e))?;

        Ok(new_state)
    }

    /// Get all pinned messages in a conversation
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID to get pinned messages from
    ///
    /// # Returns
    /// * `Ok(Vec<Message>)` - Pinned messages in chronological order
    /// * `Err(String)` - Error message if retrieval fails
    ///
    /// # Design Note
    /// Used for displaying pinned messages in conversation header.
    /// Maximum 5 pinned messages per conversation (enforced by toggle_pin_message).
    pub async fn get_pinned_messages(&self, conversation_id: &RecordId) -> Result<Vec<Message>, String> {
        let query = r"
            SELECT *
            FROM message
            WHERE conversation_id = $conversation_id
              AND pinned = true
              AND deleted = false
            ORDER BY timestamp ASC
        ";

        let mut response = self
            .client()
            .query(query)
            .bind(("conversation_id", conversation_id.clone()))
            .await
            .map_err(|e| format!("Failed to get pinned messages: {}", e))?;

        let messages: Vec<Message> = response
            .take(0)
            .map_err(|e| format!("Failed to parse pinned messages: {}", e))?;

        Ok(messages)
    }

    /// Update message content (for streaming responses)
    ///
    /// # Arguments
    /// * `message_id` - Message ID to update
    /// * `new_content` - New content text
    ///
    /// # Returns
    /// * `Ok(())` - Update succeeded
    /// * `Err(String)` - Error if update fails
    ///
    /// # Design Note
    /// Used for streaming agent responses. LIVE QUERY subscribers
    /// receive Action::Update notification automatically.
    /// Pattern: INSERT first chunk, UPDATE subsequent chunks.
    pub async fn update_message_content(
        &self,
        message_id: &RecordId,
        new_content: String,
    ) -> Result<(), String> {
        let query = r"
            UPDATE message
            SET content = $content
            WHERE id = $message_id
        ";

        self.client()
            .query(query)
            .bind(("message_id", message_id.clone()))
            .bind(("content", new_content))
            .await
            .map_err(|e| format!("Failed to update message content: {}", e))?;

        Ok(())
    }

    /// Get single message by ID
    ///
    /// # Arguments
    /// * `message_id` - Message ID
    ///
    /// # Returns
    /// * `Ok(Message)` - Message details
    /// * `Err(String)` - Not found or query failed
    pub async fn get_message(&self, message_id: &RecordId) -> Result<Message, String> {
        let query = r"
            SELECT *
            FROM message
            WHERE id = $message_id
            LIMIT 1
        ";

        let mut response = self
            .client()
            .query(query)
            .bind(("message_id", message_id.clone()))
            .await
            .map_err(|e| format!("Failed to get message: {}", e))?;

        let mut messages: Vec<Message> = response
            .take(0)
            .map_err(|e| format!("Failed to parse message: {}", e))?;

        messages
            .pop()
            .ok_or_else(|| "Message not found".to_string())
    }

    /// Pin a message (sets pinned=true)
    ///
    /// # Arguments
    /// * `message_id` - Message ID to pin
    ///
    /// # Returns
    /// * `Ok(())` - Message pinned successfully
    /// * `Err(String)` - Pin limit exceeded or update failed
    pub async fn pin_message(&self, message_id: &RecordId) -> Result<(), String> {
        // Get message to find conversation_id
        let message = self.get_message(message_id).await?;

        // Use toggle if not already pinned
        if !message.pinned {
            self.toggle_pin_message(message_id, &message.conversation_id)
                .await?;
        }

        Ok(())
    }

    /// Unpin a message (sets pinned=false)
    ///
    /// # Arguments
    /// * `message_id` - Message ID to unpin
    ///
    /// # Returns
    /// * `Ok(())` - Message unpinned successfully
    /// * `Err(String)` - Update failed
    pub async fn unpin_message(&self, message_id: &RecordId) -> Result<(), String> {
        let query = r"
            UPDATE message
            SET pinned = false
            WHERE id = $message_id
        ";

        self.client()
            .query(query)
            .bind(("message_id", message_id.clone()))
            .await
            .map_err(|e| format!("Failed to unpin message: {}", e))?;

        Ok(())
    }

    /// Get all unread messages for a user across all conversations
    ///
    /// # Arguments
    /// * `user_id` - User ID (unused for MVP, all messages returned)
    ///
    /// # Returns
    /// * `Ok(Vec<Message>)` - Unread messages (newest first)
    /// * `Err(String)` - Query failed
    ///
    /// # Design Note (Q28-Q30)
    /// Used for notifications - shows agent responses user hasn't seen.
    /// Filters for agent messages only (author_type="agent").
    pub async fn get_unread_messages(&self, _user_id: &str) -> Result<Vec<Message>, String> {
        let query = r#"
            SELECT *
            FROM message
            WHERE unread = true
              AND deleted = false
              AND author_type = "agent"
            ORDER BY timestamp DESC
            LIMIT 50
        "#;

        let mut response = self
            .client()
            .query(query)
            .await
            .map_err(|e| format!("Failed to get unread messages: {}", e))?;

        let messages: Vec<Message> = response
            .take(0)
            .map_err(|e| format!("Failed to parse unread messages: {}", e))?;

        Ok(messages)
    }

    /// Mark all messages in a conversation as read
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID to mark messages as read
    ///
    /// # Returns
    /// * `Ok(())` - Messages marked as read successfully
    /// * `Err(String)` - Error if update fails
    ///
    /// # Database Operation
    /// Updates all messages where conversation_id matches and unread=true,
    /// setting unread=false. This clears unread count for the conversation.
    pub async fn mark_messages_as_read(&self, conversation_id: &RecordId) -> Result<(), String> {
        let query = "UPDATE message SET unread = false WHERE conversation_id = $conversation_id AND unread = true";

        self.client()
            .query(query)
            .bind(("conversation_id", conversation_id.clone()))
            .await
            .map_err(|e| format!("Failed to mark messages as read: {}", e))?;

        log::debug!(
            "[Database] Marked messages as read for conversation: {}",
            conversation_id.to_sql()
        );

        Ok(())
    }
}
