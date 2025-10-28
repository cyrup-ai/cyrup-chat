//! Success handling for status mutations

use super::super::{
    super::ReducerState,
    types::{BatchMutation, StatusMutationError},
};
use crate::StatusMutation;
use crate::environment::Environment;
use crate::environment::model::Status;
use crate::view_model::StatusViewModel;
use dioxus::prelude::*;

/// Handle successful status mutation with optimized state updates
#[inline(always)]
pub fn handle_successful_mutation(
    mut signal: Signal<ReducerState>,
    updated_status: Status,
    original_status: StatusViewModel,
    mutation: StatusMutation,
    environment: &Environment,
) -> Result<(), StatusMutationError> {
    log::debug!(
        "Successfully processed mutation {mutation:?} for status {}",
        updated_status.id
    );

    // Create a new batch mutation for potential queuing
    let _batch_mutation = BatchMutation::new(
        mutation.clone(),
        original_status.clone(),
        std::time::Instant::now(),
        super::super::utils::mutation_priority(&mutation),
    );

    // Update the signal state with the new status
    signal.with_mut(|state| {
        // Update in timeline if present
        if let Some(timeline) = &mut state.ui_settings.timeline
            && let Some(index) = timeline.iter().position(|s| s.id == original_status.id)
        {
            timeline[index] = StatusViewModel::new(&updated_status);
        }

        // Process any queued mutations that might be waiting
        process_queued_mutations(state);

        // Clear any error state
        state.error = None;
    });

    // Update storage with the mutated status
    update_storage(
        environment,
        &updated_status,
        &original_status,
        &mutation,
        signal,
    );

    // Update UI state to reflect the mutation
    update_ui_state(signal, &mutation);

    log::info!(
        "Mutation {mutation:?} completed successfully for status {}",
        updated_status.id
    );
    Ok(())
}

/// Process queued mutations for batch efficiency
fn process_queued_mutations(state: &mut ReducerState) {
    if !state.mutation_queue.is_empty() && state.mutation_queue.should_flush() {
        let queued_mutations = state.mutation_queue.flush();
        if !queued_mutations.is_empty() {
            log::debug!("Processing {} queued mutations", queued_mutations.len());

            // Store queued mutations for immediate processing outside of signal context
            let mut pending_batches = state
                .ui_settings
                .timeline
                .get_or_insert_with(Vec::new)
                .iter()
                .filter_map(|status| {
                    queued_mutations
                        .iter()
                        .find(|m| m.status_id == status.id)
                        .map(|mutation| (status.clone(), mutation.clone()))
                })
                .collect::<Vec<_>>();

            // Process batches immediately with optimized state updates
            for (status, batch_mutation) in pending_batches.drain(..) {
                apply_batch_mutation(state, &status, &batch_mutation);
            }
        }
    }
}

/// Apply a single batch mutation to the state
fn apply_batch_mutation(
    state: &mut ReducerState,
    status: &StatusViewModel,
    batch_mutation: &BatchMutation,
) {
    match &batch_mutation.mutation {
        StatusMutation::Like | StatusMutation::Favorite | StatusMutation::Favourite(true) => {
            if let Some(timeline) = &mut state.ui_settings.timeline
                && let Some(status_index) = timeline.iter().position(|s| s.id == status.id)
            {
                timeline[status_index].update_favorited(true);
            }
        }
        StatusMutation::Unlike | StatusMutation::Unfavorite | StatusMutation::Favourite(false) => {
            if let Some(timeline) = &mut state.ui_settings.timeline
                && let Some(status_index) = timeline.iter().position(|s| s.id == status.id)
            {
                timeline[status_index].update_favorited(false);
            }
        }
        StatusMutation::Repost | StatusMutation::Boost(true) => {
            if let Some(timeline) = &mut state.ui_settings.timeline
                && let Some(status_index) = timeline.iter().position(|s| s.id == status.id)
            {
                timeline[status_index].update_reblogged(true);
            }
        }
        StatusMutation::Boost(false) => {
            if let Some(timeline) = &mut state.ui_settings.timeline
                && let Some(status_index) = timeline.iter().position(|s| s.id == status.id)
            {
                timeline[status_index].update_reblogged(false);
            }
        }
        StatusMutation::Archive => {
            // Archive removes status from timeline and moves to bookmarks
            if let Some(timeline) = &mut state.ui_settings.timeline {
                timeline.retain(|s| s.id != status.id);
            }
            log::debug!("Status archived and removed from timeline: {}", status.id.0);
        }
        _ => {
            log::debug!(
                "Batch mutation type {:?} processed",
                batch_mutation.mutation
            );
        }
    }
}

