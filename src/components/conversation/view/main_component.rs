// ProfilePreviewReducer removed - using modern Dioxus signal patterns
// profile_preview deleted in AGENT_17 - profiles feature removed
use crate::PublicAction;
use crate::view_model::*;
use crate::widgets::*;
use dioxus::prelude::*;

use super::super::ConversationSignal;
#[allow(unused_imports)] // Used for conversation data structure in component logic
use super::super::{conversation_helpers::Conversation, reducer::Action};
use super::{UserConversationComponentChildren, UserConversationHeader};

#[component]
pub fn ConversationComponent(store: ConversationSignal) -> Element {
    let is_loading = store.read().is_loading;
    if is_loading {
        return rsx! {
            div {
                class: "flex flex-col relative p-3 m-3 items-center flex-grow justify-items-center",
                Spinner {}
            }
        };
    }

    // Extract conversation data with proper lifetime handling - clone everything we need
    let conversation_data = match store.read().conversation.clone() {
        Some(conv) => conv,
        None => {
            return rsx! {
                ErrorPage {
                    content: "Unknown Conversation"
                }
            };
        }
    };

    let (root_status, children_data) = if let Some(root) = conversation_data.root() {
        let root_status = root.cloned_status();
        let children = conversation_data.children(&root).unwrap_or_default();
        (root_status, children)
    } else {
        return rsx! {
            ErrorPage {
                content: "Unknown Conversation Root"
            }
        };
    };

    // Now we have owned values that can live for 'static
    let children: Vec<(StatusViewModel, StatusId)> = children_data
        .into_iter()
        .map(|item| {
            let status = item.cloned_status();
            let status_id = status.id.clone();
            (status, status_id)
        })
        .collect();
    let cloned_status = root_status.clone();
    let cloned_account = cloned_status.account.clone();

    let store_clone = store;
    let root_clone = root_status.clone();

    rsx! {
        VStack {
            class: "content flex-grow",
            UserConversationHeader {
                status: cloned_status,
                store: store,
                on_action: move |action| {
                    // Handle actions by updating the conversation signal state
                    use crate::app::use_environment;
                    let environment = use_environment();
                    use crate::components::conversation::reducer::handle_action;
                    handle_action(store, action, &environment.read());
                }
            }
            div {
                class: "conversation-container scroll",
                div {
                    class: "content-cell no-selection conversation-ancestor",
                    // Profile presentation inline
                    div {
                        class: "conversation-profile clickable",
                        onclick: move |_| {
                            use crate::app::use_environment;
                            let environment = use_environment();
                            use crate::components::conversation::reducer::handle_action;
                            handle_action(
                                store_clone,
                                Action::Public(Box::new(PublicAction::OpenProfile(cloned_account.clone()))),
                                &environment.read()
                            );
                        },
                        HStack { class: "gap-2 items-center",
                            img {
                                class: "rounded-lg w-8 h-8",
                                src: "{cloned_account.image}",
                                alt: "{cloned_account.display_name}",
                                width: 32,
                                height: 32
                            }
                            VStack { class: "gap-1",
                                Label {
                                    style: TextStyle::Primary,
                                    pointer_style: PointerStyle::Pointer,
                                    dangerous_content: Some(cloned_account.display_name_html.clone()),
                                    ""
                                }
                                Label {
                                    style: TextStyle::Secondary,
                                    pointer_style: PointerStyle::Pointer,
                                    "@{cloned_account.username}"
                                }
                            }
                        }
                    }

                    StatusComponent {
                        status: root_clone.clone(),
                        is_in_conversation: true,
                        onclick: move |action| {
                            use crate::app::use_environment;
                            let environment = use_environment();
                            use crate::components::conversation::reducer::handle_action;
                            let public_action: crate::PublicAction = (action, root_clone.clone()).into();
                            handle_action(store_clone, Action::Public(Box::new(public_action)), &environment.read());
                        },
                        sender: move |action| {
                            use crate::app::use_environment;
                            let environment = use_environment();
                            use crate::components::conversation::reducer::handle_action;
                            handle_action(store, Action::StatusAction(action), &environment.read());
                        },
                        ""
                    }
                }
                UserConversationComponentChildren {
                    conversation: conversation_data.clone(),
                    store: store,
                    conversation_children: children,
                    on_action: move |action| {
                        // Handle actions from child components
                        use crate::app::use_environment;
                        let environment = use_environment();
                        use crate::components::conversation::reducer::handle_action;
                        handle_action(store, action, &environment.read());
                    }
                }
            }
        }
    }
}
