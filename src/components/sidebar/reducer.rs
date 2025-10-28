use im::{HashMap, Vector};
use std::collections::HashSet;

use crate::components::loggedin::Action;
use crate::environment::Environment;
use crate::environment::model::{Account, Notification, Status};
use crate::environment::storage::UiTab;
use crate::environment::types::{AppEvent, MainMenuEvent};
use crate::view_model::{AccountUpdateViewModel, AccountViewModel};
use dioxus::prelude::*;

// Modern Dioxus signal-based state management
pub type SidebarSignal = Signal<SidebarState>;

#[derive(Clone)]
#[allow(dead_code)] // Sidebar actions - architectural scaffolding pending integration
pub enum SidebarAction {
    Initial,
    ChangeTab(UiTab),
    SelectAccount(AccountViewModel),
    LoadTimeline,
    LoadMoreTimeline,
    LoadNotifications,
    SelectedNotifications(AccountViewModel),
    AppEvent(AppEvent),
    Reload(bool),
    DataChanged,
    FavoritesChanged,
    Search(String),
    LoadLists,
    SelectList(String),
    LoadList(String),
    Root(Box<Action>),
    HomeTimeline(Vec<Status>),
    MoreTimeline(Vec<Status>),
    Notifications,
    NotificationTimeline(Vec<Notification>),
    LoadMoreNotifications,
    MenuEvent(MainMenuEvent),
    SearchAccounts(String),
    More(MoreSelection),
    ListTimeline(ListEntry, Vec<Status>),
    ListsChanged(Vec<ListEntry>),
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // List entry structure - pending sidebar integration
pub struct ListEntry {
    pub id: String,
    pub title: String,
}

impl std::fmt::Debug for SidebarAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Initial => write!(f, "Initial"),
            Self::ChangeTab(arg0) => f.debug_tuple("ChangeTab").field(arg0).finish(),
            Self::SelectAccount(arg0) => f.debug_tuple("SelectAccount").field(arg0).finish(),
            Self::LoadTimeline => write!(f, "LoadTimeline"),
            Self::LoadMoreTimeline => write!(f, "LoadMoreTimeline"),
            Self::LoadNotifications => write!(f, "LoadNotifications"),
            Self::SelectedNotifications(arg0) => {
                f.debug_tuple("SelectedNotifications").field(arg0).finish()
            }
            Self::AppEvent(arg0) => f.debug_tuple("AppEvent").field(arg0).finish(),
            Self::Reload(arg0) => f.debug_tuple("Reload").field(arg0).finish(),
            Self::DataChanged => write!(f, "DataChanged"),
            Self::FavoritesChanged => write!(f, "FavoritesChanged"),
            Self::Search(arg0) => f.debug_tuple("Search").field(arg0).finish(),
            Self::LoadLists => write!(f, "LoadLists"),
            Self::SelectList(entry) => f.debug_tuple("SelectList").field(entry).finish(),
            Self::LoadList(id) => f.debug_tuple("LoadList").field(id).finish(),
            Self::Root(arg0) => f.debug_tuple("Root").field(arg0).finish(),
            Self::HomeTimeline(_) => write!(f, "HomeTimeline"),
            Self::MoreTimeline(_) => write!(f, "MoreTimeline"),
            Self::Notifications => write!(f, "Notifications"),
            Self::NotificationTimeline(_) => write!(f, "NotificationTimeline"),
            Self::LoadMoreNotifications => write!(f, "LoadMoreNotifications"),
            Self::MenuEvent(arg0) => f.debug_tuple("MenuEvent").field(arg0).finish(),
            Self::SearchAccounts(arg0) => f.debug_tuple("SearchAccounts").field(arg0).finish(),
            Self::More(id) => f.debug_tuple("More").field(id).finish(),
            Self::ListTimeline(entry, _) => f.debug_tuple("ListTimeline").field(entry).finish(),
            Self::ListsChanged(_) => write!(f, "ListsChanged"),
        }
    }
}

#[allow(unused)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, Default)]
pub enum MoreSelection {
    #[default]
    Classic,
    Yours,
    Local,
    Federated,
    Posts,
    Hashtags,
    Followers,
    Following,
    Bookmarks,
    Favorites,
}

