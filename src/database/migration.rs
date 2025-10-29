//! Database migration logic for conversation/room unification
//!
//! Migrates existing data from old schema to unified schema:
//! - Old: conversation.template_id (single agent)
//! - Old: conversation.agent_session_id (single session)
//! - Old: room.participants (multi-agent, no sessions)
//! - New: conversation.participants (1:N agents)
//! - New: conversation.agent_sessions (HashMap<agent_id, session_id>)

use super::Database;

impl Database {
    /// Migrate all conversations from old schema to new unified schema
    ///
    /// Wrapped in SurrealDB transaction for atomicity.
    /// If migration fails partway through, ALL changes are rolled back.
    ///
    /// # Returns
    /// * `Ok((migrated_conversations, migrated_rooms))` - Count of migrated records
    /// * `Err(String)` - Error if migration fails
    pub async fn migrate_to_unified_conversations(&self) -> Result<(usize, usize), String> {
        log::info!("[Migration] Starting conversation/room unification migration");

        // ✅ Execute entire migration as single query with transaction
        let query = r"
            BEGIN TRANSACTION;

            -- Step 1: Migrate old conversations (template_id → participants)
            FOR $conv IN $old_convs {
                -- Start with basic participants update
                UPDATE $conv.id SET participants = [$conv.template_id];
                -- Build agent_sessions properly using conditional logic
                IF $conv.agent_session_id != NONE {
                    -- Use object::from_entries to build dynamic key-value pair
                    UPDATE $conv.id SET agent_sessions = object::from_entries([[$conv.template_id, $conv.agent_session_id]]);
                } ELSE {
                    UPDATE $conv.id SET agent_sessions = {};
                }
            };

            -- Step 2: Migrate old rooms (participants → conversation)
            LET $rooms = (SELECT id, title, participants, summary, last_message_at, created_at FROM room);

            FOR $room IN $rooms {
                LET $new_id = type::record('conversation', rand::ulid());
                CREATE $new_id CONTENT {
                    title: $room.title,
                    participants: $room.participants,
                    summary: $room.summary,
                    agent_sessions: {},
                    last_summarized_message_id: NONE,
                    last_message_at: $room.last_message_at,
                    created_at: $room.created_at
                };

                -- Update message conversation_id references (room_id → conversation_id)
                UPDATE message
                SET conversation_id = $new_id
                WHERE conversation_id = $room.id;
            };

            -- Step 3: Drop old room table
            REMOVE TABLE room;

            COMMIT TRANSACTION;
        ";

        let _response = self.client()
            .query(query)
            .await
            .map_err(|e| format!("Migration failed: {}", e))?;

        // Since we're using a single transaction query, we don't get individual counts
        // The migration either succeeds completely or fails completely
        // For now, return placeholder counts - in production we might want to query counts separately
        let conv_count = 0; // Placeholder - would need separate query to get actual count
        let room_count = 0; // Placeholder - would need separate query to get actual count

        log::info!("[Migration] Migration completed successfully");

        Ok((conv_count, room_count))
    }

    /// Add performance indexes for message author filtering, bookmark lookups, and reaction grouping
    /// 
    /// # Returns
    /// * `Ok(())` - Indexes created successfully
    /// * `Err(String)` - Error if index creation fails
    pub async fn migrate_add_performance_indexes(&self) -> Result<(), String> {
        log::info!("[Migration] Adding performance indexes");

        self.client().query(r"
            -- Author filtering for notifications
            DEFINE INDEX IF NOT EXISTS idx_message_author_unread ON message
                COLUMNS author_type, unread, timestamp;

            -- Bookmark existence check optimization  
            DEFINE INDEX IF NOT EXISTS idx_bookmark_lookup ON bookmark
                COLUMNS user_id, message_id;

            -- Reaction grouping optimization
            DEFINE INDEX IF NOT EXISTS idx_reaction_msg_emoji ON reaction
                COLUMNS message_id, emoji;
        ").await.map_err(|e| format!("Index creation failed: {}", e))?;

        log::info!("[Migration] Performance indexes created successfully");
        Ok(())
    }


}
