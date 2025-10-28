//! State management and reducer logic

use super::{
    providers::{ProviderKind, RootTimelineKind},
    types::Action,
};
use crate::{
    PublicAction,
    environment::Environment,
    environment::types::UiConfig,
    view_model::{AccountViewModel, StatusId},
};
use dioxus::prelude::*;

#[derive(Debug, Clone)]
pub struct State {
    pub root_timeline_kind: RootTimelineKind,
    pub root_provider: Option<ProviderKind>,
    pub stack: Vec<AccountViewModel>,
    // Each stack can always have *one* conversation open
    pub current_conversation: Option<StatusId>,
    pub ui_settings: UiConfig,
}

impl State {
    pub fn new(kind: RootTimelineKind) -> Self {
        Self {
            root_timeline_kind: kind,
            root_provider: None,
            stack: Vec::new(),
            current_conversation: None,
            ui_settings: UiConfig::default(),
        }
    }
}

#[allow(dead_code)] // Stack reducer - architectural scaffolding pending integration
pub struct StackReducer;

// Modern Dioxus signal-based state management
pub type StackSignal = Signal<State>;

// Modern Dioxus signal-based action handler
#[allow(dead_code)] // Stack action handler - pending integration
pub fn handle_stack_action(mut signal: StackSignal, action: Action, environment: &Environment) {
    log::trace!("{action:?}");
    match action {
        Action::Initial => {
            signal.with_mut(|state| {
                state.ui_settings = environment.settings.config().unwrap_or_default();
                state.root_provider = Some(
                    state
                        .root_timeline_kind
                        .as_provider(environment, &environment.model),
                );
            });
        }
        Action::PushProfile(p) => {
            signal.with_mut(|state| {
                state.stack.push(p);
            });
        }
        Action::ResolvePushProfile(p) => {
            if crate::helper::parse_user_url(&p).is_ok() {
                // Note: Account resolution will be handled in component with use_future
            }
        }
        Action::CloseCurrent => {
            signal.with_mut(|state| {
                state.stack.pop();
            });
        }
        Action::SelectConversation(c) => {
            signal.with_mut(|state| {
                state.current_conversation = Some(c);
            });
        }
        Action::CloseConversation => {
            signal.with_mut(|state| {
                state.current_conversation = None;
            });
        }
        Action::AppEvent(_) => {
            // Note: Child communication will be handled in component context
        }
        Action::PublicAction(e) => match e {
            PublicAction::Conversation(c) => {
                signal.with_mut(|state| {
                    state.current_conversation = Some(StatusId(c));
                });
            }
            _ => {
                // Note: Parent communication will be handled in component context
            }
        },
        Action::Conversation(_) => {
            // Note: Parent communication will be handled in component context
        }
    }
}
