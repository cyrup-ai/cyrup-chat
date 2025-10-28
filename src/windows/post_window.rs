#![allow(non_snake_case)]

#[allow(dead_code)] // Post window system - pending UI integration
use std::path::PathBuf;
use std::rc::Rc;

use crate::environment::{Environment, types::AppEvent};
use crate::view_model::AccountViewModel;

use crate::components::post::*;

use dioxus::prelude::*;

// Allow opening post as a window as well
use crate::environment::OpenWindowState;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // Post window state - architectural scaffolding pending integration
pub struct PostWindowState {
    kind: PostKind,
    dropped_images: Vec<PathBuf>,
    account: AccountViewModel,
}

impl PostWindowState {
    #[allow(dead_code)] // Post window constructor - pending post window integration
    pub fn new(kind: PostKind, dropped_images: Vec<PathBuf>, account: AccountViewModel) -> Self {
        Self {
            kind,
            dropped_images,
            account,
        }
    }
}

impl OpenWindowState for PostWindowState {
    type Action = PostAction;
    fn window(
        &self,
        environment: &Environment,
        receiver: flume::Receiver<AppEvent>,
        _parent_handler: Rc<dyn Fn(Self::Action)>,
    ) -> Element {
        let kind = self.kind.clone();
        let config = environment.settings.config().ok().unwrap_or_default();

        let state = State {
            account: self.account.clone(),
            kind,
            is_window: true,
            images: Vec::new(),
            image_paths: self.dropped_images.clone(),
            posting: false,
            error_message: None,
            dropping_file: false,
            visibility: None,
            text: String::new(),
            validity: (false, 0, 500),
            config,
        };

        // Modern Dioxus context provider pattern - provide Signal<State> for post component
        use_context_provider(|| Signal::new(state.clone()));

        // Context handling with production-safe fallback
        let store = try_use_context::<Signal<State>>().unwrap_or_else(|| {
            log::error!("Post state context missing - creating fallback state for error recovery");
            // Create fallback state to prevent UI crash
            use_signal(|| {
                use crate::{
                    components::post::PostKind, environment::model::Account,
                    view_model::AccountViewModel,
                };
                let fallback_account = Account::default();
                let account_vm = AccountViewModel::new(&fallback_account);
                State::new(account_vm, PostKind::Post, true, vec![])
            })
        });

        // Handle events with modern Dioxus 0.7 async patterns
        // Use use_future for proper async event handling
        use_future({
            let mut store_handle = store;
            let receiver_handle = receiver.clone();

            move || {
                let value = receiver_handle.clone();
                async move {
                    while let Ok(event) = value.recv_async().await {
                        // Process events using signal mutations
                        store_handle.with_mut(|_state| {
                            // Update state based on the received event
                            match event {
                                AppEvent::MenuEvent(menu_event) => {
                                    log::debug!(
                                        "Received menu event in post window: {:?}",
                                        menu_event
                                    );
                                    // Handle menu events for post window
                                }
                                AppEvent::FileEvent(file_event) => {
                                    log::debug!(
                                        "Received file event in post window: {:?}",
                                        file_event
                                    );
                                    // Handle file drag/drop events
                                }
                                _ => {
                                    log::debug!("Unhandled event in post window: {:?}", event);
                                }
                            }
                        });
                    }
                }
            }
        });

        rsx! {
            PostView {
                store: store,
                environment: Signal::new(environment.clone())
            }
        }
    }
}
