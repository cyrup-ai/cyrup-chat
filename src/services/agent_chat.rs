//! Stateless agent chat service using ClaudeSDKClient
//!
//! Implements database-driven chat with:
//! - Fresh subprocess per message using resume parameter
//! - Lazy spawn pattern via conversation.agent_session_id
//! - Streaming responses via INSERT + UPDATE pattern
//! - LIVE QUERY subscribers receive Action::Update automatically

use crate::database::Database;
use crate::view_model::conversation::ConversationId;
use crate::view_model::message::{AuthorType, Message, MessageId, MessageType};
use flume::{Receiver, Sender, unbounded};
use futures_util::stream::{FuturesUnordered, StreamExt};  // For concurrent agent execution
use kodegen_tools_claude_agent::{
    ClaudeAgentOptions, ClaudeSDKClient, ContentBlock, Message as AgentMessage, SystemPrompt,
};
use std::sync::{Arc, OnceLock};
use tokio::time::{Duration, Instant};

/// Global event channel for broadcasting agent tool-use events to UI
///
/// Uses OnceLock for thread-safe lazy initialization.
/// Channel is unbounded to prevent blocking the agent stream.
static TOOL_EVENT_CHANNEL: OnceLock<(Sender<String>, Receiver<String>)> = OnceLock::new();

/// Get or initialize the tool event channel
///
/// Returns a static reference to the channel pair.
/// First call initializes, subsequent calls return the same instance.
pub fn get_tool_event_channel() -> &'static (Sender<String>, Receiver<String>) {
    TOOL_EVENT_CHANNEL.get_or_init(unbounded)
}

/// Send user message and stream agent response
///
/// # Arguments
/// * `database` - Database connection
/// * `conversation_id` - Conversation ID
/// * `user_message` - User message content
/// * `parent_message_id` - Optional parent message ID for threading
///
/// # Errors
/// Returns error if message cannot be sent or database operations fail
///
/// # Architecture
/// 1. INSERT user message to database
/// 2. GET conversation (includes agent_session_id and template_id)
/// 3. GET template for model/system_prompt/max_turns
/// 4. CREATE fresh ClaudeSDKClient with resume: conversation.agent_session_id
/// 5. Stream responses:
///    - First text chunk: INSERT message → save message_id
///    - Subsequent chunks: UPDATE message SET content = accumulated_text
///    - LIVE QUERY subscribers receive Action::Update automatically
/// 6. Extract session_id from Message::Result
/// 7. Store via database.update_conversation_session() for next turn
pub async fn send_chat_message(
    database: Arc<Database>,
    conversation_id: String,
    user_message: String,
    parent_message_id: Option<String>,
) -> Result<(), String> {
    // 1. Save user message to database
    let user_msg = Message {
        id: MessageId::default(), // DB generates actual ID
        conversation_id: ConversationId(conversation_id.clone()),
        author: "User".to_string(),
        author_type: AuthorType::Human,
        content: user_message.clone(),
        timestamp: chrono::Utc::now(),
        in_reply_to: parent_message_id.map(MessageId),
        message_type: MessageType::Normal,
        attachments: Vec::new(),
        unread: false, // User's own messages start as read
        deleted: false,
        pinned: false,
    };

    let user_msg_id = database.insert_message(&user_msg).await?;

    // 2. Get conversation (has template_id and agent_session_id)
    let conversation = database.get_conversation(&conversation_id).await?;
    let template = database.get_template(&conversation.template_id.0).await?;

    // 3. Create ClaudeSDKClient (fresh subprocess each time)
    let options = ClaudeAgentOptions {
        model: Some(template.model.to_string().to_lowercase()),
        system_prompt: Some(SystemPrompt::String(template.system_prompt.clone())),
        max_turns: Some(template.max_turns),
        // Enable core development tools
        allowed_tools: vec![
            "Read".into(),      // Read files
            "Write".into(),     // Write files
            "Edit".into(),      // Edit files
            "Bash".into(),      // Execute commands
            "Glob".into(),      // Find files by pattern
            "Grep".into(),      // Search file contents
            "Task".into(),      // Spawn sub-agents
            "WebFetch".into(),  // Fetch web content
            "WebSearch".into(), // Search the web
        ],
        // Resume from previous session if exists (lazy spawn pattern)
        resume: conversation
            .agent_session_id
            .as_ref()
            .map(|id| kodegen_tools_claude_agent::types::identifiers::SessionId::from(id.as_str())),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options, None)
        .await
        .map_err(|e| format!("Failed to create Claude client: {}", e))?;

    // 4. Send message
    client
        .send_message(&user_message)
        .await
        .map_err(|e| format!("Failed to send to agent: {}", e))?;

    // 5. Stream responses
    stream_agent_responses(client, database, conversation_id, user_msg_id).await
}

