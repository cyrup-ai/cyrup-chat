//! Model client operations for agent conversations

use super::archive_manager::StatusArchiveManager;
use super::types::{Model, ModelError};
use crate::environment::model::Status;
use crate::view_model::conversation::ConversationId;
use crate::view_model::conversation::{Conversation, ConversationSummary};
use crate::view_model::message::{AuthorType, Message, MessageId, MessageType};
use surrealdb_types::ToSql;

use megalodon::entities::StatusVisibility;

impl Model {
    /// List all conversations for home timeline
    ///
    /// Returns conversation summaries sorted by last_message_at descending.
    /// Each summary includes title, preview, timestamp, and unread count.
    ///
    /// # Returns
    /// * `Ok(Vec<ConversationSummary>)` - List of conversations
    /// * `Err(ModelError)` - Database query failed
    ///
    /// # Database Query
    /// Joins conversation and message tables to build summaries with
    /// unread counts and last message preview.
    pub async fn list_conversations(&self) -> Result<Vec<ConversationSummary>, ModelError> {
        self.database()
            .list_conversations()
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to list conversations: {}", e)))
    }

    /// Get single conversation details
    ///
    /// # Arguments
    /// * `id` - Conversation ID (e.g., "conversation:ulid")
    ///
    /// # Returns
    /// * `Ok(Conversation)` - Full conversation details
    /// * `Err(ModelError)` - Not found or query failed
    pub async fn get_conversation(&self, id: &str) -> Result<Conversation, ModelError> {
        self.database()
            .get_conversation(id)
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to get conversation: {}", e)))
    }

    /// Get all messages in conversation
    ///
    /// Returns messages sorted by timestamp ascending (oldest first).
    /// Excludes soft-deleted messages (deleted=true).
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID
    ///
    /// # Returns
    /// * `Ok(Vec<Message>)` - All messages in conversation
    /// * `Err(ModelError)` - Query failed
    pub async fn get_messages(&self, conversation_id: &str) -> Result<Vec<Message>, ModelError> {
        self.database()
            .get_all_messages(conversation_id)
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to get messages: {}", e)))
    }

    /// Send message in conversation with lazy agent spawn
    ///
    /// Implements lazy spawn pattern (Q17):
    /// 1. Check if agent session exists (conversation.agent_session_id)
    /// 2. If None, spawn agent with summary + recent messages context
    /// 3. Send user message to agent
    /// 4. Save user message to database
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation ID
    /// * `message` - User message content
    ///
    /// # Returns
    /// * `Ok(())` - Message sent and saved successfully
    /// * `Err(ModelError)` - Spawn, send, or save failed
    ///
    /// # Design Notes
    /// - User messages have author="David Maple" (Q39 hardcoded for MVP)
    /// - Agent responses retrieved separately via get_agent_messages()
    /// - If spawn fails, entire operation fails (no partial state)
    /// - If send succeeds but save fails, logs error (message still sent)
    pub async fn send_message(
        &self,
        conversation_id: &str,
        message: &str,
    ) -> Result<(), ModelError> {
        // Get conversation to check agent_session_id
        let conversation = self
            .database()
            .get_conversation(conversation_id)
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to get conversation: {}", e)))?;

        // Lazy spawn: check if any agent sessions exist for this conversation
        // For single-agent conversations, spawn if agent_sessions is empty
        if conversation.agent_sessions.is_empty() && !conversation.participants.is_empty() {
            log::info!("Lazy spawning agents for conversation: {}", conversation_id);

            // For single-agent conversations, spawn the one participant
            let agent_id = &conversation.participants[0].0;

            // Get agent template from database
            let template = self
                .database()
                .get_template(agent_id)
                .await
                .map_err(|e| ModelError::QueryFailed(format!("Failed to get template: {}", e)))?;

            // Get recent messages for context (dynamic limit based on token budget)
            let recent_messages = self
                .database()
                .get_recent_messages(conversation_id)
                .await
                .map_err(|e| {
                    ModelError::QueryFailed(format!("Failed to get recent messages: {}", e))
                })?;

            // Spawn agent session
            let session_id = self
                .agent_manager()
                .spawn_agent(
                    conversation_id,
                    &template,
                    &conversation.summary,
                    &recent_messages,
                )
                .await
                .map_err(|e| {
                    ModelError::AgentSpawnFailed(format!("Failed to spawn agent: {}", e))
                })?;

            log::info!("Agent spawned with session_id: {}", session_id);

            // Update conversation with session_id for this agent
            self.database()
                .update_agent_session(conversation_id, agent_id, &session_id)
                .await
                .map_err(|e| ModelError::QueryFailed(format!("Failed to update session: {}", e)))?;
        }

        // Send message to agent session
        self.agent_manager()
            .send_message(conversation_id, message)
            .await
            .map_err(|e| ModelError::AgentSendFailed(format!("Failed to send message: {}", e)))?;

        // Save user message to database
        let user_message = Message {
            id: MessageId(uuid::Uuid::new_v4().to_string()),
            conversation_id: ConversationId(conversation_id.to_string()),
            author: "David Maple".to_string(), // Q39: Hardcoded user for MVP
            author_type: AuthorType::Human,
            content: message.to_string(),
            timestamp: chrono::Utc::now().into(),
            in_reply_to: None,
            message_type: MessageType::Normal,
            attachments: Vec::new(),
            unread: false, // User's own message starts as read
            deleted: false,
            pinned: false,
        };

        self.database()
            .insert_message(&user_message)
            .await
            .map_err(|e| {
                log::error!("Failed to save user message: {}", e);
                ModelError::QueryFailed(format!("Failed to save message: {}", e))
            })?;

        Ok(())
    }

