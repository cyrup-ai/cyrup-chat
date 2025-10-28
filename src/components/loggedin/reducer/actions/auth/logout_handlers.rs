//! Logout handlers for authentication state management

use super::super::ReducerState;
use super::error_types::{AuthError, ResultExt};
use crate::environment::Environment;
use crate::environment::storage::UiTab;
use dioxus::prelude::*;

/// Optimized logout handler with comprehensive cleanup
///
/// Consolidates multiple logout handlers:
/// - PublicAction::Logout delegation (lines 602-608)
/// - Action::Logout handler (lines 1289-1316)
#[inline(always)]
pub fn handle_logout(
    mut signal: Signal<ReducerState>,
    environment: &Environment,
) -> Result<(), AuthError> {
    // Set logout state immediately for UI responsiveness
    signal.with_mut(|state| {
        state.flags.logging_in = true;
    });

    // Efficient toolbar cleanup with error recovery
    if let Err(e) = environment
        .platform
        .update_toolbar("", &UiTab::Timeline, true)
    {
        log::warn!("Toolbar cleanup failed (non-critical): {e}");
    }

    // Safe user retrieval with comprehensive error handling
    let user = environment
        .settings
        .users()
        .map_err(|e| AuthError::RepositoryError(format!("Failed to get users: {e}")))?
        .into_iter()
        .next()
        .ok_or_else(|| AuthError::RepositoryError("No user found for logout".to_string()))?;

    // Optimized async logout with proper error propagation
    let environment_clone = environment.clone();
    let user_id = user.id.clone();
    spawn({
        let signal = signal;
        let environment = environment_clone;
        async move {
            // User cleanup with error recovery - moved into async block
            if let Err(e) = environment.settings.remove_user(user_id).await {
                log::error!("User removal failed: {e}");
                let error_result = Err(format!("Failed to remove user: {e}"));
                if let Err(auth_error) = handle_logout_done(signal, error_result, &environment) {
                    log::error!("Logout error handling failed: {auth_error}");
                }
                return;
            }

            let result = environment
                .model
                .logout(user.id.clone())
                .await
                .map_err(|e| format!("Model logout failed: {e}"));

            // Direct function call for performance
            if let Err(auth_error) = handle_logout_done(signal, result, &environment) {
                log::error!("Logout completion failed: {auth_error}");
            }
        }
    });

    Ok(())
}

/// Optimized logout completion handler with menu state updates
///
/// Consolidates Action::LogoutDone handler from lines 1317-1329
#[inline(always)]
pub fn handle_logout_done(
    mut signal: Signal<ReducerState>,
    result: Result<(), String>,
    environment: &Environment,
) -> Result<(), AuthError> {
    // Efficient state cleanup with single signal mutation
    signal.with_mut(|state| {
        state.logged_in = false;
        state.did_logout.set(Some(true));
        state.flags.logging_in = false;

        // Set error state if logout failed
        if let Err(error) = &result {
            state.error = Some(format!("Logout failed: {error}"));
        } else {
            // Clear any previous errors on successful logout
            state.error = None;
        }

        // Reset user-related state
        state.current_user = None;
        state.selected_account = None;
        state.selected_notifications = None;
        state.has_new_notifications = false;
    });

    // Optimized menu cleanup with error recovery
    environment.platform.update_menu(|config| {
        config.logged_in = false;
        config.enable_postwindow = false;
    });

    if result.is_ok() {
        log::info!("Logout completed successfully");
    } else {
        log::error!("Logout failed: {result:?}");
        return Err(AuthError::LogoutFailed(
            result.unwrap_err_or_else(|| "Unknown logout error".to_string()),
        ));
    }

    Ok(())
}
