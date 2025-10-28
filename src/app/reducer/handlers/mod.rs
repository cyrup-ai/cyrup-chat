//! Root action handlers

mod auth;
mod environment;

use super::actions::AppAction;
use super::state::AppState;
use dioxus::prelude::*;

/// Central dispatcher for root actions
pub fn handle_app_action(mut signal: Signal<AppState>, action: AppAction) -> Result<(), String> {
    log::debug!("Root action: {action:?}");

    match action {
        AppAction::InitializeEnvironment => {
            environment::handle_initialize(signal)?;
        }
        AppAction::EnvironmentReady(result) => {
            environment::handle_ready(signal, result)?;
        }
        AppAction::CheckStoredAuth => {
            auth::handle_check_stored(signal)?;
        }
        AppAction::AuthCheckComplete(auth_state) => {
            auth::handle_check_complete(signal, auth_state)?;
        }
        AppAction::LoginRequested(provider, oauth_callbacks) => {
            auth::handle_login_requested(signal, provider, oauth_callbacks)?;
        }
        AppAction::LoginComplete(result) => {
            auth::handle_login_complete(signal, result)?;
        }
        AppAction::LogoutRequested => {
            auth::handle_logout_requested(signal)?;
        }
        AppAction::LogoutComplete(result) => {
            auth::handle_logout_complete(signal, result)?;
        }
        AppAction::ClearError => {
            signal.with_mut(|state| {
                state.error = None;
                if matches!(state.auth_status, crate::app::views::AuthStatus::Error(_)) {
                    state.auth_status = crate::app::views::AuthStatus::NotAuthenticated;
                }
            });
        }
    }

    Ok(())
}
