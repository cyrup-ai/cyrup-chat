//! Main layout components for the logged-in interface

use super::super::RootSignal;
use super::content::ContentComponent;
use super::reply::ReplyComponent;
use crate::components::sidebar::{SidebarComponent, SidebarState};
use crate::widgets::{Spinner, SplitViewComponent};
use dioxus::prelude::*;

#[component]
pub fn MainComponent(store: RootSignal) -> Element {
    log::trace!("Rerender MainComponent");

    if store.read().flags.logging_in {
        rsx! {
            div {
                class: "flex flex-col relative p-2 m-2 flex-grow items-center justify-items-center w-full h-full",
                p { class: "text-[var(--g-font-size--3)] text-[var(--g-textColor)] mt-[20%]" }
                div {
                    class: "flex flex-row relative p-2 m-2 flex-grow self-center w-full h-full",
                    Spinner {}
                }
            }
        }
    } else if store.read().logged_in {
        let reply_component = store.read().is_replying.clone().map(|(kind, images)| {
            rsx! {
                ReplyComponent {
                    store: store,
                    kind: kind,
                    images: images
                }
            }
        });

        rsx! {
            SplitViewComponent {
                sidebar: rsx! {
                    SidebarComponent {
                        store: use_signal(SidebarState::default)
                    }
                },
                content: rsx! {
                    ContentComponent { store: store }
                }
            }
            {reply_component}
        }
    } else {
        rsx! { div {} }
    }
}
