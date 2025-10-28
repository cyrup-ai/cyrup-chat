//! Main PostView component with text area and platform API setup

#![allow(non_snake_case)]

use crate::constants::ui_text;
use crate::widgets::*;
use dioxus::prelude::*;

use super::super::{PostAction, PostSignal, handle_post_action};
use crate::utils::platform_api::PlatformAPI;

#[component]
pub fn PostView(
    store: PostSignal,
    environment: Signal<crate::environment::Environment>,
) -> Element {
    let text = use_signal(|| store.read().text.clone());
    let mut is_posting_class = if store.read().posting {
        "true"
    } else {
        "false"
    };
    let is_dropping_file = store.read().dropping_file;

    if is_dropping_file {
        is_posting_class = "true";
    }

    // Configure text area behavior using platform API
    use crate::utils::{CursorPosition, TextAreaConfig, create_platform_api};

    use_future({
        let store = store;
        let environment = environment;
        move || async move {
            let platform_api = create_platform_api();
            let config = TextAreaConfig {
                auto_focus: true,
                cursor_position: CursorPosition::End,
                select_all_on_focus: false,
                delay_ms: Some(150),
                multiline: true,
                readonly: false,
            };

            if let Err(e) = platform_api.configure_text_area("text-area", config).await {
                log::warn!("Failed to configure text area: {e}");
            }

            // Create channel-based updater for file drop events to avoid Send + Sync issues
            let (tx, mut rx) =
                tokio::sync::mpsc::unbounded_channel::<crate::environment::types::AppEvent>();

            // Spawn task to handle events from channel
            spawn({
                let store = store;
                let environment = environment;
                async move {
                    while let Some(event) = rx.recv().await {
                        handle_post_action(
                            store,
                            crate::components::post::PostAction::AppEvent(event),
                            &environment.read(),
                        );
                    }
                }
            });

            // Create Send + Sync updater that sends to channel
            let updater = std::sync::Arc::new(move |event: crate::environment::types::AppEvent| {
                let _ = tx.send(event);
            });

            // Also set up upload handlers with updater for file drops
            if let Err(e) = platform_api.setup_upload_handlers(updater).await {
                log::warn!("Failed to setup upload handlers: {e}");
            }
        }
    });

    rsx! {
        div {
            class: "posting-window",
            {is_dropping_file.then(|| {
                rsx! {
                    div {
                        class: "fullscreen file-drop-box"
                    }
                }
            })}
            VStack {
                class: "width-100",
                super::toolbar::ToolbarView {
                    store: store,
                    environment: environment
                }
                textarea {
                    id: "text-area",
                    disabled: is_posting_class,
                    placeholder: "{ui_text::post_text_placeholder()}",
                    oninput: move |evt| {
                        handle_post_action(store, PostAction::UpdateText(evt.value().clone()), &environment.read());
                    },
                    autofocus: "true",
                    "{text}"
                }
                {store.read().error_message.as_ref().map(|error| {
                    rsx! {
                        ErrorBox {
                            content: error.to_string(),
                            onclick: move |_| {
                                handle_post_action(store, PostAction::ClearError, &environment.read());
                            }
                        }
                    }
                })}
                {if !store.read().validity.0 {
                    rsx! {
                        Label {
                            class: "char-count over",
                            "{store.read().validity.1} / {store.read().validity.2}"
                        }
                    }
                } else {
                    rsx! {
                        Label {
                            class: "char-count",
                            "{store.read().validity.1} / {store.read().validity.2}"
                        }
                    }
                }}
                super::images::ImagesView { store: store, environment: environment }
            }
        }
    }
}
