//! Utility action handlers
//!
//! Handles error clearing, data updates, and other utility operations.

use crate::environment::Environment;
use dioxus::prelude::*;

use super::super::{ActionError, ReducerState};

/// Handle error clearing
pub fn handle_clear_error(
    mut signal: Signal<ReducerState>,
    _environment: &mut Environment,
) -> Result<(), ActionError> {
    signal.with_mut(|state| {
        state.error = None;
    });
    Ok(())
}

/// Handle data updates
pub fn handle_data_updated(
    mut signal: Signal<ReducerState>,
    _environment: &mut Environment,
) -> Result<(), ActionError> {
    // Trigger reactive updates
    signal.with_mut(|_| {});
    Ok(())
}
