//! Agent manager integration - bridges database conversations to Claude agent sessions
//!
//! Implements lazy spawn pattern (Q48): agent session created on first message send,
//! not at conversation creation time.

use kodegen_tools_claude_agent::manager::SpawnSessionRequest;
use kodegen_tools_claude_agent::types::agent::GetOutputResponse;
use kodegen_tools_claude_agent::{AgentManager, ClaudeError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::database::Database;
use crate::view_model::{AgentModel, AgentTemplate, AuthorType, Message};

// ============================================================================
// ERROR TYPES
// ============================================================================

/// Errors from agent management operations
#[derive(Debug)]
pub enum AgentManagerError {
    /// Agent session not found for conversation
    SessionNotFound(String),
    /// Agent session already exists for conversation
    SessionAlreadyExists(String),
    /// Underlying AgentManager error
    AgentError(ClaudeError),
    /// Database operation failed
    DatabaseError(String),
}

impl std::fmt::Display for AgentManagerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionNotFound(id) => write!(f, "No agent session for conversation {}", id),
            Self::SessionAlreadyExists(id) => {
                write!(f, "Agent already spawned for conversation {}", id)
            }
            Self::AgentError(e) => write!(f, "Agent error: {}", e),
            Self::DatabaseError(e) => write!(f, "Database error: {}", e),
        }
    }
}

impl std::error::Error for AgentManagerError {}

impl From<ClaudeError> for AgentManagerError {
    fn from(e: ClaudeError) -> Self {
        Self::AgentError(e)
    }
}

// ============================================================================
// MODEL AGENT MANAGER
// ============================================================================

/// Model layer wrapper around AgentManager with conversation-to-session mapping
///
/// Provides lazy spawn pattern where agent sessions are created on first user message,
/// not at conversation creation time. Maintains bidirectional mapping between
/// conversation IDs and agent session IDs.
pub struct ModelAgentManager {
    /// Core agent manager from kodegen_tools_claude_agent
    agent_manager: AgentManager,

    /// In-memory mapping: conversation_id -> session_id
    /// Uses Arc<Mutex> for thread-safe concurrent access
    active_sessions: Arc<Mutex<HashMap<String, String>>>,