    /// Get single message by ID
    ///
    /// # Arguments
    /// * `message_id` - Message ID
    ///
    /// # Returns
    /// * `Ok(Message)` - Message details
    /// * `Err(ModelError)` - Not found or query failed
    pub async fn single_status(&self, message_id: String) -> Result<Message, ModelError> {
        self.database()
            .get_message(&message_id)
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to get message: {}", e)))
    }

    /// Get conversation context (all messages in conversation)
    ///
    /// # Arguments
    /// * `message_id` - Any message ID in the conversation
    ///
    /// # Returns
    /// * `Ok(Vec<Message>)` - All messages in the conversation
    /// * `Err(ModelError)` - Query failed
    pub async fn status_context(&self, message_id: String) -> Result<Vec<Message>, ModelError> {
        // First get the message to find its conversation_id
        let message = self.single_status(message_id).await?;

        // Then get all messages in that conversation
        self.get_messages(&message.conversation_id.0).await
    }

    /// Add reaction to message (Q12: Keep reactions for ML feedback)
    ///
    /// # Arguments
    /// * `message_id` - Message ID to react to
    /// * `favorited` - true = add 👍 reaction, false = remove 👍 reaction
    ///
    /// # Returns
    /// * `Ok(Status)` - Updated message as Status for UI
    /// * `Err(ModelError)` - Database update failed
    pub async fn set_favourite(
        &self,
        message_id: String,
        favorited: bool,
    ) -> Result<Status, ModelError> {
        let user_id = "hardcoded-david-maple"; // Q39: MVP hardcoded user

        if favorited {
            self.database()
                .add_reaction(&message_id, user_id, "👍")
                .await
                .map_err(|e| ModelError::QueryFailed(format!("Failed to add reaction: {}", e)))?;
        } else {
            self.database()
                .remove_reaction(&message_id, user_id, "👍")
                .await
                .map_err(|e| {
                    ModelError::QueryFailed(format!("Failed to remove reaction: {}", e))
                })?;
        }

        // Fetch updated message and convert to Status for UI
        let message = self
            .database()
            .get_message(&message_id)
            .await
            .map_err(|e| {
                ModelError::QueryFailed(format!("Failed to fetch updated message: {}", e))
            })?;

        Ok(message_to_status(&message, favorited))
    }