#[derive(Clone)]
#[allow(dead_code)] // Sidebar state management - architectural scaffolding pending integration
pub struct SidebarState {
    pub list_names: Vec<(String, String)>,
    pub accounts: Vector<AccountUpdateViewModel>,
    pub tab: UiTab,
    pub selected_account: Option<AccountViewModel>,
    pub selected_notifications: Option<AccountViewModel>,
    pub notification_accounts: Vector<AccountUpdateViewModel>,
    pub user_account: Option<Account>,
    pub notification_posts_empty: bool,
    pub posts_empty: bool,
    pub last_timeline_id: HashMap<String, String>,
    pub has_new_notifications: bool,
    pub total_unread_conversations: u32,
    pub loading_content: bool,
    pub loading_notifications: bool,
    pub last_notification_id: Option<String>,
    pub search_results: Vec<Account>,
    pub is_searching: bool,
    pub search_term: String,
    pub favorites: HashSet<String>,
    pub selected_list: Option<String>,
    pub no_more_load_more: HashSet<String>,
    pub more_selection: MoreSelection,
    pub home_timeline: std::collections::VecDeque<Status>,
    pub notifications: Vec<Notification>,
    pub lists: Vec<ListEntry>,
    pub list_statuses: HashMap<String, Vec<Status>>,
}

impl Default for SidebarState {
    fn default() -> Self {
        Self {
            list_names: Vec::new(),
            accounts: Vector::new(),
            tab: UiTab::default(),
            selected_account: None,
            selected_notifications: None,
            notification_accounts: Vector::new(),
            user_account: None,
            notification_posts_empty: false,
            posts_empty: false,
            last_timeline_id: HashMap::new(),
            has_new_notifications: false,
            total_unread_conversations: 0,
            loading_content: false,
            loading_notifications: false,
            last_notification_id: None,
            search_results: Vec::new(),
            is_searching: false,
            search_term: String::new(),
            favorites: HashSet::new(),
            selected_list: None,
            no_more_load_more: HashSet::new(),
            more_selection: MoreSelection::default(),
            home_timeline: std::collections::VecDeque::new(),
            notifications: Vec::new(),
            lists: Vec::new(),
            list_statuses: HashMap::new(),
        }
    }
}

impl SidebarState {
    #[allow(dead_code)] // Load more pagination - pending timeline integration
    pub fn can_load_more(&self, id: &str) -> bool {
        !self.no_more_load_more.contains(id)
    }

    #[allow(dead_code)] // Timeline ID retrieval - pending timeline integration
    pub fn timeline_id(&self) -> &str {
        self.selected_list.as_deref().unwrap_or("")
    }

    #[allow(dead_code)] // List max ID tracking - pending timeline integration
    pub fn list_max_id(&self, id: &str) -> String {
        self.last_timeline_id.get(id).cloned().unwrap_or_default()
    }

    #[allow(dead_code)] // List max ID option tracking - pending timeline integration
    pub fn list_max_id_option(&self, id: &str) -> Option<String> {
        self.last_timeline_id.get(id).cloned()
    }
}

