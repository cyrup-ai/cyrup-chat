//! Selection error types with detailed context for debugging

/// Selection error types with detailed context for debugging
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum SelectionError {
    /// Account selection failed with detailed error message
    AccountSelection(String),
    /// Notification selection failed with detailed error message
    NotificationSelection(String),
    /// Conversation selection failed with detailed error message
    ConversationSelection(String),
    /// More menu selection failed with detailed error message
    MoreSelection(String),
    /// Navigation state validation failed
    NavigationState(String),
}

impl std::fmt::Display for SelectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SelectionError::AccountSelection(msg) => {
                write!(f, "Account selection failed: {msg}")
            }
            SelectionError::NotificationSelection(msg) => {
                write!(f, "Notification selection failed: {msg}")
            }
            SelectionError::ConversationSelection(msg) => {
                write!(f, "Conversation selection failed: {msg}")
            }
            SelectionError::MoreSelection(msg) => write!(f, "More selection failed: {msg}"),
            SelectionError::NavigationState(msg) => {
                write!(f, "Navigation state failed: {msg}")
            }
        }
    }
}

impl std::error::Error for SelectionError {}