    /// Bookmark message for later reference (Q12: Keep bookmarks for UI convenience)
    ///
    /// # Arguments
    /// * `message_id` - Message ID to bookmark
    /// * `bookmarked` - true = add bookmark, false = remove bookmark
    ///
    /// # Returns
    /// * `Ok(Status)` - Updated message as Status for UI
    /// * `Err(ModelError)` - Database update failed
    pub async fn set_bookmark(
        &self,
        message_id: String,
        bookmarked: bool,
    ) -> Result<Status, ModelError> {
        let user_id = "hardcoded-david-maple"; // Q39: MVP hardcoded user

        if bookmarked {
            self.database()
                .add_bookmark(user_id, &message_id)
                .await
                .map_err(|e| ModelError::QueryFailed(format!("Failed to add bookmark: {}", e)))?;
        } else {
            self.database()
                .remove_bookmark(user_id, &message_id)
                .await
                .map_err(|e| {
                    ModelError::QueryFailed(format!("Failed to remove bookmark: {}", e))
                })?;
        }

        // Fetch updated message and convert to Status for UI
        let message = self
            .database()
            .get_message(&message_id)
            .await
            .map_err(|e| {
                ModelError::QueryFailed(format!("Failed to fetch updated message: {}", e))
            })?;

        Ok(message_to_status_with_bookmark(&message, bookmarked))
    }

    /// Soft-delete message from UI (Q34: Delete from UI only, agent context unchanged)
    ///
    /// # Arguments
    /// * `message_id` - Message ID to delete
    ///
    /// # Returns
    /// * `Ok(())` - Message marked as deleted
    /// * `Err(ModelError)` - Database update failed
    pub async fn delete_status(&self, message_id: String) -> Result<(), ModelError> {
        self.database()
            .delete_message(&message_id)
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to delete message: {}", e)))
    }

    /// Pin important message to conversation top (Q37: Allow pinning)
    ///
    /// # Arguments
    /// * `message_id` - Message ID to pin
    ///
    /// # Returns
    /// * `Ok(Status)` - Updated message as Status for UI
    /// * `Err(ModelError)` - Database update failed
    pub async fn pin_status(&self, message_id: String) -> Result<Status, ModelError> {
        self.database()
            .pin_message(&message_id)
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to pin message: {}", e)))?;

        // Fetch updated message and convert to Status for UI
        let message = self
            .database()
            .get_message(&message_id)
            .await
            .map_err(|e| {
                ModelError::QueryFailed(format!("Failed to fetch updated message: {}", e))
            })?;

        Ok(message_to_status_simple(&message))
    }

    /// Unpin message from conversation top
    ///
    /// # Arguments
    /// * `message_id` - Message ID to unpin
    ///
    /// # Returns
    /// * `Ok(Status)` - Updated message as Status for UI
    /// * `Err(ModelError)` - Database update failed
    pub async fn unpin_status(&self, message_id: String) -> Result<Status, ModelError> {
        self.database()
            .unpin_message(&message_id)
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to unpin message: {}", e)))?;

        // Fetch updated message and convert to Status for UI
        let message = self
            .database()
            .get_message(&message_id)
            .await
            .map_err(|e| {
                ModelError::QueryFailed(format!("Failed to fetch updated message: {}", e))
            })?;

        Ok(message_to_status_simple(&message))
    }

    /// Set reblog/boost status (Mastodon social feature - not applicable to chat)
    ///
    /// # Arguments
    /// * `_message_id` - Message ID (unused)
    /// * `_reblogged` - Reblog state (unused)
    ///
    /// # Returns
    /// * `Err(ModelError::NotImplemented)` - Reblog not supported in chat
    pub async fn set_reblog(
        &self,
        _message_id: String,
        _reblogged: bool,
    ) -> Result<Status, ModelError> {
        Err(ModelError::NotImplemented(
            "Reblog/boost is a Mastodon social feature not applicable to chat".to_string(),
        ))
    }

    /// Archive message (cache to time-series database for performance)
    ///
    /// # Arguments
    /// * `message_id` - Message ID to archive
    ///
    /// # Returns
    /// * `Ok(Status)` - Archived message as Status for UI
    /// * `Err(ModelError)` - Archive operation failed
    pub async fn archive_status(&self, message_id: String) -> Result<Status, ModelError> {
        // Get the message to archive
        let message = self
            .database()
            .get_message(&message_id)
            .await
            .map_err(|e| {
                ModelError::QueryFailed(format!("Failed to get message for archiving: {}", e))
            })?;

        // Convert Message to Status for archiving
        let status = message_to_status_simple(&message);

        // Archive using archive manager (triggers compression and storage)
        let _metadata = self
            .archive_manager()
            .archive_status(
                &status,
                crate::environment::native::model::archive_manager::ArchiveReason::Manual,
                "hardcoded-david-maple".to_string(), // MVP: hardcoded user
            )
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to archive status: {}", e)))?;

        // Return the archived status (now persisted with compression)
        Ok(status)
    }