// Modern Dioxus signal-based state management
#[allow(dead_code)] // Sidebar action handler - pending integration
pub fn handle_action(mut signal: SidebarSignal, action: SidebarAction, environment: &Environment) {
    log::trace!("{action:?}");

    // Handle Initial action separately to avoid borrow conflicts
    if matches!(action, SidebarAction::Initial) {
        signal.with_mut(|state| {
            if let Some(n) = environment.settings.favorites() {
                state.favorites = n;
            }
        });

        // Load conversation summaries and calculate total unread
        let database = environment.database.clone();
        let mut signal_clone = signal;
        spawn(async move {
            match database.list_conversations().await {
                Ok(summaries) => {
                    let total_unread: u32 = summaries.iter().map(|s| s.unread_count).sum();

                    signal_clone.with_mut(|state| {
                        state.total_unread_conversations = total_unread;
                    });
                }
                Err(e) => {
                    log::error!("[Sidebar] Failed to load conversation counts: {}", e);
                }
            }
        });
        return;
    }

    signal.with_mut(|state| {
        match action {
            SidebarAction::DataChanged => {
                environment.storage.with(|d| {
                    state.list_names = d
                        .timelines
                        .iter()
                        .filter(|e| !e.0.is_empty())
                        .map(|(key, value)| (key.clone(), value.title.clone()))
                        .collect();
                    state.accounts = match state
                        .selected_list
                        .as_ref()
                        .and_then(|e| d.timelines.get(e.as_str()))
                    {
                        Some(n) => n.entries.iter().cloned().collect(),
                        None => d.accounts().iter().cloned().collect(),
                    };
                });
            }
            SidebarAction::FavoritesChanged => {
                if let Some(n) = environment.settings.favorites() {
                    state.favorites = n;
                }
            }
            SidebarAction::LoadTimeline => {
                // Modern Dioxus: Use spawn for async operations in components
            }
            SidebarAction::LoadMoreTimeline => {
                // Modern Dioxus: Use spawn for async operations in components
            }
            SidebarAction::HomeTimeline(statuses) => {
                state
                    .home_timeline
                    .retain(|status| !statuses.iter().any(|new_status| new_status.id == status.id));
                for status in statuses {
                    state.home_timeline.push_back(status);
                }
            }
            SidebarAction::MoreTimeline(statuses) => {
                for status in statuses {
                    state.home_timeline.push_back(status);
                }
            }
            SidebarAction::Notifications => {
                // Modern Dioxus: Use spawn for async operations in components
            }
            SidebarAction::LoadNotifications => {
                state.tab = UiTab::Mentions;
                // Modern Dioxus: Use spawn for async operations in components
            }
            SidebarAction::LoadLists => {
                // Modern Dioxus: Use spawn for async operations in components
            }
            SidebarAction::LoadList(_entry) => {
                // Modern Dioxus: Use spawn for async operations in components
            }
            SidebarAction::ListTimeline(entry, statuses) => {
                state.list_statuses.insert(entry.id.clone(), statuses);
            }
            SidebarAction::ListsChanged(lists) => {
                state.lists = lists;
            }
            SidebarAction::ChangeTab(tab) => {
                state.tab = tab;
            }
            SidebarAction::NotificationTimeline(notifications) => {
                state.notifications = notifications;
            }
            SidebarAction::LoadMoreNotifications => {
                // Modern Dioxus: Use spawn for async operations in components
            }
            SidebarAction::Reload(force) => {
                if force {
                    state.no_more_load_more.clear();
                }
                // Modern Dioxus: Use spawn for async operations in components
            }
            SidebarAction::SelectAccount(account) => {
                state.selected_account = Some(account);
                // Modern Dioxus: Use component-level message handling
            }
            SidebarAction::Root(_action) => {
                // Modern Dioxus: Use component-level message handling
            }
            SidebarAction::MenuEvent(m) => {
                let tab = match m {
                    MainMenuEvent::Timeline => UiTab::Timeline,
                    MainMenuEvent::Mentions => UiTab::Mentions,
                    MainMenuEvent::Messages => UiTab::Messages,
                    MainMenuEvent::More => UiTab::More,
                    _ => return,
                };
                state.tab = tab;
            }
            SidebarAction::SearchAccounts(search) => {
                if search.trim().is_empty() {
                    state.search_results.clear();
                }
                state.search_term = search;
                // Modern Dioxus: Use spawn for async search operations
            }
            SidebarAction::More(s) => {
                state.more_selection = s;
                // Modern Dioxus: Use component-level message handling
            }
            SidebarAction::SelectedNotifications(account) => {
                state.selected_notifications = Some(account);
            }
            SidebarAction::AppEvent(_event) => {
                // Modern Dioxus: Use component-level message handling
            }
            SidebarAction::Search(term) => {
                state.search_term = term;
                state.is_searching = true;
            }
            SidebarAction::SelectList(entry) => {
                state.selected_list = if entry.is_empty() { None } else { Some(entry) };
            }
            SidebarAction::Initial => {
                // Handled separately above to avoid borrow conflicts
                unreachable!("Initial action should be handled before match statement");
            }
        }
    });
}

#[allow(dead_code)] // Search results update - pending search integration
fn update_search_results(state: &mut SidebarState, results: Vec<Account>) {
    state.search_results = results;
}
