use super::reducer::MoreSignal;
use crate::{
    components::{component_stack::RootTimelineKind, sidebar::MoreSelection},
    widgets::*,
};
use dioxus::prelude::*;

#[component]
pub fn MoreViewComponent(store: MoreSignal) -> Element {
    log::trace!(" {:?}", store.read().selection);
    rsx! {
        StatusesPageComponent {
            title: "classic timeline",
            store: store,
            provider: store.read().providers.classic_timeline.clone(),
            hidden: store.read().selection != MoreSelection::Classic
        }

        StatusesPageComponent {
            title: "yours",
            store: store,
            provider: store.read().providers.account.clone(),
            hidden: store.read().selection != MoreSelection::Yours
        }

        StatusesPageComponent {
            title: "bookmarks",
            store: store,
            provider: store.read().providers.bookmarks.clone(),
            hidden: store.read().selection != MoreSelection::Bookmarks
        }

        StatusesPageComponent {
            title: "favorites",
            store: store,
            provider: store.read().providers.favorites.clone(),
            hidden: store.read().selection != MoreSelection::Favorites
        }

        StatusesPageComponent {
            title: "federated",
            store: store,
            provider: store.read().providers.public.clone(),
            hidden: store.read().selection != MoreSelection::Federated
        }

        StatusesPageComponent {
            title: "local",
            store: store,
            provider: store.read().providers.local.clone(),
            hidden: store.read().selection != MoreSelection::Local
        }

        StatusesPageComponent {
            title: "follows",
            store: store,
            provider: store.read().providers.follows.clone(),
            hidden: store.read().selection != MoreSelection::Followers
        }

        StatusesPageComponent {
            title: "following",
            store: store,
            provider: store.read().providers.following.clone(),
            hidden: store.read().selection != MoreSelection::Following
        }
    }
}

#[component]
fn StatusesPageComponent(
    title: String,
    hidden: bool,
    store: MoreSignal,
    provider: Option<RootTimelineKind>,
) -> Element {
    let Some(ref provider) = provider else {
        return rsx!(div {});
    };

    use crate::components::component_stack::{Stack, State};

    rsx!(HideableView {
        hidden: hidden,
        Stack {
            store: use_signal(|| State::new(provider.clone()))
        }
    })
}

// ChildReducer trait implementation removed - using Signal-based patterns instead
