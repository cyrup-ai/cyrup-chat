use super::core::{Action, State, TimelineSignal};
use crate::PublicAction;
use crate::environment::Environment;
use crate::environment::types::{AppEvent, MainMenuEvent};
use crate::view_model::{AccountId, AccountVisibility};
use dioxus::prelude::*;
use enumset::EnumSet;

impl State {
    pub fn handle_public_action(
        _signal: TimelineSignal,
        action: PublicAction,
        environment: &mut Environment,
    ) {
        // Signal-based parent communication using proper Dioxus 0.7 patterns
        // Following the global.rs example for cross-component communication

        // Use the proper context-based parent communication pattern
        // Parent components should provide their signal through context
        spawn({
            let environment = environment.clone();
            async move {
                // Handle public action directly through environment platform
                // Q9: MVP hardcoded - no instance URL needed
                let instance_url = "";
                if let Err(e) = environment
                    .platform
                    .handle_public_action(action, instance_url)
                    .await
                {
                    log::error!("Failed to handle public action: {e}");
                }
            }
        });
    }

    pub fn handle_account_visibility(
        mut signal: TimelineSignal,
        account: AccountId,
        visibility: AccountVisibility,
        environment: &mut Environment,
    ) {
        signal.with_mut(|state| {
            let current = state
                .ui_settings
                .visibility
                .entry(account.0)
                .or_insert(EnumSet::all());
            if current.contains(visibility) {
                current.remove(visibility);
            } else {
                current.insert(visibility);
            }

            // Handle async config save with spawn to avoid blocking reducer
            spawn({
                let environment = environment.clone();
                let ui_settings = state.ui_settings.clone();
                async move {
                    if environment
                        .settings
                        .set_config(&ui_settings)
                        .await
                        .is_none()
                    {
                        log::warn!("UI configuration save returned None - may not have been saved");
                    }
                }
            });
        });
    }

    pub fn handle_app_event(
        mut signal: TimelineSignal,
        app_event: AppEvent,
        environment: &mut Environment,
    ) {
        match app_event {
            AppEvent::MenuEvent(m) => match m {
                MainMenuEvent::ScrollDown | MainMenuEvent::ScrollUp => {
                    // Use proper Signal-based UI interactions for menu scroll events
                    let posts = signal.read().posts.clone();
                    if posts.is_empty() {
                        return;
                    }
                    let (id, direction) = match m {
                        MainMenuEvent::ScrollDown => {
                            if let Some(first_post) = posts.first() {
                                (first_post.id.dom_id(), "end")
                            } else {
                                log::warn!("Cannot scroll down: no posts available");
                                return;
                            }
                        }
                        MainMenuEvent::ScrollUp => {
                            if let Some(last_post) = posts.last() {
                                (last_post.id.dom_id(), "start")
                            } else {
                                log::warn!("Cannot scroll up: no posts available");
                                return;
                            }
                        }
                        _ => {
                            log::error!("Unsupported scroll menu event: {m:?}");
                            return;
                        }
                    };
                    log::debug!("Scroll to {id} with direction {direction}");
                    // Use platform API for smooth scrolling instead of direct Effect::ui
                    spawn({
                        let environment = environment.clone();
                        async move {
                            if let Err(e) = environment
                                .platform
                                .scroll_to_element_with_behavior(&id, direction)
                                .await
                            {
                                log::warn!("Failed to scroll to element {id}: {e}");
                            }
                        }
                    });
                }
                MainMenuEvent::Reload => {
                    signal.with_mut(|state| {
                        state.provider.reset();
                        state.posts = Vec::new();
                    });
                    spawn({
                        let signal = signal;
                        let environment = environment.clone();
                        async move {
                            let mut env = environment;
                            super::handle_action(signal, Action::LoadData, &mut env);
                        }
                    });
                }
                _ => {
                    log::debug!("Other menu event: {m:?}");
                }
            },
            _ => {
                log::debug!("Other app event: {app_event:?}");
            }
        }
    }
}
