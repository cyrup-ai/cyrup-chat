//! Sidebar search component
//!
//! This module provides the search input functionality for the sidebar.

use super::super::reducer::{SidebarAction, SidebarSignal};
use dioxus::prelude::*;

#[component]
pub fn SearchComponent(placeholder: &'static str, store: SidebarSignal) -> Element {
    rsx! {
        div { class: "p-2 pe-3",
            input {
                class: "width-100",
                r#type: "text",
                value: "{store.read().search_term}",
                placeholder: "{placeholder}",
                autocomplete: "off",
                spellcheck: "false",
                oninput: move |evt| {
                    use crate::components::sidebar::handle_action;
                    use crate::app::use_environment;
                    let environment = use_environment();
                    handle_action(store, SidebarAction::Search(evt.value().clone()), &environment.read());
                }
            }
        }
    }
}
