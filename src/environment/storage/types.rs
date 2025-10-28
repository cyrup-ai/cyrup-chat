use chrono::{DateTime, Utc};
use dioxus::prelude::WritableExt;
use im::HashMap;

use crate::components::conversation::Conversation;
use crate::environment::model::Account;
use crate::view_model::*;

const LOCAL_TIMELINE_KEY: &str = "";

#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub enum UiTab {
    #[default]
    Timeline,
    Mentions,
    Messages,
    More,
}

impl UiTab {
    pub fn is_timeline(&self) -> bool {
        matches!(self, UiTab::Timeline)
    }

    pub fn is_mentions(&self) -> bool {
        matches!(self, UiTab::Mentions)
    }

    pub fn is_messages(&self) -> bool {
        matches!(self, UiTab::Messages)
    }

    pub fn is_more(&self) -> bool {
        matches!(self, UiTab::More)
    }
}

#[derive(Clone)]
pub struct Data {
    pub user_account: Option<Account>,
    pub selected_account: Option<AccountViewModel>,

    pub conversations: im::HashMap<StatusId, Conversation>,
    pub notification_accounts: im::Vector<AccountUpdateViewModel>,
    pub notification_posts: im::HashMap<AccountId, Vec<NotificationViewModel>>,

    pub selected_notifications: Option<AccountViewModel>,

    // The content of different lists.
    pub timelines: HashMap<String, TimelineEntry>,

    pub active_tab: UiTab,

    /// Bookmarks can quickly become out of date. We store it here, so that
    /// at least any UI bookmark action can be applied to our internal state
    pub bookmarks: Vec<StatusViewModel>,
    pub favorites: Vec<StatusViewModel>,
    pub conversation_list: Vec<StatusViewModel>,

    pub local_timeline: Vec<StatusViewModel>,
    pub public_timeline: Vec<StatusViewModel>,
    pub classic_timeline: Vec<StatusViewModel>,
    pub account_timeline: im::HashMap<AccountId, Vec<StatusViewModel>>,

    // did we load the maximum history for something?
    pub accounts_no_older_data: im::HashSet<AccountId>,

    // Missing fields that were causing compilation errors
    pub scroll_direction: Option<crate::environment::types::ScrollDirection>,
    pub reload_requested: bool,
    pub reload_timestamp: std::time::SystemTime,
    pub last_menu_event: Option<crate::environment::types::MainMenuEvent>,
    pub menu_event_timestamp: std::time::SystemTime,
    pub last_app_event: Option<crate::environment::types::AppEvent>,
    pub app_event_timestamp: std::time::SystemTime,
    pub parent_signal: Option<dioxus::prelude::Signal<Data>>,
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("user_account", &self.user_account.as_ref().map(|e| &e.id))
            .field(
                "selected_account",
                &self.selected_account.as_ref().map(|e| &e.id),
            )
            .field("conversations", &self.conversations.len())
            .field("notification_accounts", &self.notification_accounts.len())
            .field("notification_posts", &self.notification_posts.len())
            .field(
                "selected_notifications",
                &self.selected_notifications.as_ref().map(|e| &e.id),
            )
            .field("timelines", &self.timelines)
            .field("active_tab", &self.active_tab)
            .field("bookmarks", &self.bookmarks.len())
            .field("accounts_no_older_data", &self.accounts_no_older_data)
            .finish()
    }
}

impl Default for Data {
    fn default() -> Self {
        let mut timelines = HashMap::new();
        timelines.insert(
            LOCAL_TIMELINE_KEY.to_string(),
            TimelineEntry {
                title: crate::loc!("Timeline").to_string(),
                ..Default::default()
            },
        );
        Self {
            user_account: Default::default(),
            selected_account: Default::default(),
            conversations: Default::default(),
            notification_accounts: Default::default(),
            notification_posts: Default::default(),
            selected_notifications: Default::default(),
            timelines,
            active_tab: Default::default(),
            bookmarks: Default::default(),
            favorites: Default::default(),
            conversation_list: Default::default(),
            local_timeline: Default::default(),
            public_timeline: Default::default(),
            classic_timeline: Default::default(),
            account_timeline: Default::default(),
            accounts_no_older_data: Default::default(),
            // Initialize missing fields
            scroll_direction: None,
            reload_requested: false,
            reload_timestamp: std::time::SystemTime::now(),
            last_menu_event: None,
            menu_event_timestamp: std::time::SystemTime::now(),
            last_app_event: None,
            app_event_timestamp: std::time::SystemTime::now(),
            parent_signal: None,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Default)]
pub struct TimelineEntry {
    pub title: String,
    pub id: String,
    pub entries: im::Vector<AccountUpdateViewModel>,
    pub posts: HashMap<AccountId, Vec<StatusViewModel>>,
    /// Only if we're selected, will the timer force an update
    pub last_update: DateTime<Utc>,
}

impl std::fmt::Debug for TimelineEntry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TimelineEntry")
            .field("title", &self.title)
            .field("id", &self.id)
            .field("entries", &self.entries.len())
            .field("posts", &self.posts.len())
            .field("last_update", &self.last_update)
            .finish()
    }
}