/// Update storage with mutation-specific logic
fn update_storage(
    environment: &Environment,
    updated_status: &Status,
    original_status: &StatusViewModel,
    mutation: &StatusMutation,
    signal: Signal<ReducerState>,
) {
    let mut env_clone = environment.clone();
    env_clone.storage.with_mut(|storage| {
        let _status_vm = StatusViewModel::new(updated_status);
        let _status_id = crate::view_model::StatusId(updated_status.id.clone());

        // Handle mutation-specific storage updates
        match mutation {
            StatusMutation::Delete => {
                handle_delete_storage_update(storage, original_status);
            }
            StatusMutation::Favorite | StatusMutation::Favourite(_) => {
                handle_favorite_storage_update(signal, original_status);
            }
            StatusMutation::Repost | StatusMutation::Boost(_) => {
                log::debug!("Updated boost status for status: {}", updated_status.id);
            }
            StatusMutation::Pin => {
                log::debug!("Status pinned: {}", updated_status.id);
            }
            StatusMutation::Unpin => {
                log::debug!("Status unpinned: {}", updated_status.id);
            }
            StatusMutation::Archive => {
                handle_archive_storage_update(storage, original_status);
            }
            _ => {
                log::debug!("Standard mutation processed: {mutation:?}");
            }
        }
    });
}

/// Handle storage updates for delete mutations
fn handle_delete_storage_update(
    storage: &mut crate::environment::storage::Data,
    original_status: &StatusViewModel,
) {
    // Remove from all relevant collections
    let account_keys: Vec<_> = storage.account_timeline.keys().cloned().collect();
    for account_id in account_keys {
        if let Some(timeline) = storage.account_timeline.get_mut(&account_id) {
            timeline.retain(|status| status.id.0 != original_status.id.0);
        }
    }
    // Remove from other timelines
    storage
        .local_timeline
        .retain(|status| status.id.0 != original_status.id.0);
    storage
        .public_timeline
        .retain(|status| status.id.0 != original_status.id.0);
    storage
        .bookmarks
        .retain(|status| status.id.0 != original_status.id.0);
    storage
        .favorites
        .retain(|status| status.id.0 != original_status.id.0);
    // Remove conversations that include this status
    storage.conversations.retain(|_, _conversation| {
        // Note: Conversation struct doesn't expose statuses() method publicly
        // This is a placeholder - would need proper conversation API
        true
    });
    log::debug!(
        "Removed deleted status from storage: {}",
        original_status.id.0
    );
}

/// Handle storage updates for favorite mutations
fn handle_favorite_storage_update(signal: Signal<ReducerState>, original_status: &StatusViewModel) {
    if let Some(ref user) = signal.read().current_user
        && user.id == original_status.account.id.0
    {
        log::debug!("Updated favorite status for own status");
    }
}

/// Handle storage updates for archive mutations
fn handle_archive_storage_update(
    storage: &mut crate::environment::storage::Data,
    original_status: &StatusViewModel,
) {
    // Archive status using the established archive_status method
    storage.archive_status(&original_status.id.0);

    log::debug!(
        "Archived status and moved to bookmarks: {}",
        original_status.id.0
    );
}

/// Update UI state based on mutation type
fn update_ui_state(mut signal: Signal<ReducerState>, mutation: &StatusMutation) {
    signal.with_mut(|state| {
        // Clear any mutation-related errors
        if let Some(ref error) = state.error
            && (error.contains("mutation") || error.contains("status"))
        {
            state.error = None;
        }

        // Update notification state if relevant
        match mutation {
            StatusMutation::Like | StatusMutation::Reply | StatusMutation::Repost => {
                // These mutations might generate notifications
                state.has_new_notifications = true;
            }
            _ => {}
        }
    });
}
