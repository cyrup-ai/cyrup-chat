//! Stateless agent chat service using ClaudeSDKClient
//!
//! Implements database-driven chat with:
//! - Fresh subprocess per message using resume parameter
//! - Lazy spawn pattern via conversation.agent_session_id
//! - Streaming responses via INSERT + UPDATE pattern
//! - LIVE QUERY subscribers receive Action::Update automatically

use crate::database::Database;
use crate::view_model::message::{AuthorType, Message, MessageType};
use flume::{Receiver, Sender, unbounded};
use surrealdb_types::{RecordId, ToSql};
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

/// Send user message and stream agent response(s) - unified 1:N agent handler
///
/// # Arguments
/// * `database` - Database connection
/// * `conversation_id` - Conversation ID
/// * `user_message` - User message content (may contain @mentions for multi-agent)
/// * `mentioned_agents` - Optional agent IDs to route to (empty = all participants)
/// * `parent_message_id` - Optional parent message ID for threading
///
/// # Errors
/// Returns error if message cannot be sent or database operations fail
///
/// # Architecture
/// Single agent (participants.len() == 1):
/// 1. INSERT user message to database
/// 2. GET conversation (includes agent_sessions HashMap and participants Vec)
/// 3. GET template for model/system_prompt/max_turns
/// 4. CREATE ClaudeSDKClient with resume from agent_sessions[agent_id]
/// 5. Stream responses with debouncing (100ms OR 50 chars)
/// 6. Store session_id via update_agent_session()
///
/// Multi-agent (participants.len() > 1):
/// 1. INSERT user message to database
/// 2. Parse @mentions or use all participants
/// 3. Spawn concurrent agent tasks using FuturesUnordered
/// 4. Each agent streams independently with session persistence
/// 5. All responses appear in unified timeline via LIVE QUERY
pub async fn send_message(
    database: Arc<Database>,
    conversation_id: RecordId,
    user_message: String,
    mentioned_agents: Option<Vec<RecordId>>,
    parent_message_id: Option<RecordId>,
) -> Result<(), String> {
    // 1. Save user message to database
    let user_msg = Message {
        id: RecordId::new("message", "default"), // DB generates actual ID
        conversation_id: conversation_id.clone(),
        author: "User".to_string(),
        author_type: AuthorType::Human,
        content: user_message.clone(),
        timestamp: chrono::Utc::now().into(),
        in_reply_to: parent_message_id,
        message_type: MessageType::Normal,
        attachments: Vec::new(),
        unread: false, // User's own messages start as read
        deleted: false,
        pinned: false,
    };

    let user_msg_id = database.insert_message(&user_msg).await?;

    // 2. Get conversation (has participants and agent_sessions)
    let conversation = database.get_conversation(&conversation_id).await?;

    // 3. Determine which agents to message
    let target_agents: Vec<RecordId> = if let Some(agents) = mentioned_agents {
        // Multi-agent: Use @mentioned agents
        agents
    } else if conversation.participants.len() == 1 {
        // Single-agent: Use the one participant
        vec![conversation.participants[0].clone()]
    } else {
        // Multi-agent without mentions: Message all participants
        conversation.participants.clone()
    };

    // Validate target_agents not empty (follows pattern from notifications/content.rs:28)
    if target_agents.is_empty() {
        return Err("No target agents specified".to_string());
    }

    log::info!(
        "[Chat] Sending message to conversation {} with {} target agent(s)",
        conversation_id.to_sql(),
        target_agents.len()
    );

    // 4. Execute based on agent count
    if target_agents.len() == 1 {
        // Single agent path: Direct execution with session persistence
        let agent_id = &target_agents[0];
        send_to_single_agent(
            database,
            conversation_id,
            user_message,
            user_msg_id,
            agent_id,
            conversation.agent_sessions.get(agent_id).cloned(),
        )
        .await
    } else {
        // Multi-agent path: Concurrent execution with FuturesUnordered
        send_to_multiple_agents(
            database,
            conversation_id,
            user_message,
            user_msg_id,
            target_agents,
            conversation.agent_sessions,
        )
        .await
    }
}

/// Single agent message handler with session persistence
async fn send_to_single_agent(
    database: Arc<Database>,
    conversation_id: RecordId,
    user_message: String,
    user_msg_id: RecordId,
    agent_id: &RecordId,
    existing_session_id: Option<String>,
) -> Result<(), String> {
    log::debug!("[Chat] Single agent mode: {}", agent_id.to_sql());

    // Get agent template
    let template = database.get_template(agent_id).await?;

    // Create ClaudeSDKClient (fresh subprocess each time)
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
        resume: existing_session_id
            .as_ref()
            .map(|id| kodegen_tools_claude_agent::types::identifiers::SessionId::from(id.as_str())),
        ..Default::default()
    };

    let mut client = ClaudeSDKClient::new(options, None)
        .await
        .map_err(|e| format!("Failed to create Claude client: {}", e))?;

    // Send message
    client
        .send_message(&user_message)
        .await
        .map_err(|e| format!("Failed to send to agent: {}", e))?;

    // Stream responses
    stream_agent_responses(
        client,
        database,
        conversation_id,
        user_msg_id,
        agent_id.clone(),
    )
    .await
}

