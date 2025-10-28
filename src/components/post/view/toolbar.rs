//! Toolbar component with visibility settings and actions

#![allow(non_snake_case)]

use crate::{loc, widgets::*};
use dioxus::prelude::*;

use super::super::{PostAction, PostSignal, handle_post_action};

#[component]
pub fn ToolbarView(
    store: PostSignal,
    environment: Signal<crate::environment::Environment>,
) -> Element {
    let mut is_posting_class = if store.read().posting {
        "true"
    } else {
        "false"
    };

    // can't post if there're still images being uploaded
    let are_images_uploading = store.read().images.iter().any(|i| i.server_id.is_none());
    if are_images_uploading {
        is_posting_class = "true";
    }

    let current = store
        .read()
        .visibility
        .unwrap_or(super::super::Visibility::Public);
    let is_direct = current == super::super::Visibility::Direct;
    let cancel_text = loc!("Cancel");
    let toot_text = loc!("Toot");

    let cancel_button = (!store.read().is_window).then(|| {
        rsx! {
            button {
                class: "button me-2",
                onclick: move |_| handle_post_action(store, PostAction::Close, &environment.read()),
                "{cancel_text}"
            }
        }
    });

    let posting_spinner = store.read().posting.then(|| {
        rsx! { Spinner {} }
    });

    rsx! {
        HStack {
            class: "p-1 justify-between items-center posting-toolbar",
            {cancel_button}
            EmojiButton {}
            {posting_spinner}
            span { class: "mr-auto" }
            select {
                name: "visibility",
                class: "mr-4",
                onchange: move |evt| {
                    handle_post_action(store, PostAction::UpdateVisibility(evt.value().to_string()), &environment.read());
                },
                option {
                    value: "public",
                    "Public: Visible for all"
                }
                option {
                    value: "unlisted",
                    "Unlisted"
                }
                option {
                    value: "private",
                    "Followers only"
                }
                option {
                    selected: is_direct,
                    value: "direct",
                    "Mentioned people only"
                }
            }
            button {
                class: "button me-3",
                disabled: is_posting_class,
                r#type: "file",
                onclick: move |_| handle_post_action(store, PostAction::FileDialog, &environment.read()),
                "Attach"
            }
            button {
                class: "button me-2 highlighted",
                disabled: is_posting_class,
                onclick: move |_| handle_post_action(store, PostAction::Post, &environment.read()),
                "{toot_text}"
            }
        }
    }
}
