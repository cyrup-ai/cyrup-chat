//! Central action dispatcher with zero-cost optimizations

use super::super::{auth, handlers, post, status_mutation};
use super::{
    errors::ActionError,
    types::{Action, ReducerState},
};
use crate::environment::Environment;
use dioxus::prelude::*;

/// Zero-cost action dispatch with compile-time optimizations
///
/// This function provides efficient routing to specialized handlers
/// with comprehensive error handling and performance optimizations.
#[inline(always)]
pub fn handle_action(
    signal: Signal<ReducerState>,
    action: Action,
    environment: &mut Environment,
) -> Result<(), ActionError> {
    log::trace!("Dispatching action: {action:?}");

    match action {
        // Authentication actions - fully implemented
        Action::Login => {
            auth::handle_login(signal, environment)?;
        }
        Action::LoggedIn(result) => {
            auth::handle_logged_in(signal, result, environment)?;
        }
        Action::Logout => {
            auth::handle_logout(signal, environment)?;
        }
        Action::LogoutDone(result) => {
            auth::handle_logout_done(signal, result, environment)?;
        }

        // Data and utility actions
        Action::DataUpdated => {
            handlers::utilities::handle_data_updated(signal, environment)?;
        }
        Action::ClearError => {
            handlers::utilities::handle_clear_error(signal, environment)?;
        }

        // Navigation actions - fully implemented
        Action::SelectAccount(account) => {
            handlers::navigation::handle_select_account(signal, account, environment)?;
        }
        Action::SelectNotifications(account) => {
            handlers::navigation::handle_select_notifications(signal, account, environment)?;
        }
        Action::SelectConversation(status_id) => {
            handlers::navigation::handle_select_conversation(signal, status_id, environment)?;
        }
        Action::SelectMore(selection) => {
            handlers::navigation::handle_select_more(signal, selection, environment)?;
        }

        // Post management actions
        Action::Post(kind) => {
            post::handle_post(signal, kind, environment)?;
        }
        Action::PostDone(status) => {
            post::handle_post_done(signal, status, environment)?;
        }
        Action::PostCancel => {
            post::handle_post_cancel(signal, environment)?;
        }

        // Settings actions
        Action::Preferences => {
            handlers::settings::handle_preferences(signal, environment)?;
        }
        Action::PreferencesChanged(change) => {
            handlers::settings::handle_preferences_changed(signal, change, environment)?;
        }

        // Event handling actions
        Action::AppEvent(event) => {
            handlers::events::handle_app_event(signal, event, environment)?;
        }
        Action::MessageEvent(message) => {
            handlers::events::handle_message_event(signal, message, environment)?;
        }

        // Public actions delegation
        Action::Public(_public_action) => {
            log::debug!("Public action handling not yet implemented");
        }

        // Status mutation result handling
        Action::StatusMutationResult(result, status_vm, mutation) => {
            status_mutation::handle_status_mutation_result(
                signal,
                result,
                status_vm,
                mutation,
                environment,
            )?;
        }

        // Batch mutation processing
        Action::BatchMutation(mutations) => {
            let batch_result =
                status_mutation::batch::handle_batch_mutations(signal, mutations, environment)?;
            if !batch_result.is_fully_successful() {
                log::warn!(
                    "Batch mutation completed with {} failures",
                    batch_result.failed_count()
                );
            }
        }
    }

    Ok(())
}
