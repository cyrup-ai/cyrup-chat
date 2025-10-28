//! Stack entry components

use super::state::StackSignal;
// Profile component removed in AGENT_17 - profiles deleted
use crate::{icons::ICON_CANCEL, loc, view_model::AccountViewModel, widgets::*};
use dioxus::prelude::*;

#[component]
pub fn StackEntry(store: StackSignal, profile: AccountViewModel) -> Element {
    // only animate the stack the first time it appears
    let mut first_state = use_signal(|| false);
    let is_first_time = !first_state();
    if is_first_time {
        first_state.set(true);
    }
    let cls = if is_first_time {
        "slide-in-blurred-bottom"
    } else {
        ""
    };
    rsx! {
        div {
            class: "popup {cls}",
            ProfileStackEntry {
                store: store,
                profile: profile
            }
        }
    }
}

#[component]
fn ProfileStackEntry(store: StackSignal, profile: AccountViewModel) -> Element {
    rsx! {
        VStack {
            class: "height-100",
            div {
                class: "flex flex-row relative toolbar mr-auto",
                style: "margin: 0px; margin-bottom: 16px; align-items: baseline;",
                IconButton {
                    icon: ICON_CANCEL,
                    title: loc!("Close"),
                    onclick: move |_| {
                        store.with_mut(|state| {
                            state.stack.pop();
                        });
                    }
                }
                Label {
                    "{profile.username}"
                }
            }
            InnerProfileStackEntry {
                _store: store,
                _profile: profile
            }
        }
    }
}

#[component]
fn InnerProfileStackEntry(_store: StackSignal, _profile: AccountViewModel) -> Element {
    // Profile component removed in AGENT_17 - profiles feature deleted
    // Templates replace user profiles (Q45/Q46)
    rsx! {
        div {
            class: "placeholder",
            "Profile view not implemented"
        }
    }
}
