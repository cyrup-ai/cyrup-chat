//! Bookmark database operations
//!
//! Aligns with src/database/schema.rs bookmark table (lines 91-98)
//! Design: Q26-Q27 from MASTODON_ROSETTA_STONE.md - global bookmarks

use super::Database;
use crate::view_model::message::Message;
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;

impl Database {
    /// Bookmark a message for a user
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `message_id` - Message record ID to bookmark
    ///
    /// # Returns
    /// * `Ok(String)` - Database-generated bookmark ID
    /// * `Err(String)` - Error message if bookmark fails
    ///
    /// # Design Note
    /// Global bookmarks - not per-conversation (Q26)
    pub async fn bookmark_message(
        &self,
        user_id: &str,
        message_id: &str,
    ) -> Result<String, String> {
        #[derive(Serialize, SurrealValue)]
        struct BookmarkInsert {
            user_id: String,
            message_id: String,
        }

        let insert_data = BookmarkInsert {
            user_id: user_id.to_string(),
            message_id: message_id.to_string(),
        };

        // Define Bookmark for deserialize
        #[derive(Deserialize, SurrealValue)]
        struct Bookmark {
            id: String,
        }

        let result: Option<Bookmark> = self
            .client()
            .create("bookmark")
            .content(insert_data)
            .await
            .map_err(|e| format!("Failed to bookmark message: {}", e))?;

        result
            .map(|b| b.id)
            .ok_or_else(|| "Create returned empty result".to_string())
    }

    /// Remove a bookmark
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `message_id` - Message record ID to unbookmark
    ///
    /// # Returns
    /// * `Ok(())` - Bookmark removed successfully
    /// * `Err(String)` - Error message if removal fails
    pub async fn unbookmark_message(&self, user_id: &str, message_id: &str) -> Result<(), String> {
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            DELETE FROM bookmark 
            WHERE user_id = $user AND message_id = $message
        ";

        self.client()
            .query(query)
            .bind(("user", user_id.to_string()))
            .bind(("message", message_id.to_string()))
            .await
            .map_err(|e| format!("Failed to unbookmark: {}", e))?;

        Ok(())
    }

    /// Get all bookmarked messages for a user
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    ///
    /// # Returns
    /// * `Ok(Vec<Message>)` - List of bookmarked messages (newest first)
    /// * `Err(String)` - Error message if query fails
    ///
    /// # Database Operation
    /// Uses graph traversal to fetch full Message objects, not just IDs
    pub async fn get_bookmarked_messages(&self, user_id: &str) -> Result<Vec<Message>, String> {
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            SELECT ->message_id->message.*
            FROM bookmark
            WHERE user_id = $user
            ORDER BY created_at DESC
        ";

        let mut response = self
            .client()
            .query(query)
            .bind(("user", user_id.to_string()))
            .await
            .map_err(|e| format!("Failed to get bookmarks: {}", e))?;

        let messages: Vec<Message> = response
            .take(0)
            .map_err(|e| format!("Failed to parse bookmarks: {}", e))?;

        Ok(messages)
    }

    /// Check if a message is bookmarked by a user
    ///
    /// # Arguments
    /// * `user_id` - User identifier
    /// * `message_id` - Message record ID to check
    ///
    /// # Returns
    /// * `Ok(bool)` - True if bookmarked, false otherwise
    /// * `Err(String)` - Error message if check fails
    ///
    /// # Design Note
    /// Used for UI toggle state
    pub async fn is_bookmarked(&self, user_id: &str, message_id: &str) -> Result<bool, String> {
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            SELECT id FROM bookmark 
            WHERE user_id = $user AND message_id = $message
            LIMIT 1
        ";

        let mut response = self
            .client()
            .query(query)
            .bind(("user", user_id.to_string()))
            .bind(("message", message_id.to_string()))
            .await
            .map_err(|e| format!("Failed to check bookmark: {}", e))?;

        #[derive(Deserialize, SurrealValue)]
        #[allow(dead_code)] // Field used by serde deserializer
        struct BookmarkId {
            id: String,
        }

        let results: Vec<BookmarkId> = response
            .take(0)
            .map_err(|e| format!("Failed to parse bookmark check: {}", e))?;

        Ok(!results.is_empty())
    }

    /// Alias for bookmark_message (matches Model API naming)
    pub async fn add_bookmark(&self, user_id: &str, message_id: &str) -> Result<String, String> {
        self.bookmark_message(user_id, message_id).await
    }

    /// Alias for unbookmark_message (matches Model API naming)
    pub async fn remove_bookmark(&self, user_id: &str, message_id: &str) -> Result<(), String> {
        self.unbookmark_message(user_id, message_id).await
    }
}
