//! Content area components for different tabs

use super::super::RootSignal;
use crate::components::component_stack::RootTimelineKind;
use crate::widgets::HideableView;
use dioxus::prelude::*;

#[component]
pub fn ContentComponent(store: RootSignal) -> Element {
    let tab = store.read().active_tab;
    rsx! {
        div { class: "bg-[var(--g-backgroundWindowDark)] min-h-auto flex-grow flex h-screen",
            AccountContentComponent {
                store: store,
                hidden: !tab.is_timeline()
            }
            NotificationContentComponent {
                store: store,
                hidden: !tab.is_mentions()
            }
            MoreComponent {
                store: store,
                hidden: !tab.is_more()
            }
        }
    }
}

#[component]
pub fn AccountContentComponent(store: RootSignal, hidden: bool) -> Element {
    let store_read = store.read();
    let Some(account) = store_read.selected_account.as_ref() else {
        return rsx! {
            div {}
        };
    };

    use crate::components::component_stack::{Stack, State};

    rsx!(HideableView {
        hidden: hidden,
        Stack {
            store: use_signal(|| State::new(RootTimelineKind::ConversationList(account.clone())))
        }
    })
}

#[component]
pub fn NotificationContentComponent(store: RootSignal, hidden: bool) -> Element {
    let store_read = store.read();
    let Some(account) = store_read.selected_notifications.as_ref() else {
        return rsx! {
            div {}
        };
    };
    use crate::components::component_stack::{Stack, State};

    rsx!(HideableView {
        hidden: hidden,

        Stack {
            // Q28-Q30: Notifications shown as conversation list in MVP
            store: use_signal(|| State::new(RootTimelineKind::ConversationList(account.clone())))
        }
    })
}

#[component]
pub fn MoreComponent(store: RootSignal, hidden: bool) -> Element {
    log::trace!("render MoreComponent");
    let store_read = store.read();
    let selection = store_read.more_selection;
    let Some(account) = store_read.current_user.as_ref() else {
        return rsx!(div {});
    };

    use crate::components::more::{MoreViewComponent, State};
    rsx!(HideableView {
        hidden: hidden,
         MoreViewComponent {
            store: use_signal(|| State::new(selection, account.clone()))
        }
    })
}
