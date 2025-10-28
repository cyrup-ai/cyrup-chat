// Native platform operation errors
//
// Provides comprehensive error handling for platform-specific operations

use crate::errors::ui::UiError;
use std::fmt;

/// Platform-specific operation errors
#[derive(Debug, Clone)]
pub enum PlatformError {
    /// Element not found by ID
    ElementNotFound(String),
    
    /// Platform API call failed
    ApiCallFailed {
        api: String,
        reason: String,
    },
    
    /// Unsupported operation on current platform
    UnsupportedOperation(String),
    
    /// Text operation failed
    TextOperationFailed {
        operation: String,
        element_id: String,
        reason: String,
    },
    
    /// Focus operation failed
    FocusOperationFailed {
        element_id: String,
        reason: String,
    },
    
    /// Drag-drop operation failed
    DragDropOperationFailed(String),
    
    /// Configuration error
    ConfigurationError(String),
}

impl fmt::Display for PlatformError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PlatformError::ElementNotFound(id) => {
                write!(f, "UI element '{id}' not found")
            }
            PlatformError::ApiCallFailed { api, reason } => {
                write!(f, "Platform API '{api}' failed: {reason}")
            }
            PlatformError::UnsupportedOperation(op) => {
                write!(f, "Operation '{op}' not supported on this platform")
            }
            PlatformError::TextOperationFailed { operation, element_id, reason } => {
                write!(f, "Text operation '{operation}' failed on element '{element_id}': {reason}")
            }
            PlatformError::FocusOperationFailed { element_id, reason } => {
                write!(f, "Focus operation failed on element '{element_id}': {reason}")
            }
            PlatformError::DragDropOperationFailed(reason) => {
                write!(f, "Drag-drop operation failed: {reason}")
            }
            PlatformError::ConfigurationError(reason) => {
                write!(f, "Platform configuration error: {reason}")
            }
        }
    }
}

impl std::error::Error for PlatformError {}

impl From<PlatformError> for UiError {
    fn from(err: PlatformError) -> Self {
        UiError::PlatformError(err.to_string())
    }
}

impl PlatformError {
    /// Create element not found error
    pub fn element_not_found(element_id: impl Into<String>) -> Self {
        Self::ElementNotFound(element_id.into())
    }
    
    /// Create API call failed error
    pub fn api_call_failed(api: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::ApiCallFailed {
            api: api.into(),
            reason: reason.into(),
        }
    }
    
    /// Create unsupported operation error
    pub fn unsupported_operation(operation: impl Into<String>) -> Self {
        Self::UnsupportedOperation(operation.into())
    }
    
    /// Create text operation failed error
    pub fn text_operation_failed(
        operation: impl Into<String>,
        element_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::TextOperationFailed {
            operation: operation.into(),
            element_id: element_id.into(),
            reason: reason.into(),
        }
    }
    
    /// Create focus operation failed error
    pub fn focus_operation_failed(
        element_id: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::FocusOperationFailed {
            element_id: element_id.into(),
            reason: reason.into(),
        }
    }
    
    /// Create drag-drop operation failed error
    pub fn drag_drop_operation_failed(reason: impl Into<String>) -> Self {
        Self::DragDropOperationFailed(reason.into())
    }
    
    /// Create configuration error
    pub fn configuration_error(reason: impl Into<String>) -> Self {
        Self::ConfigurationError(reason.into())
    }
}