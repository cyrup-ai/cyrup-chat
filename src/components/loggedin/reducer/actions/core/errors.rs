//! Error handling types and implementations for loggedin reducer actions

use super::super::{auth, post, selection, status_mutation};

/// Action dispatch errors with detailed context
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ActionError {
    /// Authentication operation failed
    AuthenticationFailed(auth::AuthError),
    /// Selection operation failed
    SelectionFailed(selection::SelectionError),
    /// Post operation failed
    PostFailed(post::PostError),
    /// Status mutation operation failed
    StatusMutationFailed(status_mutation::StatusMutationError),
    /// Navigation operation failed
    NavigationFailed(String),
    /// Settings operation failed
    SettingsFailed(String),
    /// Event handling failed
    EventFailed(String),
    /// Unknown action type
    UnknownAction(String),
}

impl std::fmt::Display for ActionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActionError::AuthenticationFailed(e) => write!(f, "Authentication failed: {e}"),
            ActionError::SelectionFailed(e) => write!(f, "Selection failed: {e}"),
            ActionError::PostFailed(e) => write!(f, "Post operation failed: {e}"),
            ActionError::StatusMutationFailed(e) => write!(f, "Status mutation failed: {e}"),
            ActionError::NavigationFailed(msg) => write!(f, "Navigation failed: {msg}"),
            ActionError::SettingsFailed(msg) => write!(f, "Settings operation failed: {msg}"),
            ActionError::EventFailed(msg) => write!(f, "Event handling failed: {msg}"),
            ActionError::UnknownAction(msg) => write!(f, "Unknown action: {msg}"),
        }
    }
}

impl std::error::Error for ActionError {}

impl From<auth::AuthError> for ActionError {
    fn from(error: auth::AuthError) -> Self {
        ActionError::AuthenticationFailed(error)
    }
}

impl From<selection::SelectionError> for ActionError {
    fn from(error: selection::SelectionError) -> Self {
        ActionError::SelectionFailed(error)
    }
}

impl From<post::PostError> for ActionError {
    fn from(error: post::PostError) -> Self {
        ActionError::PostFailed(error)
    }
}

impl From<status_mutation::StatusMutationError> for ActionError {
    fn from(error: status_mutation::StatusMutationError) -> Self {
        ActionError::StatusMutationFailed(error)
    }
}
