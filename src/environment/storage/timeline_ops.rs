use megalodon::streaming::Message;
use std::collections::HashSet;

use super::super::types::TimelineDirection;
use super::types::{Data, LOCAL_TIMELINE_KEY_EXPORT as LOCAL_TIMELINE_KEY, TimelineEntry};
use crate::environment::model::Status;
use crate::view_model::*;

impl Data {
    /// Get accounts from the local timeline with safe error handling
    /// Returns empty vector if timeline doesn't exist instead of panicking
    pub fn accounts(&self) -> im::Vector<AccountUpdateViewModel> {
        self.timelines
            .get(LOCAL_TIMELINE_KEY)
            .map(|timeline| timeline.entries.clone())
            .unwrap_or_else(|| {
                log::warn!("Local timeline not found, returning empty accounts vector");
                im::Vector::new()
            })
    }

    /// Get posts from the local timeline with safe error handling  
    /// Returns empty HashMap if timeline doesn't exist instead of panicking
    pub fn posts(&self) -> im::HashMap<AccountId, Vec<StatusViewModel>> {
        self.timelines
            .get(LOCAL_TIMELINE_KEY)
            .map(|timeline| timeline.posts.clone())
            .unwrap_or_else(|| {
                log::warn!("Local timeline not found, returning empty posts HashMap");
                im::HashMap::new()
            })
    }

    /// Safe method to get a timeline entry
    #[allow(dead_code)] // Timeline entry retrieval - pending timeline integration
    fn get_timeline_entry(&self, key: &str) -> Option<&TimelineEntry> {
        self.timelines.get(key)
    }

    /// Safe method to get a mutable timeline entry
    #[allow(dead_code)] // Timeline entry mutation - pending timeline integration
    fn get_timeline_entry_mut(&mut self, key: &str) -> Option<&mut TimelineEntry> {
        self.timelines.get_mut(key)
    }

    /// Ensure local timeline exists, creating it if missing
    fn ensure_local_timeline(&mut self) {
        if !self.timelines.contains_key(LOCAL_TIMELINE_KEY) {
            log::warn!("Local timeline missing, recreating it");
            self.timelines.insert(
                LOCAL_TIMELINE_KEY.to_string(),
                TimelineEntry {
                    title: crate::loc!("Timeline").to_string(),
                    ..Default::default()
                },
            );
        }
    }

    pub fn merge_localtimeline(&mut self, posts: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.local_timeline, posts, is_reload, Some(350));
    }

    pub fn merge_publictimeline(&mut self, posts: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.public_timeline, posts, is_reload, Some(350));
    }

    pub fn merge_classictimeline(&mut self, posts: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.classic_timeline, posts, is_reload, Some(350));
    }

    pub fn merge_account(&mut self, posts: &[Status], id: &AccountId, is_reload: bool) {
        // if we don't have an entry yet, insert a new one
        let g = self.account_timeline.entry(id.clone()).or_default();
        Self::general_merge(g, posts, is_reload, None);
    }

    pub fn clear_reload(&mut self) -> bool {
        self.timelines.iter_mut().for_each(|l| {
            l.1.entries.clear();
            l.1.posts.clear();
        });
        self.selected_account = None;
        self.selected_notifications = None;
        self.bookmarks.clear();
        self.favorites.clear();
        self.local_timeline.clear();
        self.public_timeline.clear();
        self.account_timeline.clear();
        self.classic_timeline.clear();
        true
    }

    pub fn handle_push_message(&mut self, message: Message, direction: TimelineDirection) {
        match message {
            Message::Update(status) => {
                log::debug!("update classic timeline data");
                self.classic_timeline
                    .insert(0, StatusViewModel::new(&status));
                self.update_account_historical_data(&[status], &direction);
            }
            Message::Notification(notification) => {
                self.update_notifications(&[notification]);
            }
            Message::Conversation(_) => {
                //
            }
            Message::Delete(_) => {
                //
            }
            Message::StatusUpdate(status) => {
                self.update_account_historical_data(&[status], &direction);
            }
            Message::Heartbeat() => {
                //
            }
        }
    }

    /// Remove lists we don't have anymore, add new lists
    pub fn update_timelines(&mut self, timelines: &[(String, String)]) {
        let mut unknown: HashSet<_> = self.timelines.keys().cloned().collect();
        for (name, id) in timelines {
            if self.timelines.contains_key(id) {
                unknown.remove(id);
                self.timelines[id].title = name.clone();
            } else {
                self.timelines.insert(
                    id.clone(),
                    TimelineEntry {
                        title: name.clone(),
                        id: id.clone(),
                        ..Default::default()
                    },
                );
            }
        }
        // remove all unknown
        for id in unknown {
            if id != LOCAL_TIMELINE_KEY {
                self.timelines.remove(&id);
            }
        }
    }

    pub fn update_account_historical_data(
        &mut self,
        updates: &[Status],
        direction: &TimelineDirection,
    ) {
        // Ensure local timeline exists before attempting to update it
        self.ensure_local_timeline();

        // Safe access to timeline entry with proper error handling
        if let Some(timeline) = self.timelines.get_mut(LOCAL_TIMELINE_KEY) {
            Self::update_historical_data(updates, timeline, direction);
        } else {
            log::error!("Failed to access local timeline for historical data update");
        }
    }

    pub fn update_timeline_historical_data(
        &mut self,
        id: &str,
        updates: &[Status],
        direction: &TimelineDirection,
    ) {
        let Some(timeline) = self.timelines.get_mut(id) else {
            return;
        };
        Self::update_historical_data(updates, timeline, direction);
    }

    /// can't have to &mut. Proper solution is to abstract timelines into a struct
    /// that can be used in here (like the list struct)
    fn update_historical_data(
        updates: &[Status],
        timeline: &mut TimelineEntry,
        direction: &TimelineDirection,
    ) -> bool {
        let posts = &mut timeline.posts;
        let accounts = &mut timeline.entries;
        let mut modified_ids = HashSet::new();
        for update in updates.iter() {
            let id = AccountId(update.account.id.clone());
            modified_ids.insert(id.clone());
            let exists = posts.contains_key(&id);
            let new_status = StatusViewModel::new(update);
            if exists {
                // this should never fail, but still
                if let Some(account_idx) = accounts.iter().position(|o| o.id == id) {
                    // update with whatever the new status is
                    if update.created_at > accounts[account_idx].last_updated {
                        accounts[account_idx] = AccountUpdateViewModel::new(&new_status);
                    }
                }
                posts.entry(id).and_modify(|existing| {
                    // if this id already exists, we replace it
                    if let Some(ref pos) = existing.iter().position(|x| x.id.0 == update.id) {
                        existing[*pos] = new_status; // StatusViewModel::new(update);
                    } else {
                        existing.push(new_status);
                    }
                });
            } else {
                accounts.push_back(AccountUpdateViewModel::new(&new_status));
                posts.entry(id).or_insert(vec![new_status]);
            }
        }
        // Sort the accounts by date
        accounts.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));

        // for all affected accounts, sort reverse
        let changed = !modified_ids.is_empty();
        for account in modified_ids {
            if direction == &TimelineDirection::NewestBottom {
                if let Some(e) = posts.get_mut(&account) {
                    e.sort_by(|a, b| a.created.cmp(&b.created))
                }
            } else if let Some(e) = posts.get_mut(&account) {
                e.sort_by(|b, a| a.created.cmp(&b.created))
            }
        }

        changed
    }
}