    /// Login method is deprecated - use OAuth flow in LoginView instead
    ///
    /// # Returns
    /// * `Err(ModelError::NotImplemented)` - Auth now happens via OAuth in UI components
    pub async fn login(&self) -> Result<crate::auth::UserInfo, ModelError> {
        Err(ModelError::NotImplemented(
            "Use OAuth flow in LoginView instead".to_string(),
        ))
    }

    /// No-op logout (Q9: No real auth for MVP)
    ///
    /// # Arguments
    /// * `_user_id` - Ignored for hardcoded auth
    ///
    /// # Returns
    /// * Always `Ok(())`
    pub async fn logout(&self, _user_id: String) -> Result<(), ModelError> {
        // No-op for hardcoded auth
        Ok(())
    }

    /// Subscribe to real-time message stream from all active agent sessions
    ///
    /// Spawns background tasks for each active session to monitor for new messages.
    /// Callback is invoked whenever a new message arrives from any agent.
    ///
    /// # Arguments
    /// * `callback` - Function called on each new message with Message data
    ///
    /// # Returns
    /// * `Ok(())` - Subscription listeners spawned successfully
    /// * `Err(ModelError)` - Failed to establish stream
    ///
    /// # Implementation
    /// For each active agent session:
    /// - Spawns background task that polls get_output() every 2 seconds
    /// - Detects new messages by tracking last_offset
    /// - Converts agent output to Message and invokes callback
    /// - Continues until session terminates
    pub async fn subscribe_user_stream<F>(
        &self,
        callback: std::sync::Arc<F>,
    ) -> Result<(), ModelError>
    where
        F: Fn(crate::environment::model::Message) + Send + Sync + 'static,
    {
        log::info!("subscribe_user_stream: Setting up real-time agent message streaming");

        // Get all conversations to find active agent sessions
        let conversations = self.list_conversations().await?;

        log::debug!("Found {} conversations to monitor", conversations.len());

        // Spawn background task for each conversation with an active agent session
        for conv_summary in conversations {
            // Get full conversation details to access agent_session_id
            let conv_id_str = conv_summary.id.0.to_sql();
            let conversation = match self.get_conversation(&conv_id_str).await {
                Ok(conv) => conv,
                Err(e) => {
                    log::warn!("Failed to get conversation {}: {}", conv_id_str, e);
                    continue;
                }
            };

            // Only monitor conversations with active agent sessions
            if conversation.agent_sessions.is_empty() {
                continue;
            }

            let agent_manager = self.agent_manager().clone();
            let callback = callback.clone();
            let conv_id = conversation.id.0.clone();
            let conv_title = conversation.title.clone();

            log::info!(
                "Spawning stream monitor for conversation: {} ({})",
                conv_title,
                conv_id.to_sql()
            );

            // Spawn background task to poll agent output
            tokio::spawn(async move {
                let mut last_offset: i64 = 0;
                let poll_interval = std::time::Duration::from_secs(2);

                loop {
                    // Poll for new messages from agent session
                    let response = match agent_manager
                        .get_session_output(&conv_id, last_offset, 100)
                        .await
                    {
                        Ok(resp) => resp,
                        Err(e) => {
                            log::error!("Stream error for conversation {}: {}", conv_id.to_sql(), e);
                            break; // Exit loop on error
                        }
                    };

                    // Process each new message
                    for serialized_msg in response.output {
                        // Convert SerializedMessage to Message and invoke callback
                        if let Some(message) =
                            convert_serialized_to_message(&serialized_msg, &conv_id)
                        {
                            log::debug!(
                                "Stream message in {}: {} (type: {})",
                                conv_id.to_sql(),
                                &message.content[..message.content.len().min(50)],
                                serialized_msg.message_type
                            );

                            // Create Conversation entity for streaming
                            let conversation_entity = create_conversation_entity(&conv_id, message);

                            // Invoke callback with Message::Conversation
                            callback(megalodon::streaming::Message::Conversation(
                                conversation_entity,
                            ));
                        }
                    }

                    // Update offset for next poll
                    last_offset += response.messages_returned as i64;

                    // Check if session is complete
                    if response.is_complete {
                        log::info!(
                            "Agent session complete for conversation {}, ending stream",
                            conv_id.to_sql()
                        );
                        break;
                    }

                    // Wait before next poll
                    tokio::time::sleep(poll_interval).await;
                }

                log::debug!("Stream monitor terminated for conversation: {}", conv_id.to_sql());
            });
        }

        Ok(())
    }
}

