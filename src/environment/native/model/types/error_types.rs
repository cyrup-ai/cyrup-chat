//! Error types and handling for model operations

/// Error type for Model creation failures
#[derive(Debug)]
pub enum ModelError {
    /// Failed to create Megalodon client
    ClientCreationFailed(String),
    /// Failed to register application
    RegistrationFailed(String),
    /// Authentication operation failed
    AuthenticationFailed(String),
    /// Network operation failed
    NetworkFailed(String),
    /// Streaming operation failed
    StreamFailed(String),
    /// Agent spawn operation failed
    AgentSpawnFailed(String),
    /// Agent send message operation failed
    AgentSendFailed(String),
    /// Agent terminate operation failed
    AgentTerminateFailed(String),
    /// Database query operation failed
    QueryFailed(String),
    /// Conversation not found
    ConversationNotFound(String),
    /// Agent session error
    AgentSessionError(String),
    /// Feature not implemented (MVP limitation)
    NotImplemented(String),
}

impl ModelError {
    /// Create a client creation error with enhanced context
    #[inline(always)]
    pub fn client_creation_failed(msg: impl Into<String>) -> Self {
        Self::ClientCreationFailed(msg.into())
    }

    /// Create a registration failed error with enhanced context
    #[inline(always)]
    pub fn registration_failed(msg: impl Into<String>) -> Self {
        Self::RegistrationFailed(msg.into())
    }

    /// Create an authentication failed error with enhanced context
    #[inline(always)]
    pub fn authentication_failed(msg: impl Into<String>) -> Self {
        Self::AuthenticationFailed(msg.into())
    }

    /// Create a network failed error with enhanced context
    #[inline(always)]
    pub fn network_failed(msg: impl Into<String>) -> Self {
        Self::NetworkFailed(msg.into())
    }

    /// Create a stream failed error with enhanced context
    #[inline(always)]
    pub fn stream_failed(msg: impl Into<String>) -> Self {
        Self::StreamFailed(msg.into())
    }

    /// Create an agent spawn failed error with enhanced context
    #[inline(always)]
    pub fn agent_spawn_failed(msg: impl Into<String>) -> Self {
        Self::AgentSpawnFailed(msg.into())
    }

    /// Create an agent send failed error with enhanced context
    #[inline(always)]
    pub fn agent_send_failed(msg: impl Into<String>) -> Self {
        Self::AgentSendFailed(msg.into())
    }

    /// Create an agent terminate failed error with enhanced context
    #[inline(always)]
    pub fn agent_terminate_failed(msg: impl Into<String>) -> Self {
        Self::AgentTerminateFailed(msg.into())
    }

    /// Create a query failed error with enhanced context
    #[inline(always)]
    pub fn query_failed(msg: impl Into<String>) -> Self {
        Self::QueryFailed(msg.into())
    }

    /// Create a conversation not found error with enhanced context
    #[inline(always)]
    pub fn conversation_not_found(msg: impl Into<String>) -> Self {
        Self::ConversationNotFound(msg.into())
    }
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientCreationFailed(msg) => {
                write!(f, "Failed to create Megalodon client: {msg}")
            }
            Self::RegistrationFailed(msg) => {
                write!(f, "Failed to register application: {msg}")
            }
            Self::AuthenticationFailed(msg) => {
                write!(f, "Authentication failed: {msg}")
            }
            Self::NetworkFailed(msg) => {
                write!(f, "Network operation failed: {msg}")
            }
            Self::StreamFailed(msg) => {
                write!(f, "Streaming operation failed: {msg}")
            }
            Self::AgentSpawnFailed(msg) => {
                write!(f, "Agent spawn failed: {msg}")
            }
            Self::AgentSendFailed(msg) => {
                write!(f, "Agent send failed: {msg}")
            }
            Self::AgentTerminateFailed(msg) => {
                write!(f, "Agent terminate failed: {msg}")
            }
            Self::QueryFailed(msg) => {
                write!(f, "Database query failed: {msg}")
            }
            Self::ConversationNotFound(msg) => {
                write!(f, "Conversation not found: {msg}")
            }
            Self::AgentSessionError(msg) => {
                write!(f, "Agent session error: {msg}")
            }
            Self::NotImplemented(msg) => {
                write!(f, "Feature not implemented: {msg}")
            }
        }
    }
}

impl std::error::Error for ModelError {}

impl From<ModelError> for String {
    fn from(error: ModelError) -> Self {
        error.to_string()
    }
}

/// Extension trait for Result types to provide consistent error formatting
pub trait ResultExt {
    type Output;
    fn string_error(self, call: &'static str) -> Result<Self::Output, String>;
}

impl<T, E: std::fmt::Debug> ResultExt for Result<T, E> {
    type Output = T;

    #[inline(always)]
    fn string_error(self, call: &'static str) -> Result<T, String> {
        self.map_err(|e| {
            let string_error = format!("API Error: {call} {e:?}");
            log::error!("{string_error}");
            string_error
        })
    }
}
