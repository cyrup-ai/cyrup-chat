//! Root application actions

use crate::app::OAuthCallbacks;
use crate::auth::{AuthState, Provider};
use crate::environment::Environment;

#[derive(Clone, Debug)]
pub enum AppAction {
    // Environment lifecycle
    InitializeEnvironment,
    EnvironmentReady(Result<Environment, String>),

    // Authentication lifecycle
    CheckStoredAuth,
    AuthCheckComplete(Option<AuthState>),
    LoginRequested(Provider, OAuthCallbacks),
    LoginComplete(Result<AuthState, String>),
    LogoutRequested,
    LogoutComplete(Result<(), String>),

    // Error handling
    ClearError,
}
