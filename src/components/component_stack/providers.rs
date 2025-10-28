//! Timeline kind and provider mappings

use crate::components::status_timeline::{
    AnyTimelineProvider, BookmarkTimelineProvider, ConversationListProvider, RoomListProvider,
};
use crate::environment::{Environment, Model};
use crate::view_model::AccountViewModel;

/// Root timeline provider - move to status_timeline for better timeline abstraction
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(dead_code)] // Root timeline kinds - architectural scaffolding pending integration
pub enum RootTimelineKind {
    Bookmarks(AccountViewModel),
    ConversationList(AccountViewModel),
    RoomList(AccountViewModel),
}

#[derive(Debug, Clone)]
pub enum ProviderKind {
    Timeline(AnyTimelineProvider),
}

impl From<AnyTimelineProvider> for ProviderKind {
    fn from(value: AnyTimelineProvider) -> Self {
        Self::Timeline(value)
    }
}

impl RootTimelineKind {
    #[allow(dead_code)] // Timeline kind provider mapping - pending integration
    pub fn as_provider(&self, environment: &Environment, _model: &Model) -> ProviderKind {
        match self {
            RootTimelineKind::Bookmarks(a) => {
                AnyTimelineProvider::new(BookmarkTimelineProvider::new(environment.clone()), &a.id)
                    .into()
            }
            RootTimelineKind::ConversationList(a) => {
                AnyTimelineProvider::new(ConversationListProvider::new(environment.clone()), &a.id)
                    .into()
            }
            RootTimelineKind::RoomList(a) => {
                AnyTimelineProvider::new(RoomListProvider::new(environment.clone()), &a.id)
                    .into()
            }
        }
    }

    pub fn model(&self) -> AccountViewModel {
        match self {
            RootTimelineKind::Bookmarks(a) => a.clone(),
            RootTimelineKind::ConversationList(a) => a.clone(),
            RootTimelineKind::RoomList(a) => a.clone(),
        }
    }
}