// Re-export the LOCAL_TIMELINE_KEY constant
pub const LOCAL_TIMELINE_KEY_EXPORT: &str = LOCAL_TIMELINE_KEY;
impl Data {
    /// Subscribe to changes in storage by setting up parent signal for reactive updates
    #[inline(always)]
    pub fn subscribe_to_changes(&mut self, mut signal: dioxus::prelude::Signal<Data>) {
        self.parent_signal = Some(signal);
        signal.set(self.clone());
    }

    /// Check if timeline has updates by comparing last_update timestamp
    #[inline(always)]
    pub fn has_timeline_updates(&self, identifier: &str) -> bool {
        match self.timelines.get(identifier) {
            Some(timeline) => {
                let five_minutes_ago = chrono::Utc::now() - chrono::Duration::minutes(5);
                timeline.last_update > five_minutes_ago
            }
            None => false, // No timeline exists, so no updates
        }
    }

    /// Update timeline with new statuses and trigger reactive updates
    #[inline(always)]
    pub fn update_timeline_statuses(&mut self, statuses: Vec<StatusViewModel>) {
        for status in statuses {
            match status.visibility {
                Visibility::Public => {
                    self.public_timeline.push(status);
                }
                _ => {
                    self.local_timeline.push(status);
                }
            }
        }

        self.reload_timestamp = std::time::SystemTime::now();
        self.trigger_signal_update();
    }

    /// Refresh timeline data by replacing existing content
    #[inline(always)]
    pub fn refresh_timeline_data(&mut self, statuses: Vec<StatusViewModel>) {
        self.local_timeline.clear();
        self.public_timeline.clear();
        self.classic_timeline.clear();
        self.update_timeline_statuses(statuses);
    }

    /// Add a single status update to appropriate timeline
    #[inline(always)]
    pub fn add_status_update(&mut self, status: StatusViewModel) {
        match status.visibility {
            Visibility::Public => {
                self.public_timeline.push(status);
            }
            _ => {
                self.local_timeline.push(status);
            }
        }

        self.reload_timestamp = std::time::SystemTime::now();
        self.trigger_signal_update();
    }

    /// Add notification to storage and trigger updates
    #[inline(always)]
    pub fn add_notification(&mut self, notification: NotificationViewModel) {
        let account_update = AccountUpdateViewModel::new(&notification.status);
        self.notification_accounts.push_back(account_update);
        self.trigger_signal_update();
    }

    /// Remove status from all collections where it might exist
    #[inline(always)]
    pub fn remove_status(&mut self, status_id: &str) {
        let status_id_obj = StatusId(status_id.to_string());

        self.local_timeline.retain(|s| s.id != status_id_obj);
        self.public_timeline.retain(|s| s.id != status_id_obj);
        self.classic_timeline.retain(|s| s.id != status_id_obj);
        self.bookmarks.retain(|s| s.id != status_id_obj);
        self.favorites.retain(|s| s.id != status_id_obj);

        // Efficiently update account timelines using im::HashMap's update method
        let account_ids_to_update: Vec<_> = self
            .account_timeline
            .iter()
            .filter_map(|(account_id, timeline)| {
                let filtered_timeline: Vec<_> = timeline
                    .iter()
                    .filter(|s| s.id != status_id_obj)
                    .cloned()
                    .collect();
                if filtered_timeline.len() != timeline.len() {
                    Some((account_id.clone(), filtered_timeline))
                } else {
                    None
                }
            })
            .collect();

        for (account_id, filtered_timeline) in account_ids_to_update {
            self.account_timeline.insert(account_id, filtered_timeline);
        }

        self.conversations.remove(&status_id_obj);
        self.reload_timestamp = std::time::SystemTime::now();
        self.trigger_signal_update();
    }

