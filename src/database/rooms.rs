//! Room database operations (multi-agent conversations)
//!
//! Aligns with src/database/schema.rs room table (lines 79-88)

use super::Database;
use crate::view_model::agent::AgentTemplateId;
use crate::view_model::conversation::{Room, RoomId, RoomSummary};
use crate::view_model::message::MessageId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb_types::SurrealValue;

impl Database {
    /// Create a new room in the database
    ///
    /// # Arguments
    /// * `room` - Room to create (id field ignored, DB generates ID)
    ///
    /// # Returns
    /// * `Ok(String)` - Database-generated room ID
    /// * `Err(String)` - Error message if creation fails
    ///
    /// # Database Operation
    /// Inserts into room table. SurrealDB auto-generates ID and sets created_at.
    pub async fn create_room(&self, room: &Room) -> Result<String, String> {
        // Serialize room fields for database insertion
        #[derive(Serialize, SurrealValue)]
        struct RoomInsert {
            title: String,
            participants: Vec<String>,
            summary: String,
            last_message_at: DateTime<Utc>,
        }

        let insert_data = RoomInsert {
            title: room.title.clone(),
            // Extract string IDs from Vec<AgentTemplateId>
            participants: room.participants.iter().map(|p| p.0.clone()).collect(),
            summary: room.summary.clone(),
            last_message_at: room.last_message_at,
        };

        // .create() returns Option<T>, not Vec<Thing>
        let result: Option<Room> = self
            .client()
            .create("room")
            .content(insert_data)
            .await
            .map_err(|e| format!("Failed to create room: {}", e))?;

        // Extract ID from created record
        result
            .map(|r| r.id.0)
            .ok_or_else(|| "Create returned empty result".to_string())
    }

    /// Retrieve a single room by ID
    ///
    /// # Arguments
    /// * `id` - Room record ID (e.g., "room:ulid")
    ///
    /// # Returns
    /// * `Ok(Room)` - Found room with all fields
    /// * `Err(String)` - Error if not found or query fails
    pub async fn get_room(&self, id: &str) -> Result<Room, String> {
        // Define response struct matching database schema
        #[derive(Deserialize, SurrealValue)]
        struct RoomRecord {
            id: String,
            title: String,
            participants: Vec<String>,
            summary: String,
            last_summarized_message_id: Option<String>,
            last_message_at: DateTime<Utc>,
            created_at: DateTime<Utc>,
        }

        let record: Option<RoomRecord> = self
            .client()
            .select(("room", id))
            .await
            .map_err(|e| format!("Failed to get room: {}", e))?;

        let record = record.ok_or_else(|| format!("Room not found: {}", id))?;

        // Map database record to Room view model
        Ok(Room {
            id: RoomId(record.id),
            title: record.title,
            // Convert Vec<String> to Vec<AgentTemplateId>
            participants: record
                .participants
                .into_iter()
                .map(AgentTemplateId)
                .collect(),
            summary: record.summary,
            last_summarized_message_id: record.last_summarized_message_id.map(MessageId),
            last_message_at: record.last_message_at,
            created_at: record.created_at,
        })
    }

    /// Add an agent to a room's participants list
    ///
    /// # Arguments
    /// * `room_id` - Room ID to add agent to
    /// * `agent_id` - Agent template ID to add
    ///
    /// # Returns
    /// * `Ok(())` - Agent added successfully
    /// * `Err(String)` - Error if update fails
    ///
    /// # Database Operation
    /// Uses SurrealDB array += operator to append agent ID
    pub async fn add_agent_to_room(&self, room_id: &str, agent_id: &str) -> Result<(), String> {
        // Convert &str to String to satisfy 'static lifetime for async
        let query = r"
            UPDATE room 
            SET participants += [$agent] 
            WHERE id = $room_id
        ";

        self.client()
            .query(query)
            .bind(("room_id", room_id.to_string()))
            .bind(("agent", agent_id.to_string()))
            .await
            .map_err(|e| format!("Failed to add agent to room: {}", e))?;

        Ok(())
    }

    /// List all rooms ordered by most recent activity
    ///
    /// # Returns
    /// * `Ok(Vec<RoomSummary>)` - Rooms sorted by last_message_at DESC
    /// * `Err(String)` - Query execution or parsing error
    ///
    /// # Database Operation
    /// Uses SurrealDB graph traversal to:
    /// 1. Traverse backwards from room via message.conversation_id link
    /// 2. Find last non-deleted message for preview
    /// 3. Order by activity (last_message_at DESC)
    ///
    /// # Pattern Source
    /// Cloned from conversations.rs:123-169 list_conversations() method
    pub async fn list_rooms(&self) -> Result<Vec<RoomSummary>, String> {
        // SurrealDB graph traversal syntax: <-field<-table
        // Reads as: "from room, traverse back through conversation_id field to message records"
        let query = r"
            SELECT 
                id,
                title,
                participants,
                (<-conversation_id<-message WHERE deleted = false ORDER BY timestamp DESC LIMIT 1)[0].content AS last_message_preview,
                last_message_at AS last_message_timestamp
            FROM room
            ORDER BY last_message_at DESC
        ";

        let mut response = self
            .client()
            .query(query)
            .await
            .map_err(|e| format!("Failed to list rooms: {}", e))?;

        // Define response struct matching SELECT fields
        #[derive(Deserialize, SurrealValue)]
        struct QueryResult {
            id: String,
            title: String,
            participants: Vec<String>,  // Array<string> from database
            last_message_preview: Option<String>,
            last_message_timestamp: DateTime<Utc>,
        }

        let results: Vec<QueryResult> = response
            .take(0)
            .map_err(|e| format!("Failed to parse rooms: {}", e))?;

        // Map database records to RoomSummary view models
        Ok(results
            .into_iter()
            .map(|r| RoomSummary {
                id: RoomId(r.id),
                title: r.title,
                // Convert Vec<String> → Vec<AgentTemplateId>
                participants: r.participants.into_iter().map(AgentTemplateId).collect(),
                last_message_preview: r
                    .last_message_preview
                    .unwrap_or_else(|| "No messages yet".to_string()),
                last_message_timestamp: r.last_message_timestamp,
            })
            .collect())
    }
}
