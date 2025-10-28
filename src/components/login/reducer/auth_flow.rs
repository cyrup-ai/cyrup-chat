//! Authentication flow handling - code entry and validation

use super::types::{LoginAction, LoginSignal};
use crate::environment::Environment;
use dioxus::prelude::*;

pub fn handle_auth_flow_actions(
    mut signal: LoginSignal,
    action: LoginAction,
    _environment: &Environment,
) {
    match action {
        LoginAction::EnteredCode(_code) => {
            let can_authenticate = {
                let state = signal.read();
                state.app_data.is_some() && state.model.is_some()
            };
            if can_authenticate {
                signal.write().is_loading = true;

                // Spawn authentication task using the verification code
                let code_clone = _code.clone();
                let mut signal_clone = signal;

                spawn(async move {
                    // Authentication will be handled through the model's OAuth flow
                    log::debug!("Starting authentication with verification code");

                    // The actual authentication is handled by the OAuth flow in the component
                    // This sets up the state for the authentication process
                    let mut state = signal_clone.write();
                    state.verification_code = Some(code_clone);
                    state.is_loading = false;
                });
            }
        }
        LoginAction::ValidatedCode(result) => {
            let mut state = signal.write();
            match result {
                Ok(n) => {
                    state.access_token = Some(n.clone());
                    // Q9: MVP hardcoded auth - OAuth token flow not needed
                    // Model creation happens in Environment initialization
                    log::info!("OAuth token received but using hardcoded auth for MVP");
                }
                Err(e) => state.error_message = Some(format!("{e:?}")),
            }
            state.is_loading = false;
        }
        LoginAction::RetrievedUser(result) => {
            let mut state = signal.write();
            state.is_loading = false;
            match *result {
                Ok(account) => {
                    state.account = Some(account);
                    // Directly call SaveCredentials action instead of Effect::action
                }
                Err(error) => state.error_message = Some(format!("{error:?}")),
            }
        }
        _ => {} // Other actions handled elsewhere
    }
}
