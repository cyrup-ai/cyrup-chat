//! Navigation state validation for debugging

use super::super::ReducerState;
use super::errors::SelectionError;
use dioxus::prelude::*;

/// Validate navigation state consistency for debugging
///
/// This function helps ensure navigation state remains consistent
/// and can be used for debugging navigation issues.
#[allow(dead_code)] // Used for debugging navigation state
pub fn validate_navigation_state(signal: Signal<ReducerState>) -> Result<(), SelectionError> {
    signal.with(|state| {
        // Ensure only one account selection type is active at a time
        if state.selected_account.is_some() && state.selected_notifications.is_some() {
            return Err(SelectionError::NavigationState(
                "Both account and notification selections are active simultaneously".to_string(),
            ));
        }

        // Validate loading states are consistent
        if !state.logged_in && (state.flags.loading_account || state.flags.loading_notifications) {
            return Err(SelectionError::NavigationState(
                "Loading states active while not logged in".to_string(),
            ));
        }

        Ok(())
    })
}
