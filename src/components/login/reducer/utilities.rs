//! Utility actions for registration, follow, and cleanup

use super::types::{LoginAction, LoginSignal};
use crate::environment::Environment;
use dioxus::prelude::*;

pub fn handle_utility_actions(
    mut signal: LoginSignal,
    action: LoginAction,
    environment: &Environment,
) {
    match action {
        LoginAction::CloseLogin => {
            signal.write().close = true;
        }
        LoginAction::ActionRegister => {
            let instance_url = {
                let state = signal.read();
                state
                    .selected_instance
                    .as_ref()
                    .map(|instance| format!("https://{}", instance.name))
            };
            if let Some(url) = instance_url {
                environment.open_url(&url);
            }
        }
        LoginAction::ActionFollow => {
            let has_model = signal.read().model.is_some();
            if has_model {
                // Note: Follow action will be handled in component with use_future
            }
        }
        LoginAction::ActionFollowDone(_) => {
            signal.write().did_follow = true;
        }
        _ => {} // Other actions handled elsewhere
    }
}
