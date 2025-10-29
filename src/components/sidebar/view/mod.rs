//! Sidebar view components - modular implementation
//!
//! This module provides the main sidebar component and its subcomponents
//! decomposed for maintainability and performance.

pub mod accounts;
pub mod cells;
pub mod menu;
pub mod more;
pub mod notifications;
pub mod search;
pub mod tabs;

use super::reducer::{SidebarAction, SidebarSignal};
use crate::{environment::storage::UiTab, widgets::*};
use dioxus::prelude::*;

pub use accounts::SidebarAccountsComponent;
pub use cells::{AccountCellComponent, CellComponent};
pub use menu::MenuComponent;
pub use more::SidebarMoreComponent;
pub use notifications::SidebarNotificationsComponent;
pub use search::SearchComponent;
pub use tabs::{TabBar, TabBarItem};

#[component]
pub fn SidebarComponent(store: SidebarSignal) -> Element {
    log::trace!("Rerender SidebarComponent");
    let has_notifications = store.read().has_new_notifications;
    let has_messages = store.read().total_unread_conversations > 0;
    let tab = store.read().tab;
    let tabs = vec![
        TabBarItem::new(
            UiTab::Timeline,
            crate::loc!("Timelines").to_string(),
            tab.is_timeline(),
            false,
        ),
        TabBarItem::new(
            UiTab::Mentions,
            crate::loc!("Mentions").to_string(),
            tab.is_mentions(),
            has_notifications,
        ),
        TabBarItem::new(
            UiTab::Messages,
            crate::loc!("Messages").to_string(),
            tab.is_messages(),
            has_messages,
        ),
        TabBarItem::new(
            UiTab::Rooms,
            crate::loc!("Rooms").to_string(),
            tab.is_rooms(),
            false,  // TODO: Add unread room indicator in future
        ),
        TabBarItem::new(
            UiTab::More,
            crate::loc!("More").to_string(),
            tab.is_more(),
            false,
        ),
    ];

    #[cfg(target_os = "macos")]
    let is_not_macos = false;

    #[cfg(not(target_os = "macos"))]
    let is_not_macos = true;

    let cloned_tabs = tabs.clone();
    let menu_component = is_not_macos.then(|| {
        rsx! {
            MenuComponent { store: store }

            TabBar {
                items: cloned_tabs.clone(),
                onclick: move |item: TabBarItem| {
                    use crate::components::sidebar::handle_action;
                    use crate::app::use_environment;
                    let environment = use_environment();
                    handle_action(store, SidebarAction::ChangeTab(item.id), &environment.read());
                }
            }
        }
    });

    rsx! {
        VStack { class: "min-h-auto h-screen",
            {menu_component}

            {
                // Safe tab access using get() instead of direct indexing to prevent panics
                if tabs.first().is_some_and(|t| tab == t.id) {
                    rsx!(SidebarAccountsComponent {
                        store: store,
                    })
                } else if tabs.get(1).is_some_and(|t| tab == t.id) {
                    rsx!(SidebarNotificationsComponent {
                        store: store,
                    })
                } else if tabs.get(2).is_some_and(|t| tab == t.id) {
                    // Messages tab - show conversation list
                    let store_read = store.read();
                    let Some(account) = store_read.selected_account.as_ref() else {
                        return rsx!(div {
                            class: "flex flex-row relative p-3 m-3 flex-grow self-center",
                            "Loading conversations..."
                        });
                    };

                    use crate::components::component_stack::{Stack, State, RootTimelineKind};

                    rsx!(Stack {
                        store: use_signal(|| State::new(RootTimelineKind::ConversationList(account.clone())))
                    })
                } else if tabs.get(3).is_some_and(|t| tab == t.id) {
                    // Rooms tab - show room list using Stack + RoomListProvider
                    let store_read = store.read();
                    let Some(account) = store_read.selected_account.as_ref() else {
                        return rsx!(div {
                            class: "flex flex-row relative p-3 m-3 flex-grow self-center text-gray-400",
                            "Loading rooms..."
                        });
                    };

                    use crate::components::component_stack::{Stack, State, RootTimelineKind};

                    // Multi-agent conversations are now unified with single-agent in ConversationList
                    rsx!(Stack {
                        store: use_signal(|| State::new(RootTimelineKind::ConversationList(account.clone())))
                    })
                } else if tabs.get(4).is_some_and(|t| tab == t.id) {
                    rsx!(SidebarMoreComponent {
                        store: store
                    })
                } else {
                    rsx!({})
                }
            }
        }
    }
}
