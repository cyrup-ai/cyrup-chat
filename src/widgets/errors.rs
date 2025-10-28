use super::{HStack, IconButton, Paragraph, VStack};
use dioxus::prelude::*;

/// A small box that displays an error, with an optional action
#[component]
pub fn ErrorBox(content: String, onclick: EventHandler<()>) -> Element {
    rsx! {
        div {
            class: "error-box",
            HStack {
                class: "items-center",
                div {
                    class: "flex-grow",
                    Paragraph { "{content}" }
                }
                IconButton {
                    icon: crate::icons::ICON_DELETE,
                    title: "Clear",
                    onclick: move |_| onclick.call(())
                }
            }
        }
    }
}

/// A growing page with a centered error message
#[component]
pub fn ErrorPage(content: String) -> Element {
    rsx! {
        div {
            class: "p-3",
            VStack {
                class: "flex-grow text-[var(--g-font-size--3)] text-[var(--g-textColor)] font-semibold",
                h4 { "An error Occurred" },
                Paragraph { "{content}" }
            }
        }
    }
}
