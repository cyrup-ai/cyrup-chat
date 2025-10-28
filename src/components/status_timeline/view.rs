use dioxus::prelude::*;
// Modern Dioxus patterns

use crate::app::use_environment;
use crate::components::status_timeline::reducer::{Action, TimelineSignal, handle_action};

use crate::widgets::*;

#[component]
pub fn TimelineComponent(store: TimelineSignal) -> Element {
    log::trace!("Rerender TimelineComponent");

    let state = store.read();

    rsx! {
        div {
            class: "timeline-container",

            // Error message
            if let Some(error) = &state.error {
                div {
                    class: "error-message",
                    "Error: {error}"
                }
            }

            // Loading indicator
            if state.is_loading {
                div {
                    class: "loading-indicator flex flex-row relative p-2 m-2 flex-grow self-center",
                    Spinner {}
                }
            }

            // Render posts
            div {
                class: "timeline-posts",
                for post in &state.posts {
                    div {
                        key: "post-{post.id}",
                        class: "timeline-post",
                        // Use the proper Post component for rendering timeline posts
                        crate::components::post::PostView {
                            store: use_signal(|| crate::components::post::State::from_status(post.clone())),
                            environment: crate::app::use_environment(),
                        }
                    }
                }
            }

            // Load more indicator
            if state.is_loading_more {
                div {
                    class: "loading-more flex flex-row relative p-2 m-2 flex-grow self-center",
                    Spinner {}
                }
            }

            // Can load more button
            if state.can_load_more && !state.is_loading && !state.is_loading_more {
                div {
                    class: "flex flex-row relative justify-center mt-2",
                    LoadMoreButton { store: store }
                }
            }
        }
    }
}

#[component]
fn LoadMoreButton(store: TimelineSignal) -> Element {
    let mut load_more = use_signal(|| false);

    // Effect to handle the load more action
    use_effect(move || {
        if load_more() {
            let mut environment = use_environment();
            let mut env_ref = environment.write();
            handle_action(store, Action::LoadMoreData(None), &mut env_ref);
            load_more.set(false);
        }
    });

    rsx! {
        IconTextButton {
            icon: crate::icons::ICON_LOAD_OLDER_TIMELINE,
            text: "Load More",
            title: "Load more timeline data",
            class: Some("mb-3"),
            onclick: move |_| {
                load_more.set(true);
            }
        }
    }
}

#[component]
pub fn TimelineContents(store: TimelineSignal) -> Element {
    // This component is a wrapper around TimelineComponent
    // with additional layout or behavior as needed
    rsx! {
        div {
            class: "timeline-contents",
            TimelineComponent { store: store }
        }
    }
}