    /// Archive status by moving it from active collections to bookmarks
    #[inline(always)]
    pub fn archive_status(&mut self, status_id: &str) {
        let status_id_obj = StatusId(status_id.to_string());

        let mut archived_status = None;

        if let Some(pos) = self
            .local_timeline
            .iter()
            .position(|s| s.id == status_id_obj)
        {
            archived_status = Some(self.local_timeline.remove(pos));
        } else if let Some(pos) = self
            .public_timeline
            .iter()
            .position(|s| s.id == status_id_obj)
        {
            archived_status = Some(self.public_timeline.remove(pos));
        } else if let Some(pos) = self
            .classic_timeline
            .iter()
            .position(|s| s.id == status_id_obj)
        {
            archived_status = Some(self.classic_timeline.remove(pos));
        }

        if let Some(status) = archived_status {
            self.bookmarks.push(status);
            self.reload_timestamp = std::time::SystemTime::now();
            self.trigger_signal_update();
        }
    }

    /// Add sent message to local timeline
    #[inline(always)]
    pub fn add_sent_message(&mut self, message: StatusViewModel) {
        self.local_timeline.push(message);
        self.reload_timestamp = std::time::SystemTime::now();
        self.trigger_signal_update();
    }

    /// Update conversation context with new statuses
    #[inline(always)]
    pub fn update_conversation_context(
        &mut self,
        conversation_id: String,
        context: megalodon::entities::Context,
    ) {
        let conversation_status_id = StatusId(conversation_id);
        let conversation = Conversation::from_context(context);
        self.conversations
            .insert(conversation_status_id, conversation);
        self.trigger_signal_update();
    }

    /// Clear all conversation context
    #[inline(always)]
    pub fn clear_conversation_context(&mut self) {
        self.conversations.clear();
        self.trigger_signal_update();
    }

    /// Add notification update from megalodon streaming
    #[inline(always)]
    pub fn add_notification_update(&mut self, notification: megalodon::entities::Notification) {
        if let Some(status) = &notification.status {
            let status_vm = StatusViewModel::new(status);
            let account_update = AccountUpdateViewModel::new(&status_vm);
            self.notification_accounts.push_back(account_update);
            self.trigger_signal_update();
        }
    }

    /// Update existing status in all collections where it appears
    #[inline(always)]
    pub fn update_existing_status(&mut self, updated_status: StatusViewModel) {
        let status_id = &updated_status.id;

        // Update Vec collections in-place
        for timeline in [
            &mut self.local_timeline,
            &mut self.public_timeline,
            &mut self.classic_timeline,
            &mut self.bookmarks,
            &mut self.favorites,
        ] {
            if let Some(pos) = timeline.iter().position(|s| &s.id == status_id) {
                timeline[pos] = updated_status.clone();
            }
        }

        // Efficiently update account timelines using im::HashMap's update method
        let account_updates: Vec<_> = self
            .account_timeline
            .iter()
            .filter_map(|(account_id, timeline)| {
                if let Some(pos) = timeline.iter().position(|s| &s.id == status_id) {
                    let mut updated_timeline = timeline.clone();
                    updated_timeline[pos] = updated_status.clone();
                    Some((account_id.clone(), updated_timeline))
                } else {
                    None
                }
            })
            .collect();

        for (account_id, updated_timeline) in account_updates {
            self.account_timeline.insert(account_id, updated_timeline);
        }

        // Efficiently update conversations using im::HashMap's update method
        let conversation_updates: Vec<_> = self
            .conversations
            .iter()
            .filter_map(|(conv_id, conversation)| {
                let mut updated_conversation = conversation.clone();
                if updated_conversation.mutate_post(status_id, &mut |status| {
                    *status = updated_status.clone();
                }) {
                    Some((conv_id.clone(), updated_conversation))
                } else {
                    None
                }
            })
            .collect();

        for (conv_id, updated_conversation) in conversation_updates {
            self.conversations.insert(conv_id, updated_conversation);
        }

        self.reload_timestamp = std::time::SystemTime::now();
        self.trigger_signal_update();
    }

    /// Trigger signal update if parent signal is available
    #[inline(always)]
    fn trigger_signal_update(&self) {
        if let Some(mut signal) = self.parent_signal {
            signal.set(self.clone());
        }
    }
}
