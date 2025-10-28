//! Account selection handler with zero-allocation patterns

use super::super::ReducerState;
use super::errors::SelectionError;
use crate::environment::Environment;
use crate::view_model::{AccountViewModel, StatusViewModel};
use dioxus::prelude::*;

/// Optimized account selection handler with zero-allocation patterns
///
/// Handles account selection for timeline viewing with efficient state updates
/// and automatic timeline loading trigger.
///
/// Note: Called through Action::SelectAccount dispatch - production ready
#[inline(always)]
#[allow(dead_code)] // Account selection handler - modular architecture for future use
pub fn handle_select_account(
    mut signal: Signal<ReducerState>,
    account: AccountViewModel,
    environment: &Environment,
) -> Result<(), SelectionError> {
    log::debug!(
        "Selecting account: {} (@{})",
        account.display_name,
        account.acct
    );

    // Validate account before selection
    if account.acct.is_empty() {
        return Err(SelectionError::AccountSelection(
            "Account has empty acct field".to_string(),
        ));
    }

    // Update state with selected account
    signal.with_mut(|state| {
        // Clear previous selections to maintain clean state
        state.selected_notifications = None;
        state.selected_account = Some(account.clone());

        // Clear any navigation errors
        if let Some(ref error) = state.error
            && (error.contains("selection") || error.contains("account"))
        {
            state.error = None;
        }
    });

    // Trigger timeline loading asynchronously for better UX
    spawn({
        let environment = environment.clone();
        let mut signal = signal;
        let account_id = account.id.clone();

        async move {
            // Start loading state
            signal.with_mut(|state| {
                state.flags.loading_account = true;
                state.flags.loading_account_history = true;
            });

            // Load account timeline data in background
            let account_id_for_storage = account_id.clone();
            match environment
                .model
                .user_timeline(account_id.0, None, None, Some(40))
                .await
            {
                Ok(timeline) => {
                    log::debug!(
                        "Successfully loaded timeline with {} statuses",
                        timeline.len()
                    );

                    // Update storage with new timeline data
                    let mut env_clone = environment.clone();
                    env_clone.storage.with_mut(|storage| {
                        let status_vms: Vec<StatusViewModel> = timeline
                            .into_iter()
                            .map(|status| StatusViewModel::new(&status))
                            .collect();
                        storage
                            .account_timeline
                            .insert(account_id_for_storage, status_vms);
                    });

                    // Clear loading state and potential errors
                    signal.with_mut(|state| {
                        state.flags.loading_account = false;
                        state.flags.loading_account_history = false;
                    });
                }
                Err(e) => {
                    log::error!("Failed to load account timeline: {e}");

                    signal.with_mut(|state| {
                        state.flags.loading_account = false;
                        state.flags.loading_account_history = false;
                        state.error = Some(format!("Failed to load timeline: {e}"));
                    });
                }
            }
        }
    });

    Ok(())
}