/// Extract display content from SerializedMessage based on message type
///
/// Handles different content structures:
/// - Text content: Direct string or array of text blocks
/// - Tool use: Extracts tool name and input parameters
/// - Tool result: Extracts result content
/// - System messages: Extracts system message data
/// - Result messages: Formats completion summary
fn extract_message_content(
    serialized: &kodegen_tools_claude_agent::types::agent::SerializedMessage,
) -> Option<String> {
    match serialized.message_type.as_str() {
        "assistant" | "user" => {
            // Extract content field
            match serialized.content.get("content") {
                Some(serde_json::Value::String(s)) => Some(s.clone()),
                Some(serde_json::Value::Array(arr)) => {
                    // Process array of content blocks
                    let mut parts = Vec::new();

                    for block in arr {
                        // Check block type
                        if let Some(block_type) = block.get("type").and_then(|t| t.as_str()) {
                            match block_type {
                                "text" => {
                                    if let Some(text) = block.get("text").and_then(|t| t.as_str()) {
                                        parts.push(text.to_string());
                                    }
                                }
                                "tool_use" => {
                                    // Extract tool use information
                                    let name = block
                                        .get("name")
                                        .and_then(|n| n.as_str())
                                        .unwrap_or("unknown");
                                    let input = block
                                        .get("input")
                                        .map(|i| {
                                            serde_json::to_string_pretty(i).unwrap_or_default()
                                        })
                                        .unwrap_or_default();
                                    parts.push(format!("🔧 Tool Call: {}\nInput: {}", name, input));
                                }
                                "tool_result" => {
                                    // Extract tool result
                                    let content = block
                                        .get("content")
                                        .map(|c| c.to_string())
                                        .unwrap_or_default();
                                    parts.push(format!("✅ Tool Result:\n{}", content));
                                }
                                _ => {}
                            }
                        }
                    }

                    if parts.is_empty() {
                        None
                    } else {
                        Some(parts.join("\n\n"))
                    }
                }
                _ => None,
            }
        }
        msg_type if msg_type.starts_with("system_") => {
            // System message - extract and format
            let subtype = msg_type.strip_prefix("system_").unwrap_or("unknown");
            let data = serde_json::to_string_pretty(&serialized.content).unwrap_or_default();
            Some(format!("System: {}\n{}", subtype, data))
        }
        "result" => {
            // Result message - format completion summary
            let duration = serialized
                .content
                .get("duration_ms")
                .and_then(|d| d.as_u64())
                .unwrap_or(0);
            let turns = serialized
                .content
                .get("num_turns")
                .and_then(|t| t.as_u64())
                .unwrap_or(0);
            let is_error = serialized
                .content
                .get("is_error")
                .and_then(|e| e.as_bool())
                .unwrap_or(false);

            let status = if is_error {
                "❌ Completed with errors"
            } else {
                "✅ Completed successfully"
            };
            Some(format!(
                "{}\nTurns: {} | Duration: {}ms",
                status, turns, duration
            ))
        }
        _ => {
            log::debug!("Skipping message type: {}", serialized.message_type);
            None
        }
    }
}

