//! Login reducer module - organized authentication and state management

pub mod auth_flow;
pub mod credentials;
pub mod model;
pub mod oauth;
pub mod types;
pub mod utilities;

// Re-export main API
pub use types::{LoginAction, LoginSignal, LoginState, Selection};

use crate::environment::Environment;

// Modern Dioxus signal-based action handler
#[allow(dead_code)] // Login action handler - pending integration
pub fn handle_login_action(signal: LoginSignal, action: LoginAction, environment: &Environment) {
    log::trace!("{action:?}");

    // Route actions to appropriate handlers
    match action {
        // OAuth actions
        LoginAction::SelectProvider(_)
        | LoginAction::StartOAuth(_)
        | LoginAction::OAuthCompleted(_) => {
            oauth::handle_oauth_actions(signal, action, environment);
        }

        // Instance management actions
        LoginAction::Load
        | LoginAction::LoadedInstances(_)
        | LoginAction::SelectInstance(_)
        | LoginAction::ChosenInstance
        | LoginAction::RetrieveUrl(_, _) => {
            // Q9: MVP hardcoded auth - instance management not needed
            log::debug!("Instance action {:?} ignored in MVP", action);
        }

        // Authentication flow actions
        LoginAction::EnteredCode(_)
        | LoginAction::ValidatedCode(_)
        | LoginAction::RetrievedUser(_) => {
            let is_retrieved_user =
                matches!(action, LoginAction::RetrievedUser(ref b) if b.as_ref().is_ok());
            auth_flow::handle_auth_flow_actions(signal, action, environment);

            // Handle SaveCredentials immediately if we have an account after RetrievedUser
            if is_retrieved_user {
                let should_save = signal().account.is_some();
                if should_save {
                    credentials::handle_credentials_actions(
                        signal,
                        LoginAction::SaveCredentials,
                        environment,
                    );
                }
            }
        }

        // Credential management actions
        LoginAction::SaveCredentials => {
            credentials::handle_credentials_actions(signal, action, environment);
        }

        // Utility actions
        LoginAction::CloseLogin
        | LoginAction::ActionRegister
        | LoginAction::ActionFollow
        | LoginAction::ActionFollowDone(_) => {
            utilities::handle_utility_actions(signal, action, environment);
        }
    }
}
