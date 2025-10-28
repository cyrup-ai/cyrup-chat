//! OAuth authentication handling

use super::types::{LoginAction, LoginSignal};
use crate::environment::Environment;
use dioxus::prelude::*;

pub fn handle_oauth_actions(
    mut signal: LoginSignal,
    action: LoginAction,
    _environment: &Environment,
) {
    match action {
        LoginAction::SelectProvider(provider) => {
            let mut state = signal.write();
            state.selected_provider = Some(provider);
            state.error_message = None;
        }
        LoginAction::StartOAuth(_provider) => {
            let mut state = signal.write();
            state.is_loading = true;
            state.error_message = None;
            // Note: OAuth login will be handled in the component with use_future
        }
        LoginAction::OAuthCompleted(result) => {
            let mut state = signal.write();
            state.is_loading = false;
            match result {
                Ok(auth_state) => {
                    state.auth_state = Some(auth_state);
                    state.done = true;
                    state.error_message = None;
                }
                Err(error) => {
                    state.error_message = Some(error);
                    state.auth_state = None;
                }
            }
        }
        _ => {} // Other actions handled elsewhere
    }
}