/// Convert SerializedMessage from agent to view_model Message
///
/// # Arguments
/// * `serialized` - Message from agent session output
/// * `conversation_id` - Conversation this message belongs to
///
/// # Returns
/// * `Some(Message)` - Successfully converted message
/// * `None` - Message type not relevant for UI (e.g., internal events)
fn convert_serialized_to_message(
    serialized: &kodegen_tools_claude_agent::types::agent::SerializedMessage,
    conversation_id: &str,
) -> Option<crate::view_model::Message> {
    use crate::view_model::{AuthorType, ConversationId, Message, MessageId, MessageType};

    // Determine author, author_type, and message_type based on serialized.message_type
    let (author, author_type, message_type) = match serialized.message_type.as_str() {
        "user" => ("User".to_string(), AuthorType::Human, MessageType::Normal),
        "assistant" => {
            // Check if assistant message contains tool use
            let has_tool_use = serialized
                .content
                .get("content")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .any(|block| block.get("type").and_then(|t| t.as_str()) == Some("tool_use"))
                })
                .unwrap_or(false);

            if has_tool_use {
                (
                    "Assistant".to_string(),
                    AuthorType::Agent,
                    MessageType::Tool,
                )
            } else {
                (
                    "Assistant".to_string(),
                    AuthorType::Agent,
                    MessageType::Normal,
                )
            }
        }
        msg_type if msg_type.starts_with("system_") => {
            let subtype = msg_type.strip_prefix("system_").unwrap_or("unknown");
            (
                format!("System ({})", subtype),
                AuthorType::System,
                MessageType::System,
            )
        }
        "result" => (
            "System".to_string(),
            AuthorType::System,
            MessageType::System,
        ),
        unknown => {
            log::warn!("Unknown message type '{}', treating as system", unknown);
            (
                "System".to_string(),
                AuthorType::System,
                MessageType::System,
            )
        }
    };

    // Extract content using helper
    let content = extract_message_content(serialized)?;

    Some(Message {
        id: MessageId(uuid::Uuid::new_v4().to_string()),
        conversation_id: ConversationId(conversation_id.to_string()),
        author,
        author_type,
        content,
        timestamp: serialized.timestamp.into(),
        in_reply_to: None,
        message_type,
        attachments: Vec::new(),
        unread: author_type != AuthorType::Agent, // Mark non-agent messages as unread
        deleted: false,
        pinned: false,
    })
}

/// Create megalodon Conversation entity for streaming callback
///
/// # Arguments
/// * `conversation_id` - Conversation ID
/// * `message` - Latest message from agent
///
/// # Returns
/// * `Conversation` - Entity suitable for Message::Conversation variant
fn create_conversation_entity(
    conversation_id: &str,
    message: crate::view_model::Message,
) -> megalodon::entities::Conversation {
    use megalodon::entities::{Account, Conversation, Status, StatusVisibility};

    // Create agent account
    let agent_account = Account {
        id: conversation_id.to_string(),
        username: "assistant".to_string(),
        acct: "assistant".to_string(),
        display_name: "Assistant".to_string(),
        locked: false,
        discoverable: None,
        group: None,
        noindex: None,
        moved: None,
        suspended: None,
        limited: None,
        created_at: *message.timestamp,
        followers_count: 0,
        following_count: 0,
        statuses_count: 0,
        note: String::new(),
        url: String::new(),
        avatar: String::new(),
        avatar_static: String::new(),
        header: String::new(),
        header_static: String::new(),
        emojis: Vec::new(),
        fields: Vec::new(),
        bot: true,
        source: None,
        role: None,
        mute_expires_at: None,
    };

    // Create status from message
    let status = Status {
        id: message.id.0.to_sql(),
        uri: String::new(),
        created_at: *message.timestamp,
        account: agent_account.clone(),
        content: message.content.clone(),
        visibility: StatusVisibility::Direct, // Conversation is private
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: Vec::new(),
        emojis: Vec::new(),
        reblogs_count: 0,
        favourites_count: 0,
        replies_count: 0,
        url: None,
        in_reply_to_id: None,
        in_reply_to_account_id: None,
        reblog: None,
        poll: None,
        card: None,
        language: None,
        plain_content: Some(message.content.clone()),
        edited_at: None,
        favourited: None,
        reblogged: None,
        muted: None,
        bookmarked: None,
        pinned: None,
        quote: false,
        application: None,
        emoji_reactions: None,
    };

    // Build Conversation entity
    Conversation {
        id: conversation_id.to_string(),
        accounts: vec![agent_account],
        last_status: Some(status),
        unread: true,
    }
}

