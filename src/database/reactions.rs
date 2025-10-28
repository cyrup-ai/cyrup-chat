//! Reaction database operations
//!
//! Aligns with src/database/schema.rs reaction table (lines 101-108)
//! Design: Q31 from MASTODON_ROSETTA_STONE.md - emoji feedback

use super::Database;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;

/// Reaction record from database
///
/// Not a view model - used only for database operations
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct Reaction {
    pub id: String,
    pub message_id: String,
    pub user_id: String,
    pub emoji: String,
    pub created_at: DateTime<Utc>,
}

impl Database {
    /// Add a reaction to a message
    ///
    /// # Arguments
    /// * `message_id` - Message record ID to react to
    /// * `user_id` - User identifier
    /// * `emoji` - Emoji string (e.g., "👍", "❤️", "🎯")
    ///
    /// # Returns
    /// * `Ok(String)` - Reaction ID (existing or newly created)
    /// * `Err(String)` - Error message if operation fails
    ///
    /// # Design Note
    /// Prevents duplicate user+emoji combinations (idempotent operation)
    pub async fn add_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<String, String> {
        // Step 1: Check for existing reaction with same user+emoji
        // Convert &str to String to satisfy 'static lifetime for async
        let check_query = r"
            SELECT id 
            FROM type::thing('message', $message)<-message_id<-reaction
            WHERE user_id = $user AND emoji = $emoji
            LIMIT 1
        ";

        let mut check_response = self
            .client()
            .query(check_query)
            .bind(("message", message_id.to_string()))
            .bind(("user", user_id.to_string()))
            .bind(("emoji", emoji.to_string()))
            .await
            .map_err(|e| format!("Failed to check existing reaction: {}", e))?;

        #[derive(Deserialize, SurrealValue)]
        struct ReactionId {
            id: String,
        }

        let existing: Vec<ReactionId> = check_response
            .take(0)
            .map_err(|e| format!("Failed to parse reaction check: {}", e))?;

        // If duplicate found, return existing ID
        if let Some(existing_reaction) = existing.first() {
            return Ok(existing_reaction.id.clone());
        }

        // Step 2: No duplicate - create new reaction
        #[derive(Serialize, SurrealValue)]
        struct ReactionInsert {
            message_id: String,
            user_id: String,
            emoji: String,
        }

        let result: Option<Reaction> = self
            .client()
            .create("reaction")
            .content(ReactionInsert {
                message_id: message_id.to_string(),
                user_id: user_id.to_string(),
                emoji: emoji.to_string(),
            })
            .await
            .map_err(|e| format!("Failed to add reaction: {}", e))?;

        result
            .map(|r| r.id)
            .ok_or_else(|| "Create returned empty result".to_string())
    }

    /// Remove a reaction by ID
    ///
    /// # Arguments
    /// * `reaction_id` - Reaction record ID to remove
    ///
    /// # Returns
    /// * `Ok(())` - Reaction removed successfully
    /// * `Err(String)` - Error message if removal fails
    pub async fn remove_reaction_by_id(&self, reaction_id: &str) -> Result<(), String> {
        self.client()
            .delete::<Option<Reaction>>(("reaction", reaction_id))
            .await
            .map_err(|e| format!("Failed to remove reaction: {}", e))?;

        Ok(())
    }

    /// Remove a user's specific emoji reaction from a message
    ///
    /// # Arguments
    /// * `message_id` - Message record ID
    /// * `user_id` - User identifier
    /// * `emoji` - Emoji to remove
    ///
    /// # Returns
    /// * `Ok(())` - Reaction removed (or didn't exist)
    /// * `Err(String)` - Error message if query fails
    pub async fn remove_reaction(
        &self,
        message_id: &str,
        user_id: &str,
        emoji: &str,
    ) -> Result<(), String> {
        let query = r"
            DELETE reaction 
            WHERE message_id = $message 
              AND user_id = $user 
              AND emoji = $emoji
        ";

        self.client()
            .query(query)
            .bind(("message", message_id.to_string()))
            .bind(("user", user_id.to_string()))
            .bind(("emoji", emoji.to_string()))
            .await
            .map_err(|e| format!("Failed to remove reaction: {}", e))?;

        Ok(())
    }

    /// Get all reactions for a message
    ///
    /// # Arguments
    /// * `message_id` - Message record ID
    ///
    /// # Returns
    /// * `Ok(Vec<Reaction>)` - List of reactions (chronological order)
    /// * `Err(String)` - Error message if query fails
    pub async fn get_message_reactions(&self, message_id: &str) -> Result<Vec<Reaction>, String> {
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            SELECT <-message_id<-reaction.*
            FROM type::thing('message', $message)
            ORDER BY created_at ASC
        ";

        let mut response = self
            .client()
            .query(query)
            .bind(("message", message_id.to_string()))
            .await
            .map_err(|e| format!("Failed to get reactions: {}", e))?;

        let reactions: Vec<Reaction> = response
            .take(0)
            .map_err(|e| format!("Failed to parse reactions: {}", e))?;

        Ok(reactions)
    }

    /// Get reaction counts grouped by emoji
    ///
    /// # Arguments
    /// * `message_id` - Message record ID
    ///
    /// # Returns
    /// * `Ok(Vec<(String, u32)>)` - List of (emoji, count) tuples
    /// * `Err(String)` - Error message if query fails
    ///
    /// # Design Note
    /// Returns format for UI display: [("👍", 5), ("❤️", 3)]
    pub async fn get_reaction_counts(
        &self,
        message_id: &str,
    ) -> Result<Vec<(String, u32)>, String> {
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            SELECT emoji, count() AS count
            FROM type::thing('message', $message)<-message_id<-reaction
            GROUP BY emoji
            ORDER BY count DESC
        ";

        let mut response = self
            .client()
            .query(query)
            .bind(("message", message_id.to_string()))
            .await
            .map_err(|e| format!("Failed to get reaction counts: {}", e))?;

        #[derive(Deserialize, SurrealValue)]
        struct ReactionCount {
            emoji: String,
            count: u32,
        }

        let results: Vec<ReactionCount> = response
            .take(0)
            .map_err(|e| format!("Failed to parse reaction counts: {}", e))?;

        Ok(results.into_iter().map(|r| (r.emoji, r.count)).collect())
    }
}
