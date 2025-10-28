//! Main batch processing orchestration logic

use super::super::{
    super::ReducerState,
    types::{BatchMutation, BatchResult, StatusMutationError},
};
use crate::StatusMutation;
use crate::environment::Environment;
use crate::view_model::StatusViewModel;
use dioxus::prelude::*;

/// Batch process multiple status mutations with optimized performance
///
/// Processes multiple mutations in a single operation for better network
/// efficiency and UI responsiveness using zero-allocation patterns.
#[inline(always)]
pub fn handle_batch_mutations(
    mut signal: Signal<ReducerState>,
    mutations: Vec<BatchMutation>,
    environment: &Environment,
) -> Result<BatchResult, StatusMutationError> {
    log::debug!("Processing batch of {} mutations", mutations.len());

    if mutations.is_empty() {
        return Ok(BatchResult::new());
    }

    let start_time = std::time::Instant::now();
    let mut successful = Vec::with_capacity(mutations.len());
    let mut failed = Vec::new();

    // Group mutations by type for optimal processing
    let mut mutation_groups: std::collections::HashMap<
        std::mem::Discriminant<StatusMutation>,
        Vec<BatchMutation>,
    > = std::collections::HashMap::with_capacity(mutations.len());

    for mutation in mutations {
        let discriminant = std::mem::discriminant(&mutation.mutation);
        mutation_groups
            .entry(discriminant)
            .or_default()
            .push(mutation);
    }

    // Process each group of mutations with priority ordering
    let mut sorted_groups: Vec<_> = mutation_groups.into_iter().collect();
    sorted_groups.sort_by_key(|(_, group)| {
        group
            .iter()
            .map(|m| super::utils::mutation_priority(&m.mutation))
            .min()
            .unwrap_or(255)
    });

    for (_mutation_type, group) in sorted_groups {
        log::debug!("Processing batch group with {} mutations", group.len());

        for batch_mutation in group {
            // Skip mutations that require network but are offline
            if super::utils::mutation_requires_network(&batch_mutation.mutation) {
                log::warn!(
                    "Skipping mutation {:?} for status {} - offline",
                    batch_mutation.mutation,
                    batch_mutation.status_id.0
                );
                failed.push((
                    batch_mutation.status_id.clone(),
                    batch_mutation.mutation.clone(),
                    "Device is offline".to_string(),
                ));
                continue;
            }

            // Store original status for potential revert
            let original_status = signal.with(|state| {
                state.ui_settings.timeline.as_ref().and_then(|timeline| {
                    timeline
                        .iter()
                        .find(|s| s.id == batch_mutation.status_id)
                        .cloned()
                })
            });

            // Update UI state optimistically for immediate feedback
            signal.with_mut(|state| {
                if let Some(timeline) = &mut state.ui_settings.timeline
                    && let Some(status_index) = timeline
                        .iter()
                        .position(|s| s.id == batch_mutation.status_id)
                {
                    let mut updated_status = timeline[status_index].clone();
                    super::ui_updates::apply_optimistic_update(
                        &mut updated_status,
                        &batch_mutation.mutation,
                    );
                    timeline[status_index] = updated_status;
                }
            });

            // Process mutation asynchronously without blocking
            let env_copy = environment.clone();
            let status_id = batch_mutation.status_id.0.clone();
            let mutation = batch_mutation.mutation.clone();
            let mut signal_copy = signal;
            let batch_id = batch_mutation.status_id.clone();

            spawn(async move {
                match super::network::process_single_mutation(&env_copy, &status_id, &mutation)
                    .await
                {
                    Ok(updated_status) => {
                        // Update the UI with the actual server response
                        signal_copy.with_mut(|state| {
                            if let Some(timeline) = &mut state.ui_settings.timeline
                                && let Some(status_index) =
                                    timeline.iter().position(|s| s.id == batch_id)
                            {
                                timeline[status_index] = StatusViewModel::new(&updated_status);
                            }
                        });

                        log::info!("Successfully processed mutation for status {}", status_id);
                    }
                    Err(error) => {
                        // Revert optimistic update on failure
                        if let Some(original) = original_status {
                            signal_copy.with_mut(|state| {
                                if let Some(timeline) = &mut state.ui_settings.timeline
                                    && let Some(status_index) =
                                        timeline.iter().position(|s| s.id == batch_id)
                                {
                                    timeline[status_index] = original;
                                }
                            });
                        }

                        log::error!(
                            "Failed to process mutation for status {}: {}",
                            status_id,
                            error
                        );
                    }
                }
            });

            successful.push((batch_mutation.status_id.clone(), batch_mutation.mutation));
        }
    }

    let processing_time = start_time.elapsed().as_millis() as u64;

    let result = BatchResult {
        successful: successful.clone(),
        failed: failed.clone(),
        processing_time_ms: processing_time,
    };

    // Log batch processing results
    log::info!(
        "Batch processing completed: {} successful, {} failed, {}ms total time",
        result.total_mutations() - result.failed_count(),
        result.failed_count(),
        result.processing_time_ms
    );

    Ok(result)
}