impl Model {
    /// Get unread messages as notifications (Q28-Q30: Agent response notifications)
    ///
    /// # Arguments
    /// * `_max_id` - Pagination (ignored for MVP)
    /// * `_limit` - Number of notifications (default 40)
    ///
    /// # Returns
    /// * `Ok(Vec<Notification>)` - Recent agent responses user hasn't seen
    /// * `Err(ModelError)` - Query failed
    pub async fn notifications(
        &self,
        _max_id: Option<String>,
        _limit: u32,
    ) -> Result<Vec<crate::environment::model::Notification>, ModelError> {
        let user_id = "hardcoded-david-maple"; // Q39: MVP hardcoded user

        // Get unread messages as notifications
        let messages = self
            .database()
            .get_unread_messages(user_id)
            .await
            .map_err(|e| {
                ModelError::QueryFailed(format!("Failed to get unread messages: {}", e))
            })?;

        // Transform messages to notifications
        Ok(messages.into_iter().map(message_to_notification).collect())
    }

    /// List agent templates (Q10, Q20: Template-based agent spawning)
    ///
    /// Repurposed from user_timeline to list available agent role templates.
    ///
    /// # Arguments
    /// * All arguments ignored - just returns all templates
    ///
    /// # Returns
    /// * `Ok(Vec<Status>)` - Agent templates as Status objects for UI compatibility
    /// * `Err(ModelError)` - Query failed
    pub async fn user_timeline(
        &self,
        _account_id: String,
        _max_id: Option<String>,
        _min_id: Option<String>,
        _limit: Option<u32>,
    ) -> Result<Vec<crate::environment::model::Status>, ModelError> {
        // Get agent templates from database
        let templates = self
            .database()
            .list_agent_templates()
            .await
            .map_err(|e| ModelError::QueryFailed(format!("Failed to list templates: {}", e)))?;

        // Transform to Status objects for UI
        Ok(templates.into_iter().map(template_to_status).collect())
    }

    /// Stub: Update media description (Q3: Media support later)
    pub async fn update_media(
        &self,
        _id: String,
        _description: Option<String>,
    ) -> Result<(), ModelError> {
        // Stub for media upload - not implemented in MVP
        Err(ModelError::NotImplemented(
            "Media upload not supported in MVP".to_string(),
        ))
    }
}

/// Transform Message to Notification for UI
fn message_to_notification(message: Message) -> crate::environment::model::Notification {
    use crate::environment::model::{Notification, NotificationType};
    use megalodon::entities::Account;

    Notification {
        id: message.id.0.to_sql(),
        r#type: NotificationType::Mention,
        created_at: *message.timestamp,
        account: Some(Account {
            id: message.conversation_id.0.to_sql(),
            username: message.author.clone(),
            acct: message.author.clone(),
            display_name: message.author.clone(),
            locked: false,
            discoverable: None,
            group: None,
            noindex: None,
            moved: None,
            suspended: None,
            limited: None,
            created_at: *message.timestamp,
            followers_count: 0,
            following_count: 0,
            statuses_count: 0,
            note: String::new(),
            url: String::new(),
            avatar: String::new(),
            avatar_static: String::new(),
            header: String::new(),
            header_static: String::new(),
            emojis: Vec::new(),
            fields: Vec::new(),
            bot: true,
            source: None,
            role: None,
            mute_expires_at: None,
        }),
        status: None,
        reaction: None,
        target: None,
    }
}

/// Create megalodon Account from message author
fn create_account_from_message(message: &Message) -> megalodon::entities::Account {
    megalodon::entities::Account {
        id: message.conversation_id.0.to_sql(),
        username: message.author.clone(),
        acct: message.author.clone(),
        display_name: message.author.clone(),
        locked: false,
        discoverable: None,
        group: None,
        noindex: None,
        moved: None,
        suspended: None,
        limited: None,
        created_at: *message.timestamp,
        followers_count: 0,
        following_count: 0,
        statuses_count: 0,
        note: String::new(),
        url: String::new(),
        avatar: String::new(),
        avatar_static: String::new(),
        header: String::new(),
        header_static: String::new(),
        emojis: Vec::new(),
        fields: Vec::new(),
        bot: true,
        source: None,
        role: None,
        mute_expires_at: None,
    }
}

