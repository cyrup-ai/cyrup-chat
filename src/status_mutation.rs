/// Status mutation types for timeline and status updates
/// Comprehensive mutation variants for all status manipulation functionality

#[derive(Debug, Clone, PartialEq, Default)]
pub enum StatusMutation {
    /// Create a new status
    #[default]
    Create,

    /// Update an existing status
    Update,

    /// Delete a status
    Delete,

    /// Like/unlike a status
    Like,
    Unlike,

    /// Repost/share a status
    Repost,

    /// Reply to a status
    Reply,

    /// Pin/unpin a status
    Pin,
    Unpin,

    /// Archive a status
    Archive,

    /// Mark as favorite
    Favorite,
    Unfavorite,

    /// Bookmark a status
    Bookmark(bool),

    /// Alternative spelling for favorite (British English)
    Favourite(bool),

    /// Boost/reblog a status
    Boost(bool),
}

impl StatusMutation {
    /// Create a new status mutation
    pub fn new_status() -> Self {
        StatusMutation::Create
    }

    /// Extract the boolean value from mutation variants that have data
    pub fn get_status_value(&self) -> bool {
        match self {
            StatusMutation::Bookmark(value) => *value,
            StatusMutation::Favourite(value) => *value,
            StatusMutation::Boost(value) => *value,
            _ => true, // Default to true for mutations without explicit boolean state
        }
    }
}