/// Multi-agent message handler with concurrent execution
async fn send_to_multiple_agents(
    database: Arc<Database>,
    conversation_id: RecordId,
    user_message: String,
    user_msg_id: RecordId,
    target_agents: Vec<RecordId>,
    agent_sessions: std::collections::HashMap<RecordId, String>,
) -> Result<(), String> {
    log::info!(
        "[Chat] Multi-agent mode: {} agents",
        target_agents.len()
    );

    // Spawn concurrent agent tasks
    let mut agent_tasks = FuturesUnordered::new();

    for agent_id in target_agents {
        let database = database.clone();
        let conversation_id = conversation_id.clone();
        let user_message = user_message.clone();
        let user_msg_id = user_msg_id.clone();
        let existing_session_id = agent_sessions.get(&agent_id).cloned();

        agent_tasks.push(async move {
            log::info!("[Chat] Spawning agent: {}", agent_id.to_sql());

            // Get agent template
            let template = database.get_template(&agent_id).await?;

            // Create ClaudeSDKClient
            let options = ClaudeAgentOptions {
                model: Some(template.model.to_string().to_lowercase()),
                system_prompt: Some(SystemPrompt::String(template.system_prompt.clone())),
                max_turns: Some(template.max_turns),
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
                // Resume from previous session if exists
                resume: existing_session_id
                    .as_ref()
                    .map(|id| kodegen_tools_claude_agent::types::identifiers::SessionId::from(id.as_str())),
                ..Default::default()
            };

            let mut client = ClaudeSDKClient::new(options, None)
                .await
                .map_err(|e| format!("Failed to create Claude client for {}: {}", agent_id.to_sql(), e))?;

            log::debug!("[Chat] Sending message to agent {}", agent_id.to_sql());

            // Send message
            client
                .send_message(&user_message)
                .await
                .map_err(|e| format!("Failed to send to agent {}: {}", agent_id.to_sql(), e))?;

            // Stream responses
            stream_agent_responses(
                client,
                database,
                conversation_id,
                user_msg_id,
                agent_id.clone(),
            )
            .await
        });
    }

    // Track success/failure counts
    let mut success_count = 0;
    let mut failure_count = 0;
    let mut last_error: Option<String> = None;

    // Wait for all agents to complete (concurrent execution)
    while let Some(result) = agent_tasks.next().await {
        match result {
            Ok(_) => {
                success_count += 1;
                log::debug!("[Chat] Agent completed successfully");
            }
            Err(e) => {
                failure_count += 1;
                last_error = Some(e.clone());
                log::error!("[Chat] Agent error: {}", e);
            }
        }
    }

    // Return error if ALL agents failed (total failure)
    if success_count == 0 && failure_count > 0 {
        let error_msg = format!(
            "All {} agents failed to respond. Last error: {}",
            failure_count,
            last_error.unwrap_or_else(|| "Unknown".to_string())
        );

        log::error!("[Chat] {}", error_msg);
        return Err(error_msg);
    }

    log::info!(
        "[Chat] Multi-agent completion: {} succeeded, {} failed",
        success_count,
        failure_count
    );

    Ok(())
}


/// Stream agent responses and update database
///
/// Consumes ClaudeSDKClient stream and updates database with responses.
/// LIVE QUERY subscribers receive notifications automatically.
async fn stream_agent_responses(
    mut client: ClaudeSDKClient,
    database: Arc<Database>,
    conversation_id: RecordId,
    user_msg_id: RecordId,
    agent_id: RecordId,
) -> Result<(), String> {
    let mut accumulated_text = String::new();
    let mut message_id: Option<RecordId> = None;
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
                                        id: RecordId::default(),
                                        conversation_id: conversation_id.clone(),
                                        author: "Assistant".to_string(),
                                        author_type: AuthorType::Agent,
                                        content: accumulated_text.clone(),
                                        timestamp: chrono::Utc::now().into(),
                                        in_reply_to: Some(user_msg_id.clone()),
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
                if let Some(id) = message_id.as_ref()
                    && chars_since_last_update > 0 {
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

                // Store session_id from Result
                session_id = Some(sid.as_str().to_string());

                // Handle completion or error
                if is_error {
                    let error_text = result.unwrap_or_else(|| "Unknown error".to_string());
                    log::error!("[AgentChat] Agent completed with error: {}", error_text);

                    // Save error message
                    let error_msg = Message {
                        id: RecordId::default(),
                        conversation_id: conversation_id.clone(),
                        author: "system".to_string(),
                        author_type: AuthorType::System,
                        content: format!("⚠️ Agent Error: {}", error_text),
                        timestamp: chrono::Utc::now().into(),
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
                    id: RecordId::default(),
                    conversation_id: conversation_id.clone(),
                    author: "system".to_string(),
                    author_type: AuthorType::System,
                    content: format!("⚠️ Stream error: {}", e),
                    timestamp: chrono::Utc::now().into(),
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
        log::info!("[Chat] Storing session_id for agent {} in conversation {}", agent_id.to_sql(), conversation_id.to_sql());
        database
            .update_agent_session(&conversation_id, &agent_id, &sid)
            .await?;
    }

    Ok(())
}
