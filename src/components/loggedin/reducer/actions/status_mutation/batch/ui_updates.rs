//! Optimistic UI updates for immediate user feedback

use crate::StatusMutation;
use crate::view_model::StatusViewModel;

/// Apply optimistic UI update for immediate user feedback
#[inline(always)]
pub fn apply_optimistic_update(status: &mut StatusViewModel, mutation: &StatusMutation) {
    match mutation {
        StatusMutation::Like | StatusMutation::Favorite | StatusMutation::Favourite(true) => {
            status.update_favorited(true);
        }
        StatusMutation::Unlike | StatusMutation::Unfavorite | StatusMutation::Favourite(false) => {
            status.update_favorited(false);
        }
        StatusMutation::Repost | StatusMutation::Boost(true) => {
            status.update_reblogged(true);
        }
        StatusMutation::Boost(false) => {
            status.update_reblogged(false);
        }
        StatusMutation::Bookmark(true) => {
            // Bookmark UI update if needed
        }
        StatusMutation::Bookmark(false) => {
            // Unbookmark UI update if needed
        }
        StatusMutation::Create
        | StatusMutation::Update
        | StatusMutation::Delete
        | StatusMutation::Reply
        | StatusMutation::Pin
        | StatusMutation::Unpin
        | StatusMutation::Archive => {} // Other mutations don't have immediate UI feedback
    }
}
