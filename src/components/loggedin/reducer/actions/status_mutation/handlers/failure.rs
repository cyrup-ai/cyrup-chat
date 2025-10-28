//! Failure handling for status mutations

use super::super::{
    super::ReducerState,
    types::{BatchMutation, StatusMutationError},
    utils::mutation_description,
};
use crate::StatusMutation;
use crate::environment::Environment;
use crate::view_model::StatusViewModel;
use dioxus::prelude::*;

/// Handle failed status mutation with appropriate error handling
#[inline(always)]
pub fn handle_failed_mutation(
    mut signal: Signal<ReducerState>,
    error: String,
    status: StatusViewModel,
    mutation: StatusMutation,
    _environment: &Environment,
) -> Result<(), StatusMutationError> {
    log::error!(
        "Mutation {mutation:?} failed for status {}: {error}",
        status.id.0
    );

    // Create a new batch mutation for retry if appropriate
    let batch_mutation = BatchMutation::new(
        mutation.clone(),
        status.clone(),
        std::time::Instant::now(),
        1, // High priority for retries
    );

    // Determine the specific error type based on the error message
    let mutation_error = categorize_error(&error, &status);

    // Update state with error information and queue retry
    signal.with_mut(|state| {
        state.error = Some(format!(
            "Failed to {}: {error}",
            mutation_description(&mutation)
        ));

        // Queue for retry if the queue isn't full
        if state.mutation_queue.len() < 5 {
            // Max 5 retries
            state.mutation_queue.enqueue(batch_mutation);
        }
    });

    Err(mutation_error)
}

/// Categorize error based on error message content
fn categorize_error(error: &str, status: &StatusViewModel) -> StatusMutationError {
    if error.contains("not found") || error.contains("404") {
        StatusMutationError::status_not_found(format!(
            "Status {} not found: {}",
            status.id.0, error
        ))
    } else if error.contains("network") || error.contains("connection") || error.contains("timeout")
    {
        StatusMutationError::network_error(format!(
            "Network error for status {}: {}",
            status.id.0, error
        ))
    } else {
        StatusMutationError::mutation_failed(format!(
            "Mutation failed for status {}: {}",
            status.id.0, error
        ))
    }
}
