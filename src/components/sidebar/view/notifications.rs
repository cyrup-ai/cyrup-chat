//! Sidebar notifications component
//!
//! This module provides the notifications view for the sidebar with
//! notification account display and loading functionality.

use super::super::reducer::{SidebarAction, SidebarSignal};
use crate::widgets::*;
use dioxus::prelude::*;

use super::CellComponent;

#[component]
pub fn SidebarNotificationsComponent(store: SidebarSignal) -> Element {
    let selection = &store.read().selected_notifications;
    let is_loading = store.read().loading_notifications;

    let notification_accounts = store.read().notification_accounts.clone();
    let load_more_button = (!is_loading && !store.read().notification_posts_empty).then(|| {
        rsx!(div {
            class: "flex flex-row relative justify-center mt-2",
            IconTextButton {
                icon: crate::icons::ICON_LOAD_OLDER_TIMELINE,
                text: "More",
                title: "Load more notification data",
                class: "mb-4",
                onclick: move |_| {
                    use crate::components::sidebar::handle_action;
                    use crate::app::use_environment;
                    let environment = use_environment();
                    handle_action(store, SidebarAction::LoadNotifications, &environment.read());
                }
            }
        })
    });

    rsx! {
        div { class: "scroll",
            div { class: "scroll-margin-fix",
                for model in notification_accounts.clone() {
                    CellComponent {
                        model: model.clone(),
                        store: store,
                        selected: selection.as_ref() == Some(&model.account),
                        onclick: move |_| {
                            use crate::components::sidebar::handle_action;
                            use crate::app::use_environment;
                            let environment = use_environment();
                            handle_action(store, SidebarAction::SelectedNotifications(model.account.clone()), &environment.read());
                        },
                        favorited: false
                    }
                },

                {load_more_button}
            }
        }

        { is_loading.then(|| rsx!(div {
            class: "flex flex-row relative p-2 m-2 flex-grow self-center",
            Spinner {}
        }))}
    }
}