/// Transform AgentTemplate to Status for UI listing
fn template_to_status(template: crate::view_model::AgentTemplate) -> Status {
    Status {
        id: template.id.0.to_sql(),
        uri: String::new(),
        created_at: chrono::Utc::now(),
        account: megalodon::entities::Account {
            id: template.id.0.to_sql(),
            username: template.name.clone(),
            acct: template.name.clone(),
            display_name: template.name.clone(),
            locked: false,
            discoverable: None,
            group: None,
            noindex: None,
            moved: None,
            suspended: None,
            limited: None,
            created_at: chrono::Utc::now(),
            followers_count: 0,
            following_count: 0,
            statuses_count: 0,
            note: template.system_prompt.clone(),
            url: String::new(),
            avatar: String::new(),
            avatar_static: String::new(),
            header: String::new(),
            header_static: String::new(),
            emojis: Vec::new(),
            fields: Vec::new(),
            bot: true,
            source: None,
            role: None,
            mute_expires_at: None,
        },
        content: template.system_prompt.clone(),
        visibility: StatusVisibility::Public,
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: Vec::new(),
        emojis: Vec::new(),
        reblogs_count: 0,
        favourites_count: 0,
        replies_count: 0,
        url: None,
        in_reply_to_id: None,
        in_reply_to_account_id: None,
        reblog: None,
        poll: None,
        card: None,
        language: None,
        plain_content: Some(template.system_prompt.clone()),
        edited_at: None,
        favourited: None,
        reblogged: None,
        muted: None,
        bookmarked: None,
        pinned: None,
        quote: false,
        application: None,
        emoji_reactions: None,
    }
}

/// Transform Message to Status with favorite status for UI
fn message_to_status(message: &Message, is_favourited: bool) -> Status {
    Status {
        id: message.id.0.to_sql(),
        uri: String::new(),
        created_at: *message.timestamp,
        account: create_account_from_message(message),
        content: message.content.clone(),
        in_reply_to_id: message.in_reply_to.as_ref().map(|id| id.0.to_sql()),
        visibility: StatusVisibility::Public,
        favourited: Some(is_favourited),
        favourites_count: if is_favourited { 1 } else { 0 },
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: Vec::new(),
        emojis: Vec::new(),
        reblogs_count: 0,
        replies_count: 0,
        url: None,
        in_reply_to_account_id: None,
        reblog: None,
        poll: None,
        card: None,
        language: None,
        plain_content: Some(message.content.clone()),
        edited_at: None,
        reblogged: None,
        muted: None,
        bookmarked: None,
        pinned: Some(message.pinned),
        quote: false,
        application: None,
        emoji_reactions: None,
    }
}

/// Transform Message to Status with bookmark status for UI
fn message_to_status_with_bookmark(message: &Message, is_bookmarked: bool) -> Status {
    Status {
        id: message.id.0.to_sql(),
        uri: String::new(),
        created_at: *message.timestamp,
        account: create_account_from_message(message),
        content: message.content.clone(),
        in_reply_to_id: message.in_reply_to.as_ref().map(|id| id.0.to_sql()),
        visibility: StatusVisibility::Public,
        bookmarked: Some(is_bookmarked),
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: Vec::new(),
        emojis: Vec::new(),
        reblogs_count: 0,
        favourites_count: 0,
        replies_count: 0,
        url: None,
        in_reply_to_account_id: None,
        reblog: None,
        poll: None,
        card: None,
        language: None,
        plain_content: Some(message.content.clone()),
        edited_at: None,
        favourited: None,
        reblogged: None,
        muted: None,
        pinned: Some(message.pinned),
        quote: false,
        application: None,
        emoji_reactions: None,
    }
}

/// Transform Message to Status (simple version for general use)
fn message_to_status_simple(message: &Message) -> Status {
    Status {
        id: message.id.0.to_sql(),
        uri: String::new(),
        created_at: *message.timestamp,
        account: create_account_from_message(message),
        content: message.content.clone(),
        in_reply_to_id: message.in_reply_to.as_ref().map(|id| id.0.to_sql()),
        visibility: StatusVisibility::Public,
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: Vec::new(),
        emojis: Vec::new(),
        reblogs_count: 0,
        favourites_count: 0,
        replies_count: 0,
        url: None,
        in_reply_to_account_id: None,
        reblog: None,
        poll: None,
        card: None,
        language: None,
        plain_content: Some(message.content.clone()),
        edited_at: None,
        favourited: None,
        reblogged: None,
        muted: None,
        bookmarked: None,
        pinned: Some(message.pinned),
        quote: false,
        application: None,
        emoji_reactions: None,
    }
}
