//! Sidebar accounts component
//!
//! This module provides the accounts view for the sidebar with search,
//! filtering, and account selection functionality.

use super::super::reducer::{SidebarAction, SidebarSignal};
use crate::{constants::ui_text, loc, view_model::AccountViewModel, widgets::*};
use dioxus::prelude::*;
use itertools::Itertools;
use std::cmp::Ordering;

use super::{AccountCellComponent, CellComponent, SearchComponent};

#[component]
pub fn SidebarAccountsComponent(store: SidebarSignal) -> Element {
    log::trace!("SidebarAccountsComponent: {}", store.read().accounts.len());
    let selection = &store.read().selected_account;
    let is_loading = store.read().loading_content;
    let search_term = &store.read().search_term;
    let favorites = &store.read().favorites;

    // Can we load more for the selected list / timeline?
    // Use separate store reads to avoid lifetime conflicts
    let can_load_more = {
        let store_read = store.read();
        let timeline_id = store_read.timeline_id();
        store_read.can_load_more(timeline_id)
    };

    let more_text = loc!("More");
    let more_section = (!store.read().search_results.is_empty()).then(|| {
        rsx!(div {
            class: "m-3",
            Label {
                style: TextStyle::Secondary,
                "{more_text}"
            }
        })
    });

    let search_results = store.read().search_results.clone();
    let accounts = store.read().accounts.clone();

    // Process accounts list with proper lifetime handling
    let filtered_accounts: Vec<_> = accounts
        .iter()
        .filter(|model| {
            if search_term.is_empty() {
                true
            } else {
                model.account.display_name.contains(search_term)
                    || model.account.username.contains(search_term)
            }
        })
        .sorted_by(
            |a, b| match (favorites.contains(&a.id.0), favorites.contains(&b.id.0)) {
                (true, false) => Ordering::Less,
                (false, true) => Ordering::Greater,
                _ => Ordering::Equal,
            },
        )
        .cloned()
        .collect();

    rsx! {
        ListSelector { store: store }

        SearchComponent { placeholder: ui_text::general_search_placeholder(), store: store }

        div { class: "scroll",
            div { class: "scroll-margin-fix",
                {
                filtered_accounts.into_iter()
                    .map(move |model| rsx!(CellComponent {
                        model: model.clone(),
                        selected: selection.as_ref().map(|e| &e.id) == Some(&model.id),
                        store: store,
                        onclick: move |_| {
                            use crate::components::sidebar::handle_action;
                            use crate::app::use_environment;
                            let environment = use_environment();
                            handle_action(store, SidebarAction::SelectAccount(model.account.clone()), &environment.read());
                        },
                        favorited: favorites.contains(&model.id.0)
                    }))
                },

                {more_section}

                for account in search_results.clone() {
                    AccountCellComponent {
                        account: account.clone(),
                        selected: store.read().selected_account.as_ref().map(|e| e.id.0.as_str()) == Some(account.id.as_str()),
                        store: store,
                        onclick: move |_| {
                            use crate::components::sidebar::handle_action;
                            use crate::app::use_environment;
                            let environment = use_environment();
                            handle_action(store, SidebarAction::SelectAccount(AccountViewModel::new(&account)), &environment.read());
                        }
                    }
                }

                {
                    (search_term.is_empty() && !is_loading && !store.read().posts_empty && can_load_more)
                    .then(|| rsx!(div {
                        class: "flex flex-row relative justify-center mt-2",
                        IconTextButton {
                            icon: crate::icons::ICON_LOAD_OLDER_TIMELINE,
                            text: "More",
                            title: "Load more timeline data",
                            class: "mb-4",
                            onclick: move |_| {
                                use crate::components::sidebar::handle_action;
                                use crate::app::use_environment;
                                let environment = use_environment();
                                handle_action(store, SidebarAction::LoadMoreTimeline, &environment.read());
                            },
                        },
                    }))
                }
            }
        }

        { (is_loading || store.read().is_searching).then(|| rsx!(div {
            class: "flex flex-row relative p-2 m-2 flex-grow self-center",
            Spinner {}
        }))}
    }
}

#[component]
fn ListSelector(store: SidebarSignal) -> Element {
    let has_timelines = if store.read().list_names.len() > 1 {
        "false"
    } else {
        "true"
    };
    rsx! {
        select {
            name: "list",
            class: "m-2",
            disabled: "{has_timelines}",
            onchange: move |evt| {
                use crate::components::sidebar::handle_action;
                use crate::app::use_environment;
                let environment = use_environment();
                handle_action(store, SidebarAction::SelectList(evt.value().clone()), &environment.read());
            },
            option { value: "", {loc!("Timeline")} }
            for (id , name) in store.read().list_names.iter() {
                option { value: "{id}", "{name}" }
            }
        }
    }
}