/// Send message to multiple agents concurrently (multi-agent room routing)
///
/// # Arguments
/// * `database` - Database connection
/// * `room_id` - Room ID (used as conversation_id for messages)
/// * `user_message` - User message content (may contain @mentions)
/// * `mentioned_agents` - Agent template IDs to route message to (from mention_parser)
///
/// # Returns
/// * `Ok(())` - All agent responses streamed successfully
/// * `Err(String)` - Error if user message save fails (agent errors logged but don't fail)
///
/// # Architecture
/// 1. Save user message to database (conversation_id = room_id)
/// 2. Spawn concurrent agent tasks using FuturesUnordered
/// 3. Each agent:
///    - Loads template (model, system_prompt, max_turns)
///    - Creates fresh ClaudeSDKClient (no resume)
///    - Streams responses using stream_agent_responses()
/// 4. All agents run concurrently, responses appear in room timeline
///
/// # Streaming Pattern
/// - Uses existing stream_agent_responses() helper (lines 127-250)
/// - Debouncing: 100ms interval OR 50 chars (lines 135-136)
/// - LIVE QUERY: Database updates trigger UI refresh automatically
///
/// # Error Handling
/// - User message save failure → return Err (critical)
/// - Individual agent failures → log error, continue with other agents (non-critical)
pub async fn send_room_message(
    database: Arc<Database>,
    room_id: String,
    user_message: String,
    mentioned_agents: Vec<String>,  // Agent template IDs from @mentions
) -> Result<(), String> {
    log::info!(
        "[RoomChat] Sending message to room {} with {} mentioned agents",
        room_id,
        mentioned_agents.len()
    );

    // 1. Save user message to database
    // Note: Rooms use same message table as conversations (conversation_id field)
    let user_msg = Message {
        id: MessageId::default(),  // DB generates actual ID
        conversation_id: ConversationId(room_id.clone()),
        author: "User".to_string(),
        author_type: AuthorType::Human,
        content: user_message.clone(),
        timestamp: chrono::Utc::now(),
        in_reply_to: None,
        message_type: MessageType::Normal,
        attachments: Vec::new(),
        unread: false,  // User's own messages start as read
        deleted: false,
        pinned: false,
    };

    let user_msg_id = database
        .insert_message(&user_msg)
        .await
        .map_err(|e| format!("Failed to save user message: {}", e))?;

    log::debug!("[RoomChat] Saved user message: {}", user_msg_id);

    // 2. Spawn concurrent agent tasks
    let mut agent_tasks = FuturesUnordered::new();

    for agent_id in mentioned_agents {
        let database = database.clone();
        let room_id = room_id.clone();
        let user_message = user_message.clone();
        let user_msg_id = user_msg_id.clone();

        agent_tasks.push(async move {
            log::info!("[RoomChat] Spawning agent: {}", agent_id);

            // Get agent template (model, system_prompt, max_turns)
            let template = database.get_template(&agent_id).await?;

            // Create fresh ClaudeSDKClient for this agent
            let options = ClaudeAgentOptions {
                model: Some(template.model.to_string().to_lowercase()),
                system_prompt: Some(SystemPrompt::String(template.system_prompt.clone())),
                max_turns: Some(template.max_turns),
                // Enable core development tools
                allowed_tools: vec![
                    "Read".into(),
                    "Write".into(),
                    "Edit".into(),
                    "Bash".into(),
                    "Glob".into(),
                    "Grep".into(),
                    "Task".into(),
                    "WebFetch".into(),
                    "WebSearch".into(),
                ],
                resume: None,  // Fresh session for each room message (no state retention)
                ..Default::default()
            };

            let mut client = ClaudeSDKClient::new(options, None)
                .await
                .map_err(|e| format!("Failed to create Claude client for {}: {}", agent_id, e))?;

            log::debug!("[RoomChat] Sending message to agent {}", agent_id);

            // Send message to agent
            client
                .send_message(&user_message)
                .await
                .map_err(|e| format!("Failed to send to agent {}: {}", agent_id, e))?;

            // Stream responses using existing helper
            // This handles: INSERT first chunk → UPDATE subsequent chunks → LIVE QUERY
            stream_agent_responses(client, database, room_id, user_msg_id).await
        });
    }

    // 3. Wait for all agents to complete (concurrent execution)
    while let Some(result) = agent_tasks.next().await {
        if let Err(e) = result {
            // Log error but continue with other agents
            log::error!("[RoomChat] Agent error: {}", e);
        }
    }

    log::info!("[RoomChat] All agents completed");
    Ok(())
}

