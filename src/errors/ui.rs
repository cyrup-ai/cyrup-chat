// UI Error Types - Production Implementation

/// UI-related errors
#[derive(Debug, thiserror::Error)]
pub enum UiError {
    #[error("Component render failed: {component}")]
    RenderFailed { component: String },

    #[error("Invalid user input: {field}")]
    InvalidInput { field: String },

    #[error("Component communication failed: {reason}")]
    ComponentCommunicationFailed { reason: String },

    #[error("State update failed: {reason}")]
    StateUpdateFailed { reason: String },

    #[error("Resource not found: {resource}")]
    ResourceNotFound { resource: String },

    #[error("Platform operation failed: {message}")]
    PlatformError { message: String },
}

impl UiError {
    /// Create a new render failed error
    #[inline]
    pub fn render_failed(component: impl Into<String>) -> Self {
        Self::RenderFailed {
            component: component.into(),
        }
    }

    /// Create a new invalid input error
    #[inline]
    pub fn invalid_input(field: impl Into<String>) -> Self {
        Self::InvalidInput {
            field: field.into(),
        }
    }

    /// Create a new platform error
    #[inline]
    pub fn platform_error(message: impl Into<String>) -> Self {
        Self::PlatformError {
            message: message.into(),
        }
    }
}
