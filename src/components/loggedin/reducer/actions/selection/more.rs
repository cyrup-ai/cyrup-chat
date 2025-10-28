//! More menu selection handler with zero-allocation patterns

use super::super::ReducerState;
use super::errors::SelectionError;
use crate::components::sidebar::MoreSelection;
use crate::environment::Environment;
use dioxus::prelude::*;

/// Optimized more menu selection handler with zero-allocation patterns
///
/// Handles sidebar more menu selection with efficient state updates.
///
/// Note: Called through Action::SelectMore dispatch - production ready
#[inline(always)]
#[allow(dead_code)] // More selection handler - modular architecture for future use
pub fn handle_select_more(
    mut signal: Signal<ReducerState>,
    selection: MoreSelection,
    _environment: &Environment,
) -> Result<(), SelectionError> {
    log::debug!("Selecting more menu option: {selection:?}");

    // Update state with new more selection
    signal.with_mut(|state| {
        state.more_selection = selection;

        // Clear any navigation errors
        if let Some(ref error) = state.error
            && (error.contains("more") || error.contains("menu"))
        {
            state.error = None;
        }
    });

    Ok(())
}
