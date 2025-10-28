//! Sidebar tab components
//!
//! This module provides the tab bar and tab button components for the sidebar.

use crate::environment::storage::UiTab;
use dioxus::prelude::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TabBarItem {
    pub id: UiTab,
    pub label: String,
    pub selected: bool,
    pub dot: bool,
}

impl TabBarItem {
    #[allow(dead_code)] // Tab bar item constructor - pending UI integration
    pub fn new(id: UiTab, label: String, selected: bool, dot: bool) -> Self {
        Self {
            id,
            label,
            selected,
            dot,
        }
    }
}

#[component]
pub fn TabBar(items: Vec<TabBarItem>, onclick: EventHandler<TabBarItem>) -> Element {
    let items_memo = use_memo(move || items.clone());
    // Fix E0716 & E0597: Clone the items to avoid lifetime issues with temporary values
    let items_cloned = items_memo().clone();
    let items_iter = items_cloned.into_iter().map(|item| {
        let item_for_click = item.clone();
        rsx!(TabButton {
            label: item.label.clone(),
            onclick: move |_| onclick.call(item_for_click.clone()),
            selected: item.selected,
            dot: item.dot,
        })
    });
    rsx! {
        div { class: "tabbar",
            {items_iter}
        }
    }
}

#[component]
pub fn TabButton(label: String, onclick: EventHandler<()>, selected: bool, dot: bool) -> Element {
    let dot = dot.then(|| rsx!(span { class: "dot" }));
    let cls = if selected { " selected" } else { "" };
    rsx! {
        button { class: "button {cls}", onclick: move |_| {
                onclick.call(());
            }, {dot}, "{label}" }
    }
}
