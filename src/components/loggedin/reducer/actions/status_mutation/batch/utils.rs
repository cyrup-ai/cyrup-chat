//! Utility functions for batch processing optimization

use crate::StatusMutation;

/// Check if a mutation requires network access for optimization
#[inline(always)]
pub fn mutation_requires_network(mutation: &StatusMutation) -> bool {
    match mutation {
        StatusMutation::Like
        | StatusMutation::Unlike
        | StatusMutation::Repost
        | StatusMutation::Delete
        | StatusMutation::Pin
        | StatusMutation::Unpin
        | StatusMutation::Create
        | StatusMutation::Update
        | StatusMutation::Reply
        | StatusMutation::Archive
        | StatusMutation::Favorite
        | StatusMutation::Unfavorite
        | StatusMutation::Bookmark(_)
        | StatusMutation::Favourite(_)
        | StatusMutation::Boost(_) => true,
    }
}

/// Get mutation priority for optimal processing order
#[inline(always)]
pub fn mutation_priority(mutation: &StatusMutation) -> u8 {
    match mutation {
        StatusMutation::Delete => 0, // Highest priority
        StatusMutation::Pin | StatusMutation::Unpin => 1,
        StatusMutation::Like
        | StatusMutation::Unlike
        | StatusMutation::Favorite
        | StatusMutation::Unfavorite
        | StatusMutation::Favourite(_) => 2,
        StatusMutation::Repost | StatusMutation::Boost(_) => 3,
        StatusMutation::Bookmark(_) => 4,
        StatusMutation::Create
        | StatusMutation::Update
        | StatusMutation::Reply
        | StatusMutation::Archive => 5,
    }
}
