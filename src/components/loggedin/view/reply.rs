//! Reply component for post composition

use super::super::RootSignal;
use crate::components::post::PostKind;

use crate::view_model::AccountViewModel;
use dioxus::prelude::*;
use std::path::PathBuf;

#[component]
pub fn ReplyComponent(store: RootSignal, kind: PostKind, images: Vec<PathBuf>) -> Element {
    let Some(account) = store.read().user_account.as_ref().cloned() else {
        return rsx! { div {} };
    };

    // Environment context handling with production-safe fallback
    let environment = crate::app::context::use_environment_or_default();

    // we don't need the sender.. we have a build-in one. check how to get rid of it
    use crate::components::post::{PostView, State};
    let account_vm = AccountViewModel::new(&account);
    let state = State::new(account_vm, kind.clone(), false, images.to_vec());
    let post_signal = use_signal(|| state);

    rsx! {
        div {
            class: "reply-window-container",
            div {
                class: "reply-window-child",
                PostView {
                    store: post_signal,
                    environment: environment
                }
            }
        }
    }
}
