//! Main logged-in application component with environment and event handling

use super::super::reducer::{Action, ReducerState};
use super::layout::MainComponent;
use crate::environment::Environment;
use crate::widgets::ErrorBox;
use dioxus::prelude::*;
use std::sync::Arc;

#[component]
pub fn LoggedInApp(environment: Signal<Environment>, should_show_login: Signal<bool>) -> Element {
    log::trace!("rerender loggedin-app 1");

    // Modern Dioxus 0.7 patterns
    let mut update_signal = use_signal(|| 0u32);
    let (sender, receiver) = use_signal(flume::unbounded::<Action>)();

    // Handle menu events with thread-safe approach
    let sender_for_menu = sender.clone();
    use_effect(move || {
        let environment_read = environment.read();
        let sender_clone = sender_for_menu.clone();
        environment_read
            .platform
            .handle_menu_events(Arc::new(move |a| {
                let _ = sender_clone.send(Action::AppEvent(a));
            }));
    });

    // Handle received actions using modern use_future pattern
    use_future({
        let receiver_for_effect = receiver.clone();
        move || {
            let receiver_clone = receiver_for_effect.clone();
            async move {
                while let Ok(action) = receiver_clone.recv_async().await {
                    log::debug!("Processing action in loggedin view: {:?}", action);

                    // Process the action and trigger UI update
                    match action {
                        Action::AppEvent(app_event) => match app_event {
                            crate::environment::types::AppEvent::MenuEvent(menu_event) => {
                                log::debug!("Menu event in loggedin view: {:?}", menu_event);
                                // Menu events trigger UI refresh
                                update_signal.set(update_signal() + 1);
                            }
                            crate::environment::types::AppEvent::FocusChange(focus) => {
                                log::debug!("Focus change in loggedin view: {:?}", focus);
                                // Focus changes may affect UI state
                                update_signal.set(update_signal() + 1);
                            }
                            crate::environment::types::AppEvent::ClosingWindow => {
                                log::debug!("Window closing in loggedin view");
                                // Window closing events trigger cleanup
                                update_signal.set(update_signal() + 1);
                            }
                            crate::environment::types::AppEvent::FileEvent(_) => {
                                log::debug!("File event in loggedin view");
                                // File events may trigger UI updates
                                update_signal.set(update_signal() + 1);
                            }
                        },
                        _ => {
                            // Handle other Action variants if needed
                            update_signal.set(update_signal() + 1);
                        }
                    }
                }
            }
        }
    });

    let cloned_sender = sender.clone();
    let toolbar_sender = Arc::new(move |action| {
        if let Err(e) = cloned_sender.send(Action::AppEvent(action)) {
            log::error!("Could not send msg: {e:?}");
        }
        // Signal updates will be triggered through the message handling system
        // No direct signal mutation needed in the Fn closure
    });

    use_effect(move || {
        // Set toolbar handler with proper Dioxus 0.7 pattern
        let environment_read = environment.read();
        environment_read
            .platform
            .set_toolbar_handler(toolbar_sender.clone());
    });

    // Replaced navicula root with modern Dioxus Signal pattern
    let state = use_signal(ReducerState::default);

    let is_dropping = state.read().flags.is_dropping;
    let error = state.read().error.clone();

    // Provide environment as context for child components
    use_context_provider(|| environment);
    
    // Provide selected conversation ID as context for chat component
    let selected_conversation_id = use_signal(|| "conversation:default_chat".to_string());
    use_context_provider(|| selected_conversation_id);

    rsx! {
        div {
            MainComponent { store: state }

            if let Some(error) = error {
                div {
                    class: "error-box-bottom",
                    ErrorBox {
                        content: error,
                        onclick: move |_| {
                            // Handle error clearing through proper action pattern
                            use crate::components::loggedin::handle_action;
                            use crate::app::use_environment;
                            let mut environment = use_environment();
                            let _ = handle_action(state, Action::ClearError, &mut environment.write());
                        }
                    }
                }
            }

            if is_dropping {
                div {
                    class: "fullscreen file-drop-box"
                }
            }
        }
    }
}
