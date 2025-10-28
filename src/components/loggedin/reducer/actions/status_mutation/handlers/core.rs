//! Core handler entry point and result routing

use super::super::{super::ReducerState, types::StatusMutationError};
use crate::StatusMutation;
use crate::environment::Environment;
use crate::environment::model::Status;
use crate::view_model::StatusViewModel;
use dioxus::prelude::*;

/// Optimized status mutation result handler with batch processing support
///
/// Handles the result of status mutation operations with efficient state updates
/// and batch processing for better performance.
#[inline(always)]
pub fn handle_status_mutation_result(
    signal: Signal<ReducerState>,
    result: Result<Status, String>,
    status: StatusViewModel,
    mutation: StatusMutation,
    environment: &Environment,
) -> Result<(), StatusMutationError> {
    log::debug!(
        "Processing status mutation result: {mutation:?} for status {}",
        status.id.0
    );

    // Validate input parameters
    if status.id.0.is_empty() {
        return Err(StatusMutationError::invalid_mutation("Status ID is empty"));
    }

    match result {
        Ok(updated_status) => super::success::handle_successful_mutation(
            signal,
            updated_status,
            status,
            mutation,
            environment,
        ),
        Err(error) => {
            super::failure::handle_failed_mutation(signal, error, status, mutation, environment)
        }
    }
}
