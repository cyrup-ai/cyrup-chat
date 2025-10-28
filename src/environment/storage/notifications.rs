use std::collections::HashSet;

use super::types::Data;
use crate::environment::model::Notification;
use crate::view_model::*;

impl Data {
    pub fn update_notifications(&mut self, notifications: &[Notification]) -> bool {
        let posts = &mut self.notification_posts;
        let accounts = &mut self.notification_accounts;

        let mut updated = HashSet::new();
        for notification in notifications.iter() {
            let Some(ref account) = notification.account else {
                continue;
            };
            let id = AccountId(account.id.clone());
            let exists = posts.contains_key(&id);
            let Some(ref status) = notification.status else {
                continue;
            };
            let Some(nm) = NotificationViewModel::new(notification) else {
                continue;
            };
            updated.insert(id.clone());
            let new_status = StatusViewModel::new(status);
            if exists {
                if let Some(account_idx) = accounts.iter().position(|o| o.id == id) {
                    // update with whatever the new status is
                    if status.created_at > accounts[account_idx].last_updated {
                        accounts[account_idx] = AccountUpdateViewModel::new(&new_status);
                    }
                }
                posts.entry(id).and_modify(|existing| {
                    // if this id already exists, we replace it
                    if let Some(ref pos) = existing.iter().position(|x| x.id == notification.id) {
                        existing[*pos] = nm;
                    } else {
                        existing.push(nm)
                    }
                });
            } else {
                accounts.push_back(AccountUpdateViewModel::new(&new_status));
                posts.entry(id).or_insert(vec![nm]);
            }
        }
        // Sort the accounts by date
        accounts.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));

        !updated.is_empty()
    }
}
