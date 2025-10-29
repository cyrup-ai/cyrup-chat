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
        AppAction::MenuEvent(menu_event) => {
            use crate::environment::types::MainMenuEvent;
            
            // Menu events that affect root state
            log::debug!("Root received menu event: {:?}", menu_event);
            
            match menu_event {
                MainMenuEvent::Logout => {
                    // Trigger logout flow
                    auth::handle_logout_requested(signal)?;
                }
                MainMenuEvent::NewPost => {
                    // NewPost opens a window, handled by platform layer
                    log::debug!("NewPost menu event - handled by platform");
                }
                _ => {
                    // View navigation events (Timeline, Mentions, Messages, More) 
                    // are handled by MainView component via platform.handle_menu_events()
                    log::debug!("View navigation event - forwarded to MainView");
                }
            }
        }
    }

    Ok(())
}
