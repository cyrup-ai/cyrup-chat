//! Core types and state definitions for loggedin reducer actions

use crate::components::post::PostKind;
use crate::components::sidebar::MoreSelection;
use crate::environment::model::{Account, Message, Status};
use crate::environment::types::AppEvent;
use crate::view_model::{AccountViewModel, StatusId, StatusViewModel};
use crate::windows::preferences_window::PreferencesChange;
use crate::{PublicAction, StatusMutation};
// use dioxus::prelude::*; // Currently unused - enable when needed
use std::cell::Cell;
use std::path::PathBuf;

/// Consolidated state flags for efficient memory layout
#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct State {
    /// Authentication operation in progress
    pub logging_in: bool,
    /// Main content loading state
    pub loading_content: bool,
    /// Account data loading state
    pub loading_account: bool,
    /// Historical posts loading state
    pub loading_account_history: bool,
    /// Conversation loading state
    pub loading_conversation: bool,
    /// Notifications loading state
    pub loading_notifications: bool,
    /// File drop operation state
    pub is_dropping: bool,
}

/// Main reducer state with optimized memory layout
#[derive(Clone, PartialEq, Debug)]
pub struct ReducerState {
    /// Last processed notification ID for efficient pagination
    pub last_notification_id: Option<String>,
    /// UI configuration settings
    pub ui_settings: crate::environment::types::UiConfig,
    /// Operation state flags
    pub flags: State,
    /// Currently authenticated user account
    pub user_account: Option<crate::environment::model::Account>,
    /// Active UI tab
    pub active_tab: crate::environment::storage::UiTab,
    /// Selected account for timeline viewing
    pub selected_account: Option<AccountViewModel>,
    /// Selected account for notification viewing  
    pub selected_notifications: Option<AccountViewModel>,
    /// Current error message for user display
    pub error: Option<String>,
    /// Logout completion state (Cell for interior mutability)
    pub did_logout: Cell<Option<bool>>,
    /// New notification indicator
    pub has_new_notifications: bool,
    /// Authentication status
    pub logged_in: bool,
    /// Active reply composition state
    pub is_replying: Option<(PostKind, Vec<PathBuf>)>,
    /// Current more menu selection
    pub more_selection: MoreSelection,
    /// Current authenticated user (redundant with user_account, kept for compatibility)
    pub current_user: Option<crate::environment::model::Account>,
    /// Batch mutation queue for optimized status operations
    pub mutation_queue: super::super::status_mutation::MutationQueue,
}

impl Default for ReducerState {
    fn default() -> Self {
        Self {
            last_notification_id: None,
            ui_settings: Default::default(),
            flags: Default::default(),
            user_account: None,
            active_tab: Default::default(),
            selected_account: None,
            selected_notifications: None,
            error: None,
            did_logout: Cell::new(None),
            has_new_notifications: false,
            logged_in: false,
            is_replying: None,
            more_selection: Default::default(),
            current_user: None,
            mutation_queue: super::super::status_mutation::MutationQueue::new(10, 500), // 10 mutations max, 500ms flush interval
        }
    }
}

/// Action enumeration for efficient dispatch
#[derive(Clone, Debug)]
#[allow(dead_code)]
pub enum Action {
    // Authentication actions
    Login,
    LoggedIn(Result<Account, String>),
    DataUpdated,

    // Navigation actions
    SelectAccount(AccountViewModel),
    SelectNotifications(AccountViewModel),
    SelectConversation(StatusId),
    SelectMore(MoreSelection),

    // Public actions delegation
    Public(PublicAction),
    StatusMutationResult(Result<Status, String>, StatusViewModel, StatusMutation),

    // Post management actions
    Post(PostKind),
    PostDone(Status),
    PostCancel,

    // Settings actions
    Preferences,
    PreferencesChanged(PreferencesChange),

    // Event handling actions
    AppEvent(AppEvent),
    MessageEvent(Message),

    // Error and cleanup actions
    ClearError,
    Logout,

    // Batch mutation actions
    BatchMutation(Vec<super::super::status_mutation::types::BatchMutation>),
    LogoutDone(Result<(), String>),
}
