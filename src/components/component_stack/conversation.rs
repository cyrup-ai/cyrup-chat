//! Conversation component

use super::state::StackSignal;
use crate::view_model::StatusId;
use dioxus::prelude::*;

#[component]
pub fn ConversationComponent(conversation: StatusId, store: StackSignal) -> Element {
    use crate::components::conversation::{ConversationComponent, State as ConversationState};
    rsx! {
        ConversationComponent {
            store: use_signal(|| ConversationState::new(conversation.clone()))
        }
    }
}
