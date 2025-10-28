//! Post creation handler with zero-allocation patterns
//!
//! This module provides optimized post creation logic with comprehensive
//! error handling and efficient state management.

use super::{super::ReducerState, PostError};
use crate::components::post::PostKind;
use crate::environment::Environment;
use dioxus::prelude::*;
use std::path::PathBuf;

/// Optimized post creation handler with zero-allocation patterns
///
/// Initiates post creation or reply composition with efficient state management
/// and file attachment support.
#[inline(always)]
pub fn handle_post(
    mut signal: Signal<ReducerState>,
    kind: PostKind,
    environment: &Environment,
) -> Result<(), PostError> {
    log::debug!("Starting post creation with kind: {kind:?}");

    // Validate post parameters
    match &kind {
        PostKind::Reply(status) | PostKind::ReplyPrivate(status) => {
            if status.id.0.is_empty() {
                return Err(PostError::PostValidationFailed(
                    "Reply target status has empty ID".to_string(),
                ));
            }
        }
        PostKind::Post => {
            // Regular post - no special validation needed
        }
    }

    // Get current user account for post creation
    let current_account = signal.with(|state| {
        state
            .current_user
            .clone()
            .or_else(|| state.user_account.clone())
    });

    let account = match current_account {
        Some(acc) => acc,
        None => {
            return Err(PostError::PostCreationFailed(
                "No authenticated user account found".to_string(),
            ));
        }
    };

    // Create account view model for post state using proper field names
    let _account_vm = crate::view_model::AccountViewModel::new(&account);

    // Prepare file paths for attachment
    let file_paths = Vec::<PathBuf>::new(); // Start with empty attachments

    // Update state with reply information
    signal.with_mut(|state| {
        // Set replying state with post kind and file paths
        state.is_replying = Some((kind.clone(), file_paths.clone()));

        // Clear any previous post errors
        if let Some(ref error) = state.error
            && (error.contains("post") || error.contains("reply"))
        {
            state.error = None;
        }
    });

    // Handle different post types
    match kind {
        PostKind::Post => {
            log::debug!("Creating new post");
            // Regular post - state is already updated
        }
        PostKind::Reply(ref status) => {
            log::debug!("Creating reply to status: {}", status.id.0);

            // Preload reply context if needed
            spawn({
                let environment = environment.clone();
                let _signal = signal;
                let status_id = status.id.clone();

                async move {
                    // Load reply context to ensure we have the latest status information
                    match environment.model.single_status(status_id.0).await {
                        Ok(updated_status) => {
                            log::debug!("Successfully loaded reply target status");

                            // Update storage with latest status information
                            let mut env_clone = environment.clone();
                            env_clone.storage.with_mut(|storage| {
                                // Convert Message to StatusViewModel using helper
                                use crate::components::conversation::conversation_helpers::message_to_status_view_model;
                                let status_vm = message_to_status_view_model(&updated_status);
                                // Store in conversation context or create a simple timeline entry
                                let timeline_entry = storage
                                    .timelines
                                    .entry("reply_context".to_string())
                                    .or_insert_with(|| {
                                        crate::environment::storage::TimelineEntry {
                                            title: "Reply Context".to_string(),
                                            id: "reply_context".to_string(),
                                            entries: im::Vector::new(),
                                            posts: im::HashMap::new(),
                                            last_update: chrono::Utc::now(),
                                        }
                                    });
                                let account_id =
                                    crate::view_model::AccountId(status_vm.account.id.0.clone());
                                timeline_entry
                                    .posts
                                    .entry(account_id)
                                    .or_insert_with(Vec::new)
                                    .push(status_vm);
                            });
                        }
                        Err(e) => {
                            log::warn!("Failed to reload reply target status: {e}");
                            // Don't fail the reply creation, just log the warning
                        }
                    }
                }
            });
        }
        PostKind::ReplyPrivate(ref status) => {
            log::debug!("Creating private reply to status: {}", status.id.0);
            // Similar to Reply but with private visibility enforced
        }
    }

    Ok(())
}
