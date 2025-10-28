//! Utility functions for status mutations
//!
//! This module provides helper functions and utilities for status
//! mutation operations with zero-allocation patterns.

use crate::StatusMutation;

/// Get a human-readable description of a mutation for error messages
///
/// This function provides consistent mutation descriptions across
/// the application for user-facing error messages.
#[inline(always)]
pub fn mutation_description(mutation: &StatusMutation) -> &'static str {
    match mutation {
        StatusMutation::Create => "create status",
        StatusMutation::Update => "update status",
        StatusMutation::Delete => "delete status",
        StatusMutation::Like => "like status",
        StatusMutation::Unlike => "unlike status",
        StatusMutation::Repost => "repost status",
        StatusMutation::Reply => "reply to status",
        StatusMutation::Pin => "pin status",
        StatusMutation::Unpin => "unpin status",
        StatusMutation::Archive => "archive status",
        StatusMutation::Favorite => "favorite status",
        StatusMutation::Unfavorite => "unfavorite status",
        StatusMutation::Bookmark(_) => "bookmark status",
        StatusMutation::Favourite(_) => "favourite status",
        StatusMutation::Boost(_) => "boost status",
    }
}

/// Get the priority level of a mutation for batch processing
///
/// This function assigns priority levels to mutations to determine
/// processing order in batch operations.
#[inline(always)]
pub fn mutation_priority(mutation: &StatusMutation) -> u8 {
    match mutation {
        // High priority - user interaction feedback
        StatusMutation::Like
        | StatusMutation::Unlike
        | StatusMutation::Favorite
        | StatusMutation::Unfavorite
        | StatusMutation::Favourite(_)
        | StatusMutation::Boost(_) => 1,

        // Medium priority - content operations
        StatusMutation::Repost | StatusMutation::Reply | StatusMutation::Bookmark(_) => 2,

        // Lower priority - management operations
        StatusMutation::Pin | StatusMutation::Unpin | StatusMutation::Archive => 3,

        // Lowest priority - destructive operations
        StatusMutation::Delete => 4,

        // Default priority for other operations
        _ => 2,
    }
}
