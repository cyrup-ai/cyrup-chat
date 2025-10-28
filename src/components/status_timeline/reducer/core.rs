use super::super::providers::AnyTimelineProvider;
use crate::PublicAction;
use crate::environment::model::Status;
use crate::environment::types::{AppEvent, TimelineDirection, UiConfig};
use crate::view_model::{
    AccountId, AccountViewModel, AccountVisibility, StatusId, StatusViewModel,
};
use dioxus::prelude::*;

// Modern Dioxus signal-based state management
pub type TimelineSignal = Signal<State>;

#[derive(Clone, Debug)]
pub struct State {
    pub is_loading: bool,
    pub is_loading_more: bool,
    pub can_load_more: bool,
    pub provider: AnyTimelineProvider,
    pub error: Option<String>,
    /// If there is a related account to these posts, it is set here
    pub account: Option<AccountViewModel>,
    /// This is needed because every provider will present different data
    pub posts: Vec<StatusViewModel>,
    pub ui_settings: UiConfig,
    pub should_scroll_to_newest: bool,
    pub forced_direction: Option<TimelineDirection>,
    pub identifier: String,
    pub known_conversations: Vec<StatusId>,
}

#[derive(Clone)]
pub enum Action {
    Initial,
    LoadData,
    LoadedData(Result<Vec<Status>, String>, bool),
    LoadMoreData(Option<StatusId>),
    LoadedMoreData(Result<Vec<Status>, String>),
    ShouldReloadSoft,
    ReloadSoft(bool),
    DataChanged,
    AccountVisibility(AccountId, AccountVisibility),
    Public(Box<PublicAction>),
    AppEvent(AppEvent),
}

impl std::fmt::Debug for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initial => write!(f, "Initial"),
            Self::LoadData => write!(f, "LoadData"),
            Self::LoadedData(_arg0, _arg1) => f.debug_tuple("LoadedData").finish(),
            Self::LoadMoreData(_arg0) => f.debug_tuple("LoadMoreData").finish(),
            Self::LoadedMoreData(_arg0) => f.debug_tuple("LoadedMoreData").finish(),
            Self::ShouldReloadSoft => write!(f, "ShouldReloadSoft"),
            Self::ReloadSoft(arg0) => f.debug_tuple("ReloadSoft").field(arg0).finish(),
            Self::DataChanged => write!(f, "DataChanged"),
            Self::AccountVisibility(arg0, arg1) => f
                .debug_tuple("AccountVisibility")
                .field(arg0)
                .field(arg1)
                .finish(),
            Self::Public(arg0) => f.debug_tuple("Public").field(arg0).finish(),
            Self::AppEvent(arg0) => f.debug_tuple("AppEvent").field(arg0).finish(),
        }
    }
}

impl State {
    pub fn new(
        provider: AnyTimelineProvider,
        ui_settings: UiConfig,
        account: Option<AccountViewModel>,
    ) -> Self {
        let forced_direction = provider.forced_direction();
        Self {
            provider,
            is_loading: Default::default(),
            is_loading_more: Default::default(),
            can_load_more: Default::default(),
            error: Default::default(),
            account,
            posts: Default::default(),
            ui_settings,
            should_scroll_to_newest: false,
            forced_direction,
            identifier: Default::default(),
            known_conversations: Vec::new(),
        }
    }

    pub fn direction(&self) -> TimelineDirection {
        self.forced_direction.unwrap_or(self.ui_settings.direction)
    }
}
