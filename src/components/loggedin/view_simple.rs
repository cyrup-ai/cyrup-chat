use crate::components::chat::ChatComponent;
use crate::environment::Environment;
use dioxus::prelude::*;

#[component]
pub fn LoggedInApp(environment: Signal<Environment>, should_show_login: Signal<bool>) -> Element {
    log::trace!("rerender loggedin-app");

    rsx! {
        div {
            class: "app-container",
            div {
                class: "sidebar",
                ChatHistorySidebar {}
            }
            div {
                class: "content-component",
                ChatComponent {}
            }
        }
    }
}

#[component]
fn ChatHistorySidebar() -> Element {
    rsx! {
        div {
            class: "chat-history-sidebar",
            div {
                class: "sidebar-header",
                h3 { "Chat History" }
            }
            div {
                class: "history-list",
                div {
                    class: "history-item active",
                    div { class: "history-date", "Today" }
                    div { class: "history-title", "New conversation" }
                }
                div {
                    class: "history-item",
                    div { class: "history-date", "Yesterday" }
                    div { class: "history-title", "Fluent Builder for AI Domain Model" }
                }
                div {
                    class: "history-item",
                    div { class: "history-date", "Yesterday" }
                    div { class: "history-title", "AI Executive Job Search Strategy" }
                }
                div {
                    class: "history-item",
                    div { class: "history-date", "June" }
                    div { class: "history-title", "Setting Environment Variables" }
                }
                div {
                    class: "history-item",
                    div { class: "history-date", "June" }
                    div { class: "history-title", "Ultra-Fast Key Binding Resolution" }
                }
                div {
                    class: "history-item",
                    div { class: "history-date", "June" }
                    div { class: "history-title", "Optimized Key Binding Resolution" }
                }
            }
        }
    }
}
