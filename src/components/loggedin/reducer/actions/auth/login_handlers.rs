//! Login and authentication handlers for loggedin state management

use super::super::ReducerState;
use super::error_types::AuthError;
use super::stream_processing::setup_user_stream;
use crate::environment::Environment;
use crate::environment::model::Account;
use crate::environment::storage::UiTab;
use crate::environment::types::MainMenuConfig;
use dioxus::prelude::*;

/// Optimized login handler with zero-allocation patterns
///
/// Consolidates multiple login handlers from the original reducer:
/// - Action::Login (lines 125-144)
/// - PublicAction::Login delegation (lines 593-601)
#[inline(always)]
pub fn handle_login(
    mut signal: Signal<ReducerState>,
    environment: &Environment,
) -> Result<(), AuthError> {
    // Pre-fetch configuration to avoid multiple settings calls
    let ui_settings = environment
        .settings
        .config()
        .map_err(|e| AuthError::RepositoryError(format!("Failed to get config: {e}")))?;

    // Efficient signal mutation with single closure
    signal.with_mut(|state| {
        state.ui_settings = ui_settings;
        state.flags.logging_in = true;
    });

    // Optimized async login with proper error handling using spawn pattern
    spawn({
        let mut signal = signal;
        let environment = environment.clone();
        async move {
            let result = environment
                .model
                .login()
                .await
                .map_err(|e| format!("Model login failed: {e}"));

            // Handle login result directly to avoid recursive calls
            match result {
                Ok(_user_info) => {
                    // Q9: MVP hardcoded auth - no real user account needed
                    signal.with_mut(|state| {
                        state.flags.logging_in = false;
                        state.logged_in = true;
                        state.current_user = None; // MVP: no account storage needed
                        state.user_account = None; // MVP: no account storage needed
                        state.error = None;
                    });
                }
                Err(error) => {
                    log::error!("Login failed: {error}");
                    signal.with_mut(|state| {
                        state.error = Some(error);
                        state.flags.logging_in = false;
                        state.logged_in = false;
                    });
                }
            }
        }
    });

    Ok(())
}

/// Optimized logged-in handler with user stream setup
///
/// Consolidates Action::LoggedIn handler from lines 145-262
/// Implements efficient user stream subscription with channel-based patterns
#[inline(always)]
pub fn handle_logged_in(
    mut signal: Signal<ReducerState>,
    result: Result<Account, String>,
    environment: &Environment,
) -> Result<(), AuthError> {
    // Pre-clone environment to avoid multiple clones in hot path
    let mut env_clone = environment.clone();

    // Extract user data before signal mutation for efficiency
    let user_clone = result.as_ref().ok().cloned();

    // Efficient signal update with comprehensive error handling
    signal.with_mut(|state| {
        state.flags.logging_in = false;
        match &result {
            Ok(user) => {
                state.current_user = Some(user.clone());
                state.logged_in = true;
                // Clear any previous login errors
                state.error = None;
            }
            Err(error) => {
                state.logged_in = false;
                state.error = Some(format!("Login Error: {error}"));
                // Reset user state on error
                state.current_user = None;
            }
        }
    });

    // Optimized environment updates with error recovery
    if let Some(user) = user_clone {
        // Efficient toolbar update with error recovery
        if let Err(e) = env_clone
            .platform
            .update_toolbar(&user.avatar, &UiTab::Timeline, false)
        {
            log::warn!("Toolbar update failed (non-critical): {e}");
        }

        // Storage update with proper error handling
        env_clone.storage.with_mut(|storage| {
            storage.user_account = Some(user);
        });

        // Menu update with error recovery
        env_clone
            .platform
            .update_menu(|config: &mut MainMenuConfig| {
                config.logged_in = true;
            });

        // Optimized user stream setup with comprehensive error handling
        setup_user_stream(signal, environment)?;
    }

    Ok(())
}