    /// Database handle for updating conversation.agent_session_id
    db: Arc<Database>,
}
impl ModelAgentManager {
    /// Create new ModelAgentManager with database connection
    ///
    /// # Arguments
    /// * `db` - Shared database connection for updating conversation records
    ///
    /// # Returns
    /// Initialized ModelAgentManager with empty session map
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            agent_manager: AgentManager::new(),
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            db,
        }
    }

    /// Spawn agent session for conversation (lazy spawn pattern Q48)
    ///
    /// Builds system prompt from template + conversation context (summary + recent messages),
    /// spawns AgentManager session, stores mapping, and updates database.
    ///
    /// # Arguments
    /// * `conversation_id` - Database conversation ID
    /// * `template` - Agent configuration template
    /// * `summary` - Conversation summary (rolling context)
    /// * `recent_messages` - Last N messages for immediate context
    ///
    /// # Returns
    /// * `Ok(session_id)` - Successfully spawned agent
    /// * `Err(SessionAlreadyExists)` - Agent already spawned for this conversation
    /// * `Err(AgentError)` - AgentManager spawn failed
    /// * `Err(DatabaseError)` - Failed to update conversation record
    ///
    /// # Errors
    /// Returns error if session already exists or spawn fails
    pub async fn spawn_agent(
        &self,
        conversation_id: &str,
        template: &AgentTemplate,
        summary: &str,
        recent_messages: &[Message],
    ) -> Result<String, AgentManagerError> {
        // Check if session already exists
        let sessions = self.active_sessions.lock().await;
        if sessions.contains_key(conversation_id) {
            return Err(AgentManagerError::SessionAlreadyExists(
                conversation_id.to_string(),
            ));
        }
        drop(sessions);

        // Build conversation context from summary + recent messages
        let context = Self::build_conversation_context(summary, recent_messages);

        // Combine template system prompt with conversation context
        let full_system_prompt = format!(
            "{}\n\n# CONVERSATION CONTEXT\n\n{}",
            template.system_prompt, context
        );

        // Map AgentModel enum to string for AgentManager
        let model_str = Self::model_to_string(&template.model);

        // Build spawn request
        let request = SpawnSessionRequest {
            prompt: "Continue the conversation".to_string(), // Initial prompt
            system_prompt: Some(full_system_prompt),
            allowed_tools: vec![],    // Default: all tools allowed
            disallowed_tools: vec![], // Default: no restrictions
            max_turns: template.max_turns,
            model: Some(model_str),
            cwd: None,        // Not used for chat agents
            add_dirs: vec![], // Not used for chat agents
            label: format!("conv-{}", conversation_id),
        };

        // Spawn session via AgentManager
        let session_id = self.agent_manager.spawn_session(request).await?;

        // Store mapping in memory
        let mut sessions = self.active_sessions.lock().await;
        sessions.insert(conversation_id.to_string(), session_id.clone());
        drop(sessions);

        // Update database conversation.agent_session_id
        self.update_conversation_session_id(conversation_id, &session_id)
            .await?;

        Ok(session_id)
    }

    /// Send user message to agent in conversation
    ///
    /// Looks up session_id from conversation_id and forwards to AgentManager.
    ///
    /// # Arguments
    /// * `conversation_id` - Database conversation ID
    /// * `message` - User message content
    ///
    /// # Returns
    /// * `Ok(())` - Message sent successfully
    /// * `Err(SessionNotFound)` - No agent spawned for conversation
    /// * `Err(AgentError)` - AgentManager send failed
    pub async fn send_message(
        &self,
        conversation_id: &str,
        message: &str,
    ) -> Result<(), AgentManagerError> {
        // Look up session_id
        let sessions = self.active_sessions.lock().await;
        let session_id = sessions
            .get(conversation_id)
            .ok_or_else(|| AgentManagerError::SessionNotFound(conversation_id.to_string()))?
            .clone();
        drop(sessions);

        // Forward to AgentManager
        self.agent_manager
            .send_message(&session_id, message)
            .await?;

        Ok(())
    }

    /// Get agent session output for conversation
    ///
    /// Retrieves paginated messages from agent session. Supports both forward
    /// pagination (offset >= 0) and tail mode (offset < 0).
    ///
    /// # Arguments
    /// * `conversation_id` - Database conversation ID
    /// * `offset` - Starting position (>= 0) or tail count (< 0)
    /// * `length` - Maximum messages to return
    ///
    /// # Returns
    /// * `Ok(GetOutputResponse)` - Messages with pagination metadata
    /// * `Err(SessionNotFound)` - No agent spawned for conversation
    /// * `Err(AgentError)` - AgentManager get_output failed
    ///
    /// # Examples
    /// ```
    /// // Get first 100 messages
    /// get_session_output("conv_123", 0, 100).await?;
    ///
    /// // Get last 20 messages (tail mode)
    /// get_session_output("conv_123", -20, 0).await?;
    ///
    /// // Get next page starting at offset 100
    /// get_session_output("conv_123", 100, 50).await?;
    /// ```
    pub async fn get_session_output(
        &self,
        conversation_id: &str,
        offset: i64,
        length: usize,
    ) -> Result<GetOutputResponse, AgentManagerError> {
        // Look up session_id
        let sessions = self.active_sessions.lock().await;
        let session_id = sessions
            .get(conversation_id)
            .ok_or_else(|| AgentManagerError::SessionNotFound(conversation_id.to_string()))?
            .clone();
        drop(sessions);

        // Forward to AgentManager
        let output = self
            .agent_manager
            .get_output(&session_id, offset, length)
            .await?;

        Ok(output)
    }

    /// Terminate agent session for conversation
    ///
    /// Gracefully shuts down agent session, removes from active sessions map,
    /// and updates database. Session will be moved to AgentManager's completed
    /// sessions (retained for 1 minute).
    ///
    /// # Arguments
    /// * `conversation_id` - Database conversation ID
    ///
    /// # Returns
    /// * `Ok(())` - Session terminated successfully
    /// * `Err(SessionNotFound)` - No agent spawned for conversation
    /// * `Err(AgentError)` - AgentManager terminate failed
    pub async fn terminate_session(&self, conversation_id: &str) -> Result<(), AgentManagerError> {
        // Look up and remove session_id
        let mut sessions = self.active_sessions.lock().await;
        let session_id = sessions
            .remove(conversation_id)
            .ok_or_else(|| AgentManagerError::SessionNotFound(conversation_id.to_string()))?;
        drop(sessions);

        // Forward to AgentManager
        self.agent_manager.terminate_session(&session_id).await?;

        Ok(())
    }

    /// Check if agent is actively processing (working status detection)
    ///
    /// Returns true if agent received a message within last 2 seconds
    /// and session is not complete.
    ///
    /// # Arguments
    /// * `conversation_id` - Database conversation ID
    ///
    /// # Returns
    /// * `Ok(true)` - Agent is actively working
    /// * `Ok(false)` - Agent is idle or session complete
    /// * `Err(SessionNotFound)` - No agent spawned for conversation
    pub async fn is_agent_working(&self, conversation_id: &str) -> Result<bool, AgentManagerError> {
        let sessions = self.active_sessions.lock().await;
        let session_id = sessions
            .get(conversation_id)
            .ok_or_else(|| AgentManagerError::SessionNotFound(conversation_id.to_string()))?
            .clone();
        drop(sessions);

        let working = self.agent_manager.is_working(&session_id).await?;

        Ok(working)
    }

    // ========================================================================
    // PRIVATE HELPER METHODS
    // ========================================================================

    /// Build conversation context from summary and recent messages
    ///
    /// Formats summary and last N messages into context string for system prompt.
    /// This provides the agent with immediate conversation context without
    /// needing to load the full message history.
    ///
    /// # Format
    /// ```text
    /// SUMMARY:
    /// <conversation summary>
    ///
    /// RECENT MESSAGES:
    /// Human (2024-01-15 10:30): Hello, how are you?
    /// Agent (2024-01-15 10:31): I'm doing well! How can I help?
    /// Human (2024-01-15 10:32): I need help with Rust
    /// ```
    fn build_conversation_context(summary: &str, recent_messages: &[Message]) -> String {
        let mut context = String::new();

        // Add summary if present
        if !summary.is_empty() {
            context.push_str("SUMMARY:\n");
            context.push_str(summary);
            context.push_str("\n\n");
        }

        // Add recent messages
        if !recent_messages.is_empty() {
            context.push_str("RECENT MESSAGES:\n");
            for msg in recent_messages {
                // Format: "Human (2024-01-15 10:30): Message content"
                let author_label = match msg.author_type {
                    AuthorType::Human => "Human",
                    AuthorType::Agent => "Agent",
                    AuthorType::System => "System",
                    AuthorType::Tool => "Tool",
                };
                let timestamp = msg.timestamp.format("%Y-%m-%d %H:%M");
                context.push_str(&format!(
                    "{} ({}): {}\n",
                    author_label, timestamp, msg.content
                ));
            }
        }

        context
    }

    /// Map AgentModel enum to string for AgentManager
    ///
    /// Converts view model AgentModel enum to lowercase string expected
    /// by kodegen_tools_claude_agent.
    fn model_to_string(model: &AgentModel) -> String {
        match model {
            AgentModel::Sonnet => "sonnet".to_string(),
            AgentModel::Haiku => "haiku".to_string(),
            AgentModel::Opus => "opus".to_string(),
        }
    }

    /// Update conversation.agent_session_id in database
    ///
    /// Stores agent session ID in database for persistence and recovery.
    /// This allows reconstructing the conversation-to-session mapping on restart.
    async fn update_conversation_session_id(
        &self,
        conversation_id: &str,
        session_id: &str,
    ) -> Result<(), AgentManagerError> {
        // Convert &str to String to satisfy 'static lifetime for async
        self.db
            .client()
            .query("UPDATE conversation SET agent_session_id = $session_id WHERE id = $conv_id")
            .bind(("session_id", session_id.to_string()))
            .bind(("conv_id", conversation_id.to_string()))
            .await
            .map_err(|e| AgentManagerError::DatabaseError(e.to_string()))?;

        Ok(())
    }
}
