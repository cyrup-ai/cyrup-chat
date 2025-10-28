use crate::components::component_stack::RootTimelineKind;
// profiles removed in AGENT_17 - profiles feature deleted
use crate::components::sidebar::MoreSelection;
use crate::environment::Environment;
use crate::environment::model::Account;
use crate::environment::types::{AppEvent, UiConfig};
use crate::view_model::AccountViewModel;
use dioxus::prelude::*;

// Modern Dioxus signal-based state management
pub type MoreSignal = Signal<State>;

#[derive(Clone, Debug, Default)]
#[allow(dead_code)] // More providers structure - pending integration
pub struct Providers {
    pub classic_timeline: Option<RootTimelineKind>,
    pub bookmarks: Option<RootTimelineKind>,
    pub favorites: Option<RootTimelineKind>,
    pub account: Option<RootTimelineKind>,
    pub local: Option<RootTimelineKind>,
    pub public: Option<RootTimelineKind>,
    pub follows: Option<RootTimelineKind>,
    pub following: Option<RootTimelineKind>,
    pub posts: Option<RootTimelineKind>,
    pub hashtags: Option<RootTimelineKind>,
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // More messages - architectural scaffolding pending integration
pub enum Message {
    AppEvent(AppEvent),
    Selection(MoreSelection, bool),
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // Public actions - architectural scaffolding pending integration
pub enum PublicAction {
    Timeline(crate::PublicAction),
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // More actions - architectural scaffolding pending integration
pub enum Action {
    Initial,
    Selection(MoreSelection),
    AppEvent(AppEvent),
    Conversation(Box<crate::PublicAction>),
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // More state structure - pending integration
pub struct State {
    pub selection: MoreSelection,
    pub providers: Providers,
    pub ui_settings: UiConfig,
    pub user_account: Account,
}

impl State {
    #[allow(dead_code)] // More state constructor - pending integration
    pub fn new(selection: MoreSelection, user_account: Account) -> Self {
        Self {
            selection,
            // selected_conversation,
            providers: Providers::default(),
            ui_settings: Default::default(),
            user_account,
        }
    }
}

#[allow(dead_code)] // More action handler - pending integration
pub fn handle_action(mut signal: MoreSignal, action: Action, environment: &Environment) {
    log::trace!("{action:?}");

    signal.with_mut(|state| {
        match action {
            Action::Initial => {
                state.ui_settings = environment.settings.config().unwrap_or_default();
                handle_selection(
                    state.selection,
                    &mut state.providers,
                    AccountViewModel::new(&state.user_account),
                );
            }
            Action::Selection(a) => {
                let re_selected = state.selection == a;
                state.selection = a;

                // Optimize by skipping redundant selection processing
                if !re_selected {
                    handle_selection(
                        state.selection,
                        &mut state.providers,
                        AccountViewModel::new(&state.user_account),
                    );
                }
                // Modern Dioxus: Use component-level message handling instead of context.send_children
                // Component will handle Message::Selection via use_effect
            }
            Action::AppEvent(app_event) => {
                // Process app events through modern Dioxus component messaging
                log::debug!("Processing app event: {:?}", app_event);

                // Handle app events based on type
                match app_event {
                    crate::environment::types::AppEvent::MenuEvent(menu_event) => {
                        log::debug!("Menu event received: {:?}", menu_event);
                        // Menu events are handled by parent components
                    }
                    crate::environment::types::AppEvent::FocusChange(focus) => {
                        log::debug!("Focus change: {:?}", focus);
                        // Focus changes are handled by window management
                    }
                    crate::environment::types::AppEvent::FileEvent(_) => {
                        log::debug!("File event received in more component");
                        // File events are handled by appropriate file handling components
                    }
                    crate::environment::types::AppEvent::ClosingWindow => {
                        log::debug!("Window closing event received");
                        // Window closing is handled by platform layer
                    }
                }
                // Component will handle Message::AppEvent via use_effect
            }
            Action::Conversation(_a) => {
                // Modern Dioxus: Use component-level message handling instead of context.send_parent
                // Component will handle PublicAction::Timeline via use_effect
            }
        }
    });
}

/// Create a new timeline provider or get an existing one
#[allow(dead_code)] // More selection handler - pending integration
fn handle_selection(
    selection: MoreSelection,
    providers: &mut Providers,
    account: AccountViewModel,
) {
    match selection {
        // Most timeline variants deleted in AGENT_17 - only Bookmarks and ConversationList remain
        MoreSelection::Classic => {
            providers.classic_timeline = Some(RootTimelineKind::ConversationList(account))
        }
        MoreSelection::Yours => {
            providers.account = Some(RootTimelineKind::ConversationList(account))
        }
        MoreSelection::Bookmarks => {
            providers.bookmarks = Some(RootTimelineKind::Bookmarks(account))
        }
        MoreSelection::Favorites => {
            // Favorites deleted - map to ConversationList
            providers.favorites = Some(RootTimelineKind::ConversationList(account))
        }
        MoreSelection::Federated => {
            // Federated deleted - map to ConversationList
            providers.public = Some(RootTimelineKind::ConversationList(account))
        }
        MoreSelection::Local => {
            // Local deleted - map to ConversationList
            providers.local = Some(RootTimelineKind::ConversationList(account))
        }
        MoreSelection::Followers => {
            // Profiles/Relationship deleted - map to ConversationList
            providers.follows = Some(RootTimelineKind::ConversationList(account))
        }
        MoreSelection::Following => {
            // Profiles/Relationship deleted - map to ConversationList
            providers.following = Some(RootTimelineKind::ConversationList(account))
        }
        MoreSelection::Posts => {
            // Account timeline deleted - map to ConversationList
            providers.posts = Some(RootTimelineKind::ConversationList(account))
        }
        MoreSelection::Hashtags => {
            // Hashtags not implemented - map to ConversationList
            providers.hashtags = Some(RootTimelineKind::ConversationList(account))
        }
    }
}
