//! Main Stack component implementation

use super::{
    conversation::ConversationComponent, providers::ProviderKind, stack_entry::StackEntry,
    state::StackSignal,
};
// Profiles removed in AGENT_17 - profiles feature deleted
use crate::components::status_timeline::{AnyTimelineProvider, TimelineComponent};
use dioxus::prelude::*;

#[component]
pub fn Stack(store: StackSignal) -> Element {
    let store_read = store.read();
    let Some(provider) = store_read.root_provider.as_ref() else {
        return rsx!(div {});
    };
    rsx! {
        div {
            style: "position: relative; flex-basis: 520px; max-width: 520px; flex-shrink: 0;",
            match provider {
                ProviderKind::Timeline(t) => rsx!(TimelineInStack {
                    store: store,
                    provider: t.clone()
                }),
            }

            for profile in store.read().stack.iter() {
                StackEntry {
                    store: store,
                    profile: profile.clone()
                }
            }
        }
        {store
            .read().current_conversation
            .as_ref()
            .map(|c| rsx!(ConversationComponent { conversation: c.clone(), store: store }))}
    }
}

#[component]
pub fn TimelineInStack(store: StackSignal, provider: AnyTimelineProvider) -> Element {
    use crate::components::status_timeline::State as TimelineState;
    rsx! {
        TimelineComponent {
            store: use_signal(|| {
                TimelineState::new(provider.clone(), store.read().ui_settings.clone(), Some(store.read().root_timeline_kind.model()))
            })
        }
    }
}

// ProfilesInStack removed in AGENT_17 - profiles feature deleted (Q45/Q46: templates replace profiles)