/// Stream agent responses and update database
///
/// Consumes ClaudeSDKClient stream and updates database with responses.
/// LIVE QUERY subscribers receive notifications automatically.
async fn stream_agent_responses(
    mut client: ClaudeSDKClient,
    database: Arc<Database>,
    conversation_id: String,
    user_msg_id: String,
) -> Result<(), String> {
    let mut accumulated_text = String::new();
    let mut message_id: Option<String> = None;
    let mut session_id: Option<String> = None;

    // Debouncing state
    let mut last_update = Instant::now();
    let update_interval = Duration::from_millis(100);
    let mut chars_since_last_update: usize = 0;
    const MIN_CHARS_FOR_UPDATE: usize = 50;

    // Process stream messages
    while let Some(message) = client.next_message().await {
        match message {
            Ok(AgentMessage::Assistant {
                message,
                session_id: sid,
                ..
            }) => {
                // Store session_id for later
                if let Some(sid) = sid {
                    session_id = Some(sid.as_str().to_string());
                }

                // Iterate through content blocks
                for block in &message.content {
                    match block {
                        ContentBlock::Text { text } => {
                            // Accumulate text chunks
                            let chunk_len = text.len();
                            accumulated_text.push_str(text);
                            chars_since_last_update += chunk_len;

                            // Debouncing logic: Only update if conditions are met
                            let should_update = message_id.is_none()  // Always insert first chunk
                                || last_update.elapsed() >= update_interval  // Time-based: 100ms elapsed
                                || chars_since_last_update >= MIN_CHARS_FOR_UPDATE; // Size-based: 50+ chars

                            if should_update {
                                // First chunk: INSERT message
                                if message_id.is_none() {
                                    let msg = Message {
                                        id: MessageId::default(),
                                        conversation_id: ConversationId(conversation_id.clone()),
                                        author: "Assistant".to_string(),
                                        author_type: AuthorType::Agent,
                                        content: accumulated_text.clone(),
                                        timestamp: chrono::Utc::now(),
                                        in_reply_to: Some(MessageId(user_msg_id.clone())),
                                        message_type: MessageType::Normal,
                                        attachments: Vec::new(),
                                        unread: true,
                                        deleted: false,
                                        pinned: false,
                                    };

                                    match database.insert_message(&msg).await {
                                        Ok(id) => {
                                            message_id = Some(id);
                                            last_update = Instant::now();
                                            chars_since_last_update = 0;
                                            log::debug!(
                                                "[AgentChat] Inserted initial message: {:?}",
                                                message_id
                                            );
                                        }
                                        Err(e) => {
                                            log::error!(
                                                "[AgentChat] Failed to insert message: {}",
                                                e
                                            );
                                            return Err(e);
                                        }
                                    }
                                } else {
                                    // Subsequent chunks: UPDATE message (debounced)
                                    // LIVE QUERY subscribers receive Action::Update automatically
                                    if let Some(id) = message_id.as_ref() {
                                        match database
                                            .update_message_content(id, accumulated_text.clone())
                                            .await
                                        {
                                            Ok(_) => {
                                                last_update = Instant::now();
                                                chars_since_last_update = 0;
                                                log::debug!(
                                                    "[AgentChat] Updated message with {} chars (debounced)",
                                                    accumulated_text.len()
                                                );
                                            }
                                            Err(e) => {
                                                log::error!(
                                                    "[AgentChat] Failed to update message: {}",
                                                    e
                                                );
                                                return Err(e);
                                            }
                                        }
                                    } else {
                                        log::error!(
                                            "[AgentChat] message_id is None in update path"
                                        );
                                        return Err(
                                            "Internal error: message_id not set before update"
                                                .to_string(),
                                        );
                                    }
                                }
                            }
                        }
                        ContentBlock::ToolUse { name, .. } => {
                            log::info!("[AgentChat] Agent using tool: {}", name);

                            // Broadcast to UI subscribers
                            let (sender, _) = get_tool_event_channel();
                            if let Err(e) = sender.send(name.clone()) {
                                log::warn!("[AgentChat] Failed to broadcast tool event: {}", e);
                            }
                        }
                        ContentBlock::Thinking { thinking, .. } => {
                            log::debug!("[AgentChat] Agent thinking: {}", thinking);
                        }
                        _ => {}
                    }
                }
            }
            Ok(AgentMessage::Result {
                is_error,
                result,
                session_id: sid,
                ..
            }) => {
                // IMPORTANT: Flush any pending updates before completing
                if let Some(id) = message_id.as_ref() {
                    if chars_since_last_update > 0 {
                        match database
                            .update_message_content(id, accumulated_text.clone())
                            .await
                        {
                            Ok(_) => {
                                log::debug!(
                                    "[AgentChat] Flushed final update with {} chars",
                                    accumulated_text.len()
                                );
                            }
                            Err(e) => {
                                log::error!("[AgentChat] Failed to flush final update: {}", e);
                                return Err(e);
                            }
                        }
                    }
                }

                // Store session_id from Result
                session_id = Some(sid.as_str().to_string());

                // Handle completion or error
                if is_error {
                    let error_text = result.unwrap_or_else(|| "Unknown error".to_string());
                    log::error!("[AgentChat] Agent completed with error: {}", error_text);

                    // Save error message
                    let error_msg = Message {
                        id: MessageId::default(),
                        conversation_id: ConversationId(conversation_id.clone()),
                        author: "system".to_string(),
                        author_type: AuthorType::System,
                        content: format!("⚠️ Agent Error: {}", error_text),
                        timestamp: chrono::Utc::now(),
                        in_reply_to: None,
                        message_type: MessageType::Error,
                        attachments: Vec::new(),
                        unread: true,
                        deleted: false,
                        pinned: false,
                    };

                    if let Err(e) = database.insert_message(&error_msg).await {
                        log::error!("[AgentChat] Failed to save error message: {}", e);
                    }
                } else {
                    log::info!("[AgentChat] Agent completed successfully");
                }

                break; // Stream complete
            }
            Ok(AgentMessage::System { subtype, data }) => {
                log::debug!("[AgentChat] System message: {} - {:?}", subtype, data);
            }
            Err(e) => {
                log::error!("[AgentChat] Stream error: {}", e);

                let error_msg = Message {
                    id: MessageId::default(),
                    conversation_id: ConversationId(conversation_id.clone()),
                    author: "system".to_string(),
                    author_type: AuthorType::System,
                    content: format!("⚠️ Stream error: {}", e),
                    timestamp: chrono::Utc::now(),
                    in_reply_to: None,
                    message_type: MessageType::Error,
                    attachments: Vec::new(),
                    unread: true,
                    deleted: false,
                    pinned: false,
                };

                if let Err(e) = database.insert_message(&error_msg).await {
                    log::error!("[AgentChat] Failed to save error message: {}", e);
                }

                break;
            }
            _ => {}
        }
    }

    // Store session_id for next turn (lazy spawn pattern)
    if let Some(sid) = session_id {
        log::info!("[AgentChat] Storing session_id for next turn: {}", sid);
        database
            .update_conversation_session(&conversation_id, &sid)
            .await?;
    }

    Ok(())
}
