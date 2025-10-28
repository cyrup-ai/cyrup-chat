use crate::view_model::*;
use crate::widgets::*;
use crate::{icons, loc};
use dioxus::prelude::*;

use super::super::ConversationSignal;
use super::super::{conversation_helpers::Conversation, reducer::Action};
use super::UserConversationComponentChildren;
use crate::PublicAction;

#[component]
pub fn UserConversationComponentChild(
    conversation: Conversation,
    store: ConversationSignal,
    child_status: StatusViewModel,
    child_id: StatusId,
    on_action: EventHandler<Action>,
) -> Element {
    // profile_preview removed in AGENT_17 - profiles feature deleted
    use crate::widgets::StatusAction;

    // Get children through efficient conversation tree traversal
    // Using optimized lookup with proper lifetime management
    let children = if let Some(root) = conversation.root() {
        if let Some(all_children) = conversation.children(&root) {
            // Find the matching child and get its children
            let mut result = vec![];
            for item in all_children {
                if item.cloned_status().id == child_id
                    && let Some(grandchildren) = conversation.children(&item)
                {
                    result = grandchildren
                        .into_iter()
                        .map(|item| {
                            let status = item.cloned_status();
                            let status_id = status.id.clone();
                            (status, status_id)
                        })
                        .collect();
                    break;
                }
            }
            result
        } else {
            vec![]
        }
    } else {
        vec![]
    };

    let has_children = !children.is_empty();
    let cls = if has_children { "has-children" } else { "" };

    // Clone conversation for use in the children component
    let conversation_for_children = conversation.clone();

    // Use a closure to read from the store to ensure proper lifetime handling
    let conversation_id = {
        let store_value = store.read();
        store_value.conversation_id.clone()
    };

    // Determine if the current child is selected
    let is_selected = if conversation_id == child_id {
        "conversation-child-selected".to_string()
    } else {
        String::new()
    };

    let id = child_status.id.dom_id();
    let child_key = format!("child-{}", child_status.id.0.clone());
    let child_clone = child_status.clone();
    let on_action_clone = on_action;
    let child_clone2 = child_status.clone();
    let on_action_clone2 = on_action;
    let on_action_clone3 = on_action;
    let on_action_clone4 = on_action;

    let child_class = format!("conversation-child {cls} {is_selected}");
    let conv_id = format!("conv-{id}");
    let child_account = child_status.account.clone();
    let child_status_images = child_status.status_images.clone();
    let child_media = child_status.media.clone();

    rsx! {
        div {
            div {
                class: child_class,
                div {
                    id: conv_id,
                    class: "optionbox",
                    IconButton {
                        icon: icons::ICON_OPTIONS,
                        title: loc!("Options").to_string(),
                        onclick: move |_e: Event<MouseData>| {
                            let child = child_clone.clone();
                            // Primary action: Reply
                            on_action_clone(Action::Public(Box::new((StatusAction::Reply, child.clone()).into())));
                        }
                    }
                }

                // Profile presentation inline
                div {
                    class: "conversation-profile child-profile clickable",
                    onclick: move |_| {
                        on_action_clone3(Action::Public(Box::new(PublicAction::OpenProfile(child_account.clone()))));
                    },
                    HStack { class: "gap-2 items-center",
                        img {
                            class: "rounded-md w-7 h-7",
                            src: "{child_account.image}",
                            alt: "{child_account.display_name}",
                            width: 28,
                            height: 28
                        }
                        Label {
                            style: TextStyle::Primary,
                            pointer_style: PointerStyle::Pointer,
                            dangerous_content: Some(child_account.display_name_html.clone()),
                            ""
                        }
                    }
                }

                TextContent {
                    content: child_clone2.content.clone(),
                    onclick: move |action| match action {
                        TextContentAction::Tag(tag) => {
                            on_action_clone2(Action::Public(Box::new(PublicAction::OpenTag(tag))));
                        }
                        TextContentAction::Link(link) => {
                            on_action_clone2(Action::Public(Box::new(PublicAction::OpenLink(link))));
                        }
                        TextContentAction::Account(link) => {
                            on_action_clone2(Action::Public(Box::new(PublicAction::OpenLink(link))));
                        }
                    },
                    class: ""
                }

                // Extract image rendering to avoid temporary value issues
                for (description, preview, url) in child_status_images.iter() {
                    div {
                        class: "media-object",
                        img {
                            src: "{preview}",
                            alt: "{description}",
                            onclick: {
                                let on_action = on_action_clone3;
                                let url = url.clone();
                                move |_| on_action(Action::Public(Box::new(PublicAction::OpenImage(url.to_string()))))
                            },
                        }
                    }
                }

                // Extract media rendering to avoid temporary value issues
                for media in child_media.iter() {
                    if let Some(preview) = media.preview_url.as_ref() {
                        div {
                            class: "media-object",
                            img {
                                src: "{preview}",
                                alt: "{media.description}",
                                onclick: {
                                    let on_action = on_action_clone4;
                                    let media_clone = media.clone();
                                    move |_| on_action(Action::Public(Box::new(PublicAction::OpenVideo(media_clone.video_url.clone()))))
                                },
                            }
                        }
                    } else {
                        div {
                            class: "media-object",
                            span {
                                class: "empty text-[var(--g-font-size--3)] text-[var(--g-textColor)]",
                                title: "{media.description}",
                                onclick: {
                                    let on_action = on_action_clone4;
                                    let media_clone = media.clone();
                                    move |_| on_action(Action::Public(Box::new(PublicAction::OpenVideo(media_clone.video_url.clone()))))
                                },
                                "Video"
                            }
                        }
                    }
                }
            }
            {if has_children {
                // In Dioxus, we use the `key` attribute with a string literal
                // We'll use the child's ID as the key for efficient diffing
                rsx! {
                    UserConversationComponentChildren {
                        key: "{child_key}",
                        conversation: conversation_for_children,
                        store: store,
                        conversation_children: children,
                        on_action: on_action
                    }
                }
            } else {
                rsx!{}
            }}
        }
    }
}
