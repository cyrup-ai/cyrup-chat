//! Sidebar menu component
//!
//! This module provides the top menu bar for the sidebar with user account
//! display, reload button, and compose button.

use super::super::reducer::{SidebarAction, SidebarSignal};
use crate::{
    PublicAction,
    components::{loggedin::Action, post::PostKind},
    environment::types::{AppEvent, MainMenuEvent},
    widgets::*,
};
use dioxus::prelude::*;

#[component]
pub fn MenuComponent(store: SidebarSignal) -> Element {
    // Extract values and clone them to avoid temporary references
    let (username, user_account) = {
        let store_value = store.read();
        let username = store_value
            .user_account
            .as_ref()
            .map(|e| e.username.clone())
            .unwrap_or_default();
        let user_account = store_value.user_account.clone();
        (username, user_account)
    };

    rsx! {
        HStack { class: "justify-between justify-items-center p-2 ml-2 no-selection",
            div {
                title: "{username}",
                {user_account.clone().map(|account| {
                    let account_for_menu = account.clone();
                    rsx! {
                        img {
                            onclick: move |evt| {
                            // Context menu handling with proper Dioxus 0.7 patterns
                            use crate::components::sidebar::handle_action;
                            use crate::app::use_environment;

                            // For right-click, show context menu through platform API
                            if evt.trigger_button() == Some(dioxus::html::input_data::MouseButton::Secondary) {
                                let environment = use_environment();
                                let menu_items = vec![
                                    ("Open in Browser", SidebarAction::Root(Box::new(Action::Public(PublicAction::OpenLink(account_for_menu.url.clone()))))),
                                    ("Copy URL", SidebarAction::Root(Box::new(Action::Public(PublicAction::Copy(account_for_menu.url.clone()))))),
                                    ("Logout", SidebarAction::Root(Box::new(Action::Logout))),
                                ];

                                // Show context menu through environment platform
                                environment.read().platform.show_context_menu(
                                    (evt.client_coordinates().x as i32, evt.client_coordinates().y as i32),
                                    "Account Options",
                                    menu_items,
                                    move |action| {
                                        handle_action(store, action, &environment.read());
                                    }
                                );
                            }
                        },
                        class: "rounded w-[22px] h-[22px] self-center",
                        src: "{account.avatar_static}"
                    }
                    }
                }).unwrap_or_else(|| rsx! {
                    span {
                        style: "display: inline-block",
                        class: "rounded w-[22px] h-[22px] self-center"
                    }
                })}
            }
            div { class: "icon-button ml-auto mr-2", button {
                r#type: "button",
                onclick: move |_| {
                    use crate::components::sidebar::handle_action;
                    use crate::app::use_environment;
                    let environment = use_environment();
                    handle_action(store, SidebarAction::Root(Box::new(Action::AppEvent(AppEvent::MenuEvent(MainMenuEvent::Reload)))), &environment.read());
                },
                dangerous_inner_html: crate::icons::ICON_RELOAD
            } }
            div { class: "icon-button",
                button {
                    r#type: "button",
                    onclick: move |_| {
                        use crate::components::sidebar::handle_action;
                        use crate::app::use_environment;
                        let environment = use_environment();
                        handle_action(store, SidebarAction::Root(Box::new(Action::Post(PostKind::Post))), &environment.read());
                    },
                    dangerous_inner_html: crate::icons::ICON_WRITE
                }
            }
        }
    }
}
