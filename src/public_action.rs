use crate::{
    components::post::PostKind, status_mutation::StatusMutation, view_model::AccountViewModel,
};

use crate::view_model::StatusViewModel;

/// Central action enum for application-wide state management
/// Comprehensive action variants for all application functionality

#[derive(Debug, Clone, PartialEq, Default)]
pub enum PublicAction {
    /// Navigate to a different view
    Navigate(String),

    /// User authentication actions
    Login,
    Logout,

    /// Profile related actions
    UpdateProfile,
    ViewProfile(String),
    OpenProfile(AccountViewModel),
    // OpenProfiles deleted in AGENT_17 - profiles feature removed
    OpenProfileLink(String), // Open external profile link

    /// Link and clipboard actions
    OpenLink(String),
    Copy(String),
    OpenTag(String),
    OpenImage(String),
    OpenVideo(String),

    /// Conversation actions
    OpenConversation(String),
    CloseConversation,
    Conversation(String), // Navigate to conversation
    Close,                // Close current view/dialog
    SendMessage(String),

    /// Status and timeline actions
    UpdateStatus,
    RefreshTimeline,
    StatusMutation(StatusMutation, StatusViewModel), // (mutation_type, status)

    /// Post actions (from the code pattern)
    Post(PostKind),

    /// General UI actions
    ShowNotification(String),
    HideNotification,
    ToggleSidebar,

    /// Settings actions
    OpenSettings,
    CloseSettings,

    /// Error handling
    ShowError(String),
    ClearError,

    /// No operation
    #[default]
    NoOp,
}

/// Convert StatusAction + StatusViewModel tuple into PublicAction
/// This handles the common pattern where status-related actions need context
impl From<(crate::widgets::StatusAction, StatusViewModel)> for PublicAction {
    fn from((action, status): (crate::widgets::StatusAction, StatusViewModel)) -> Self {
        use crate::widgets::StatusAction;

        match action {
            StatusAction::Reply => PublicAction::StatusMutation(StatusMutation::Reply, status),
            StatusAction::Boost(enabled) => {
                PublicAction::StatusMutation(StatusMutation::Boost(enabled), status)
            }
            StatusAction::Favorite(enabled) => {
                PublicAction::StatusMutation(StatusMutation::Favourite(enabled), status)
            }
            StatusAction::Bookmark(enabled) => {
                PublicAction::StatusMutation(StatusMutation::Bookmark(enabled), status)
            }
            StatusAction::Clicked => PublicAction::OpenConversation(status.id.0),
            StatusAction::OpenTag(tag) => PublicAction::OpenTag(tag),
            StatusAction::OpenLink(link) => PublicAction::OpenLink(link),
            StatusAction::OpenAccount(account) => PublicAction::OpenLink(account),
            StatusAction::OpenImage(image) => PublicAction::OpenImage(image),
            StatusAction::OpenVideo(video) => PublicAction::OpenVideo(video),
            StatusAction::Copy(text) => PublicAction::Copy(text),
        }
    }
}
