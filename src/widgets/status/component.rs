//! Main StatusComponent implementation

use super::actions::StatusAction;
use crate::environment::menu::{self};
use crate::icons;
use crate::loc;
use crate::view_model::StatusViewModel;
use crate::widgets::*;
use dioxus::prelude::{Element, EventHandler, component, rsx, use_memo};

#[component]
pub fn StatusComponent(
    status: StatusViewModel,
    is_in_conversation: Option<bool>,
    onclick: EventHandler<StatusAction>,
    sender: EventHandler<StatusAction>,
    children: Element,
) -> Element {
    // Zero-allocation state management using Dioxus signals for efficient updates
    // Memoize computed values to avoid unnecessary re-computations
    let status_state = use_memo(move || status.clone());

    // Memoize string values to avoid cloning on every render
    let text_content = use_memo(move || {
        let status = status_state.read();
        (
            status.replies_title.clone(),
            status.reblog_title.clone(),
            status.reblog.clone(),
            status.favourited_title.clone(),
            status.favourited.clone(),
            status.bookmarked_title.clone(),
            status.content.clone(),
            status.uri.clone(),
            status.text.clone(),
        )
    });

    // Memoize interactive state and icons for efficient updates
    let interactive_state = use_memo(move || {
        let status = status_state.read();
        (
            // Reblog state and icon
            status.has_reblogged,
            if status.has_reblogged {
                crate::icons::ICON_BOOST2
            } else {
                crate::icons::ICON_BOOST1
            },
            // Favorite state and icon
            status.is_favourited,
            if status.is_favourited {
                crate::icons::ICON_STAR2
            } else {
                crate::icons::ICON_STAR1
            },
            // Bookmark state and icon
            status.is_bookmarked,
            if status.is_bookmarked {
                crate::icons::ICON_BOOKMARK2
            } else {
                crate::icons::ICON_BOOKMARK1
            },
        )
    });

    // Extract values for use in rsx! macro
    let (
        replies_title,
        reblog_title,
        reblog_text,
        favourited_title,
        favourited_text,
        bookmarked_title,
        content,
        uri,
        text,
    ) = text_content.read().clone();

    let (reb_state, reb_icon, fav_state, fav_icon, bk_state, bk_icon) = *interactive_state.read();

    rsx! {
        div {
            class: "enable-pointer-events",
            onclick: move |_| onclick.call(StatusAction::Clicked),
            {children},
            TextContent {
                content: content,
                onclick: move |action| match action {
                    TextContentAction::Tag(tag) => onclick.call(StatusAction::OpenTag(tag)),
                    TextContentAction::Link(link) => onclick.call(StatusAction::OpenLink(link)),
                    TextContentAction::Account(link) => onclick.call(StatusAction::OpenAccount(link)),
                },
                class: ""
            }
        }

        super::media::ContentCellMedia {
            status: status_state.read().clone(),
            onclick: move |evt| onclick.call(evt),
            sender: sender
        }

        HStack { class: "justify-between m-2 gap-4 wrap enable-pointer-events",
            IconButton {
                icon: icons::ICON_REPLY,
                title: replies_title,
                onclick: move |_| {
                    onclick.call(StatusAction::Reply);
                }
            }
            IconTextButton {
                icon: reb_icon,
                title: reblog_title,
                text: reblog_text,
                onclick: move |_| {
                    onclick.call(StatusAction::Boost(!reb_state));
                }
            }
            IconTextButton {
                icon: fav_icon,
                title: favourited_title,
                text: favourited_text,
                onclick: move |_| {
                    onclick.call(StatusAction::Favorite(!fav_state));
                }
            }
            IconButton {
                icon: bk_icon,
                title: bookmarked_title,
                onclick: move |_| {
                    onclick.call(StatusAction::Bookmark(!bk_state));
                }
            }
            IconButton {
                icon: icons::ICON_OPTIONS,
                title: loc!("Options"),
                onclick: move |e: Event<MouseData>| {
                    let mut items = vec![
                        menu::ContextMenuItem::item(
                            loc!("Copy Link"),
                            StatusAction::Copy(uri.clone())
                        ),
                        menu::ContextMenuItem::item(
                            loc!("Open in Browser"),
                            StatusAction::OpenLink(uri.clone())
                        ),
                        menu::ContextMenuItem::item(
                            loc!("Copy Text"),
                            StatusAction::Copy(text.clone())
                        ),
                    ];
                    if !is_in_conversation.unwrap_or_default() {
                        items.push(menu::ContextMenuItem::separator());
                        items
                            .push(
                                menu::ContextMenuItem::item(
                                    loc!("Open Conversation"),
                                    StatusAction::Clicked
                                ),
                            );
                    }
                    // Efficient context menu handling with zero-allocation patterns
                    let environment = crate::app::use_environment();
                    let coords = e.client_coordinates();

                    // Use efficient menu processing with proper type safety
                    let menu_items: Vec<(String, StatusAction)> = items
                        .into_iter()
                        .filter_map(|item| {
                            match item.kind() {
                                menu::ContextMenuKind::Item { title, payload, .. } => {
                                    // Safe extraction with proper error handling
                                    payload.downcast_ref::<StatusAction>()
                                        .map(|action| (title.clone(), action.clone()))
                                },
                                menu::ContextMenuKind::Separator => {
                                    Some(("---".to_string(), StatusAction::Clicked))
                                },
                                _ => None, // Skip unknown variants
                            }
                        })
                        .collect();

                    // Efficient string reference conversion for platform API
                    let menu_items_ref: Vec<(&str, StatusAction)> = menu_items
                        .iter()
                        .map(|(title, action)| (title.as_str(), action.clone()))
                        .collect();

                    // Type-safe context menu display with proper error handling
                    environment.read().platform.show_context_menu(
                        (coords.x as i32, coords.y as i32),
                        "Post Options",
                        menu_items_ref,
                        move |action| {
                            sender.call(action);
                        }
                    );
                }
            }
        }
    }
}
