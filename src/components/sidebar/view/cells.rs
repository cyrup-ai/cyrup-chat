//! Sidebar cell components
//!
//! This module provides the cell components for displaying accounts
//! and account updates in the sidebar.

use super::super::reducer::{SidebarAction, SidebarSignal};
use crate::{
    PublicAction, components::loggedin::Action, environment::model::Account, loc,
    view_model::AccountUpdateViewModel, widgets::*,
};
use dioxus::prelude::*;

#[component]
pub fn CellComponent(
    model: AccountUpdateViewModel,
    selected: bool,
    store: SidebarSignal,
    onclick: EventHandler<()>,
    favorited: bool,
) -> Element {
    let class = if selected {
        "p-4 pl-3 ml-1 rounded-md select-none cursor-pointer selection-macos text-[var(--g-textColor)]"
    } else {
        "p-4 pl-3 ml-1 rounded-md select-none cursor-pointer"
    };
    let favorite_title = loc!("Favorite Account. Always at the top");
    let favorite_icon = favorited.then(|| {
        rsx!(div {
            class: "favorite-icon",
            Icon {
                icon: crate::icons::ICON_LIKE_ACCOUNT,
                title: "{favorite_title}",
            }
        })
    });
    rsx! {
        div {
            class: "{class} flex-grow",
            onclick: move |_| onclick.call(()),
            oncontextmenu: move |e| {
                // Context menu for cell component with Dioxus 0.7 patterns
                use crate::components::sidebar::handle_action;
                use crate::app::use_environment;

                e.prevent_default(); // Prevent browser context menu
                let environment = use_environment();
                let menu_items = vec![
                    ("Select Account", SidebarAction::SelectAccount(model.account.clone())),
                    ("Open in Browser", SidebarAction::Root(Box::new(Action::Public(PublicAction::OpenLink(model.account.url.clone()))))),
                    ("Copy URL", SidebarAction::Root(Box::new(Action::Public(PublicAction::Copy(model.account.url.clone()))))),
                ];

                // Show context menu through environment platform
                environment.read().platform.show_context_menu(
                    (e.client_coordinates().x as i32, e.client_coordinates().y as i32),
                    "Account Actions",
                    menu_items,
                    move |action| {
                        handle_action(store, action, &environment.read());
                    }
                );
            },
            HStack { class: "gap-2 flex-grow",
                VStack { class: "items-center flex-shrink-0 noclip",
                    img {
                        class: "rounded-lg w-[42px] h-[42px]",
                        src: "{model.account.image}",
                        alt: "{model.account.display_name}",
                        width: 42,
                        height: 42
                    }
                    {favorite_icon}
                }
                VStack { class: "gap-1 flex-grow account-preview-fields",
                    HStack { class: "justify-between flex-wrap h-[1.3em] overflow-y-hidden",
                        Label {
                            onclick: move |_| onclick.call(()),
                            style: TextStyle::Primary,
                            class: "mr-auto",
                            pointer_style: PointerStyle::Pointer,
                            "{model.account.username}"
                        }
                        Label { style: TextStyle::Secondary, pointer_style: PointerStyle::Pointer, "{model.last_updated_human}" }
                    }
                    Paragraph {
                        class: "status-content",
                        style: TextStyle::Tertiary,
                        pointer_style: PointerStyle::Pointer,
                        "{model.content}"
                    }
                }
            }
        }
    }
}

#[component]
pub fn AccountCellComponent(
    store: SidebarSignal,
    account: Account,
    selected: bool,
    onclick: EventHandler<()>,
) -> Element {
    let class = if selected {
        "p-4 pl-3 ml-1 rounded-md select-none cursor-pointer selection-macos text-[var(--g-textColor)]"
    } else {
        "p-4 pl-3 ml-1 rounded-md select-none cursor-pointer"
    };
    rsx! {
        div {
            class: "{class} flex-grow",
            onclick: move |_| onclick.call(()),
            oncontextmenu: move |e| {
                // Context menu for account cell with proper Dioxus 0.7 patterns
                use crate::components::sidebar::handle_action;
                use crate::app::use_environment;

                e.prevent_default(); // Prevent browser context menu
                let environment = use_environment();
                let menu_items = vec![
                    ("Open in Browser", SidebarAction::Root(Box::new(Action::Public(PublicAction::OpenLink(account.url.clone()))))),
                    ("Copy URL", SidebarAction::Root(Box::new(Action::Public(PublicAction::Copy(account.url.clone()))))),
                    ("Copy Account Name", SidebarAction::Root(Box::new(Action::Public(PublicAction::Copy(account.acct.clone()))))),
                ];

                // Show context menu through environment platform
                environment.read().platform.show_context_menu(
                    (e.client_coordinates().x as i32, e.client_coordinates().y as i32),
                    "Account",
                    menu_items,
                    move |action| {
                        handle_action(store, action, &environment.read());
                    }
                );
            },
            HStack { class: "gap-2 flex-grow",
                img {
                    class: "rounded-lg w-8 h-8",
                    src: "{account.avatar_static}",
                    alt: "{account.display_name}",
                    width: 32,
                    height: 32
                }
                VStack { class: "gap-1 flex-grow account-preview-fields",
                    HStack { class: "justify-between flex-wrap h-[1.3em] overflow-y-hidden",
                        Label {
                            onclick: move |_| onclick.call(()),
                            style: TextStyle::Primary,
                            class: "mr-auto",
                            pointer_style: PointerStyle::Pointer,
                            "{account.username}"
                        }
                    }
                    Paragraph {
                        class: "status-content",
                        style: TextStyle::Tertiary,
                        pointer_style: PointerStyle::Pointer,
                        "{account.display_name}"
                    }
                }
            }
        }
    }
}
