use crate::view_model::*;
use crate::widgets::*;
use dioxus::prelude::*;

use super::super::ConversationSignal;
use super::super::{conversation_helpers::Conversation, reducer::Action};
use super::UserConversationComponentChild;

// ChildReducer trait implementation removed - using Signal-based patterns instead

#[component]
pub fn UserConversationComponentChildren(
    conversation: Conversation,
    store: ConversationSignal,
    conversation_children: Vec<(StatusViewModel, StatusId)>,
    on_action: EventHandler<Action>,
) -> Element {
    let mut hidden = use_signal(|| false);
    let is_hidden = *hidden.read();
    let ln = conversation_children.len();
    let has_children = !conversation_children.is_empty();
    let cls = if has_children { "has-children" } else { "" };

    // Clone conversation for use in the closure
    let conversation_clone = conversation.clone();
    let content = if is_hidden {
        rsx! {
            div {
                class: "hidden-content",
                onclick: move |_| hidden.toggle(),
                Label { "{ln} More" }
            }
        }
    } else {
        rsx! {
            for (child_status, child_id) in conversation_children.iter() {
                UserConversationComponentChild {
                    key: "{child_id.0}",
                    conversation: conversation_clone.clone(),
                    store: store,
                    child_status: child_status.clone(),
                    child_id: child_id.clone(),
                    on_action: on_action
                }
            }
            div {
                class: "sideline",
                onclick: move |_| hidden.toggle()
            }
        }
    };

    let div_class = format!("conversation-children {cls}");
    rsx! {
        div {
            class: div_class,
            {content}
        }
    }
}
