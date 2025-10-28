use super::types::Data;
use crate::environment::model::Status;
use crate::view_model::*;
use std::collections::HashSet;

impl Data {
    pub(super) fn general_merge(
        into: &mut Vec<StatusViewModel>,
        items: &[Status],
        is_reload: bool,
        limit_count: Option<usize>,
    ) {
        let existing: HashSet<_> = into.iter().map(|e| e.id.0.clone()).collect();
        for entry in items.iter() {
            if !existing.contains(entry.id.as_str()) {
                if is_reload {
                    into.insert(0, StatusViewModel::new(entry))
                } else {
                    into.push(StatusViewModel::new(entry));
                }
            }
        }
        // by default, only keep the newest LIMIT. Otherwise memory piles up
        if let Some(limit) = limit_count
            && into.len() > limit
        {
            let _ = into.drain(limit..).collect::<Vec<_>>();
        }
    }

    pub fn merge_bookmarks(&mut self, bookmarks: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.bookmarks, bookmarks, is_reload, None);
    }

    pub fn merge_favorites(&mut self, favorites: &[Status], is_reload: bool) {
        Self::general_merge(&mut self.favorites, favorites, is_reload, None);
    }

    pub fn merge_conversation_list(
        &mut self,
        conversations: Vec<StatusViewModel>,
        is_reload: bool,
    ) {
        if is_reload {
            // Full reload: replace entire list
            self.conversation_list = conversations;
        } else {
            // Incremental: merge avoiding duplicates
            let existing: HashSet<_> = self
                .conversation_list
                .iter()
                .map(|c| c.id.0.clone())
                .collect();

            for conv in conversations {
                if !existing.contains(&conv.id.0) {
                    self.conversation_list.push(conv);
                }
            }
        }
    }

    pub fn mutate_post(
        &mut self,
        id: StatusId,
        account_id: AccountId,
        mut action: impl FnMut(&mut StatusViewModel),
    ) -> bool {
        let mut found = false;

        if let Some(posts) = self.notification_posts.get_mut(&account_id) {
            for item in posts.iter_mut() {
                if item.status.id == id {
                    action(&mut item.status);
                    found = true;
                }
            }
        }

        // Go through all the timelines
        if let Some(posts) = self.account_timeline.get_mut(&account_id) {
            for item in posts.iter_mut() {
                if item.id == id {
                    action(item);
                    found = true;
                }
            }
        }

        // next, go through all posts & boosts
        for (_, timeline) in self.timelines.iter_mut() {
            for (_, posts) in timeline.posts.iter_mut() {
                for p in posts.iter_mut() {
                    if p.id == id {
                        action(p);
                        found = true;
                    }
                    if let Some(o) = p.reblog_status.as_mut()
                        && o.id == id
                    {
                        action(o);
                        found = true;
                    }
                }
            }
        }

        // bookmarks, favorites, and so on
        for posts in [
            self.bookmarks.iter_mut(),
            self.favorites.iter_mut(),
            self.local_timeline.iter_mut(),
            self.public_timeline.iter_mut(),
            self.classic_timeline.iter_mut(),
        ] {
            for p in posts {
                if p.id == id {
                    action(p);
                    found = true;
                }
                if let Some(o) = p.reblog_status.as_mut()
                    && o.id == id
                {
                    action(o);
                    found = true;
                }
            }
        }

        // finally, go through the conversations
        let mut found_conv = false;
        for (_, c) in self.conversations.iter_mut() {
            let r = c.mutate_post(&id, &mut action);
            if r {
                found_conv = r;
            }
        }
        found_conv || found
    }

    pub fn replied_to_status(&mut self, status_id: &str) -> bool {
        for (_, timeline) in self.timelines.iter_mut() {
            for (_, stati) in timeline.posts.iter_mut() {
                for status in stati.iter_mut() {
                    if status.id.0 == status_id {
                        status.did_reply();
                        return true;
                    }
                }
            }
        }

        for (_, stati) in self.notification_posts.iter_mut() {
            for notification in stati.iter_mut() {
                if notification.status.id.0 == status_id {
                    notification.status.did_reply();
                    return true;
                }
            }
        }
        false
    }

    pub fn changed_bookmark(&mut self, status: &StatusViewModel, added: bool) {
        if added {
            self.bookmarks.insert(0, status.clone());
        } else {
            let Some(idx) = self.bookmarks.iter().position(|a| a.id == status.id) else {
                return;
            };
            self.bookmarks.remove(idx);
        }
    }

    pub fn changed_favorite(&mut self, status: &StatusViewModel, added: bool) {
        if added {
            self.favorites.insert(0, status.clone());
        } else {
            let Some(idx) = self.favorites.iter().position(|a| a.id == status.id) else {
                return;
            };
            self.favorites.remove(idx);
        }
    }

    pub fn changed_boost(&mut self, _status: &StatusViewModel, _added: bool) {
        // not sure we need to do anything here
    }
}
