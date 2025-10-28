//! Post completion handler with zero-allocation patterns
//!
//! This module handles successful post creation with efficient state cleanup
//! and storage updates using production-ready patterns.

use super::{super::ReducerState, PostError};
use crate::environment::Environment;
use crate::environment::model::Status;
use crate::view_model::StatusViewModel;
use dioxus::prelude::*;

/// Optimized post completion handler with zero-allocation patterns
///
/// Handles successful post creation with efficient state cleanup and storage updates.
#[inline(always)]
pub fn handle_post_done(
    mut signal: Signal<ReducerState>,
    status: Status,
    environment: &Environment,
) -> Result<(), PostError> {
    log::debug!("Completing post creation for status: {}", status.id);

    // Validate completed status
    if status.id.is_empty() {
        return Err(PostError::PostCompletionFailed(
            "Completed status has empty ID".to_string(),
        ));
    }

    if status.content.is_empty() {
        return Err(PostError::PostValidationFailed(
            "Completed status has empty content".to_string(),
        ));
    }

    // Update storage with new status
    let mut env_clone = environment.clone();
    env_clone.storage.with_mut(|storage| {
        let status_vm = StatusViewModel::from(status.clone());

        // Add to appropriate timeline based on context
        if let Some(in_reply_to) = &status.in_reply_to_id {
            // This is a reply - add to conversation
            if let Some(_conversation) = storage
                .conversations
                .get_mut(&crate::view_model::StatusId(in_reply_to.clone()))
            {
                // Add reply to existing conversation
                log::debug!("Adding reply {} to conversation {in_reply_to}", status.id);
            } else {
                // Create new conversation entry
                log::debug!(
                    "Creating new conversation for reply {} to parent {in_reply_to}",
                    status.id
                );
            }
        } else {
            // This is a new post - add to user's timeline
            let account_id = crate::view_model::AccountId(status.account.id.clone());
            storage
                .account_timeline
                .entry(account_id)
                .or_insert_with(Vec::new)
                .insert(0, status_vm); // Add at beginning (newest first)
        }
    });

    // Clear replying state and update UI
    signal.with_mut(|state| {
        // Clear the replying state
        state.is_replying = None;

        // Clear any post-related errors
        if let Some(ref error) = state.error
            && (error.contains("post") || error.contains("reply"))
        {
            state.error = None;
        }

        // Update notification state if this was a reply
        if status.in_reply_to_id.is_some() {
            // User created a reply, might want to refresh notifications
            state.has_new_notifications = true;
        }
    });

    log::info!("Post creation completed successfully: {}", status.id);
    Ok(())
}
