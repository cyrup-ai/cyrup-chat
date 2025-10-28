//! Credential management and user data persistence

use super::types::{LoginAction, LoginSignal};
use crate::environment::{Environment, types::User};
use dioxus::prelude::*;
use std::cell::RefCell;

pub fn handle_credentials_actions(
    signal: LoginSignal,
    action: LoginAction,
    environment: &Environment,
) {
    // Other actions handled elsewhere
    if let LoginAction::SaveCredentials = action {
        let credentials = {
            let state = signal.read();
            match (
                &state.access_token,
                &state.account,
                &state.app_data,
                &state.model,
            ) {
                (Some(token), Some(account), Some(appdata), Some(model)) => Some((
                    token.clone(),
                    account.clone(),
                    appdata.clone(),
                    model.clone(),
                )),
                _ => None,
            }
        };

        if let Some((token, account, appdata, model)) = credentials {
            let user = User::new(model.url(), account.account, token, appdata);

            // Handle async repository operation with spawn
            spawn({
                let mut signal = signal;
                let environment = environment.clone();
                async move {
                    match environment.settings.update_or_insert_user(user).await {
                        Ok(_) => {
                            let mut state = signal.write();
                            state.done = true;
                            state.send_model = RefCell::new(state.model.clone());
                        }
                        Err(e) => {
                            signal.write().error_message = Some(format!("{e:?}"));
                        }
                    }
                }
            });
        }
    }
}
