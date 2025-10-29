//! Real-time message streaming from agent sessions
//!
//! Streams agent responses using kodegen_tools_claude_agent and saves
//! each message chunk to database immediately for UI updates.

use futures_util::StreamExt;
use std::sync::Arc;

use crate::database::Database;
use crate::services::agent_chat::get_tool_event_channel;
use crate::view_model::conversation::ConversationId;
use crate::view_model::message::{AuthorType, Message, MessageId, MessageType};
use kodegen_tools_claude_agent::{
    ClaudeAgentOptions, ContentBlock, Message as AgentMessage, query,
};

/// Stream agent responses and save to database
///
/// Spawns background task that:
/// 1. Streams messages from agent via query()
/// 2. Extracts text from ContentBlock::Text
/// 3. Saves each text chunk to database immediately
/// 4. Logs tool usage for typing indicators
/// 5. Handles errors and completion
///
/// # Arguments
/// * `conversation_id` - Conversation to save messages to
/// * `prompt` - User prompt to send to agent
/// * `options` - Agent configuration (model, max_turns, system prompt)
/// * `db` - Database connection
///
/// # Pattern
/// Follows SummarizerService pattern from [src/services/summarizer.rs:190-212](../src/services/summarizer.rs)
pub async fn stream_agent_responses(
    conversation_id: String,
    prompt: String,
    options: ClaudeAgentOptions,
    db: Arc<Database>,
) -> Result<(), String> {
    // Spawn background task for non-blocking streaming
    tokio::spawn(async move {
        // Start agent stream
        let stream = match query(&prompt, Some(options)).await {
            Ok(s) => s,
            Err(e) => {
                log::error!("Failed to start agent stream: {}", e);
                // Save error message to database
                let error_msg = create_error_message(
                    &conversation_id,
                    &format!("Failed to start agent: {}", e),
                );
                if let Err(e) = db.insert_message(&error_msg).await {
                    log::error!("Failed to save error message: {}", e);
                }
                return;
            }
        };

        let mut stream = Box::pin(stream);
        let mut accumulated_text = String::new();

        // Process stream messages
        while let Some(message) = stream.next().await {
            match message {
                Ok(AgentMessage::Assistant { message, .. }) => {
                    // Iterate through content blocks
                    for block in &message.content {
                        match block {
                            ContentBlock::Text { text } => {
                                // Accumulate text chunks
                                accumulated_text.push_str(text);

                                // Save to database (could batch for performance)
                                let msg = create_agent_message(&conversation_id, &accumulated_text);

                                if let Err(e) = db.insert_message(&msg).await {
                                    log::error!("Failed to save agent message: {}", e);
                                }
                            }
                            ContentBlock::ToolUse { name, .. } => {
                                // Q11: Log tool usage for typing indicator
                                log::info!("Agent using tool: {}", name);

                                // Broadcast to UI subscribers
                                let (sender, _) = get_tool_event_channel();
                                if let Err(e) = sender.send(name.clone()) {
                                    log::warn!("Failed to broadcast tool event: {}", e);
                                }
                            }
                            ContentBlock::Thinking { thinking, .. } => {
                                // Log thinking for debugging
                                log::debug!("Agent thinking: {}", thinking);
                            }
                            _ => {}
                        }
                    }
                }
                Ok(AgentMessage::Result {
                    is_error, result, ..
                }) => {
                    // Handle completion or error
                    if is_error {
                        let error_text = result.unwrap_or_else(|| "Unknown error".to_string());
                        log::error!("Agent completed with error: {}", error_text);

                        // Save error message
                        let error_msg = create_error_message(&conversation_id, &error_text);
                        if let Err(e) = db.insert_message(&error_msg).await {
                            log::error!("Failed to save error message: {}", e);
                        }
                    } else {
                        log::info!("Agent completed successfully");

                        // Q29: Trigger OS notification when agent finishes
                        let preview = if accumulated_text.len() > 100 {
                            format!("{}...", &accumulated_text[..100])
                        } else {
                            accumulated_text.clone()
                        };

                        // Spawn notification in background (don't block stream completion)
                        tokio::spawn(async move {
                            if let Err(e) =
                                crate::notifications::NotificationService::send_notification(
                                    "Agent Response",
                                    &preview,
                                )
                                .await
                            {
                                log::error!("Failed to send OS notification: {}", e);
                            }
                        });
                    }
                    break; // Stream complete
                }
                Ok(AgentMessage::System { subtype, data }) => {
                    // Log system messages for debugging
                    log::debug!("System message: {} - {:?}", subtype, data);
                }
                Err(e) => {
                    // Stream error
                    log::error!("Stream error: {}", e);

                    let error_msg =
                        create_error_message(&conversation_id, &format!("Stream error: {}", e));
                    if let Err(e) = db.insert_message(&error_msg).await {
                        log::error!("Failed to save error message: {}", e);
                    }
                    break;
                }
                _ => {}
            }
        }
    });

    Ok(())
}

/// Create agent message for database insertion
fn create_agent_message(conversation_id: &str, content: &str) -> Message {
    Message {
        id: MessageId::default(), // DB generates actual ID
        conversation_id: ConversationId::from(conversation_id),
        author: "Assistant".to_string(),
        author_type: AuthorType::Agent,
        content: content.to_string(),
        timestamp: chrono::Utc::now().into(),
        in_reply_to: None,
        message_type: MessageType::Normal,
        attachments: Vec::new(),
        unread: true, // New agent messages start as unread
        deleted: false,
        pinned: false,
    }
}

/// Create error message for database insertion
fn create_error_message(conversation_id: &str, error: &str) -> Message {
    Message {
        id: MessageId::default(),
        conversation_id: ConversationId::from(conversation_id),
        author: "system".to_string(),
        author_type: AuthorType::System,
        content: format!("⚠️ Agent Error: {}", error),
        timestamp: chrono::Utc::now().into(),
        in_reply_to: None,
        message_type: MessageType::Error,
        attachments: Vec::new(),
        unread: true,
        deleted: false,
        pinned: false,
    }
}
