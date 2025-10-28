//! Authentication action handlers

use crate::app::OAuthCallbacks;
use crate::app::reducer::state::AppState;
use crate::app::views::AuthStatus; // ← REUSE existing type
use crate::auth::{self, AuthState, Provider};
use dioxus::prelude::*;

pub fn handle_check_stored(mut signal: Signal<AppState>) -> Result<(), String> {
    signal.with_mut(|state| {
        state.auth_status = AuthStatus::Loading;
    });

    spawn(async move {
        let auth_state = check_stored_auth().await;
        let _ = handle_check_complete(signal, auth_state);
    });

    Ok(())
}

async fn check_stored_auth() -> Option<AuthState> {
    // Note: Auth check now handled by auth::Vault - stored auth moved to secure keyring
    // User metadata is in Settings, but tokens are in Vault
    // For now, return None to require fresh login
    None
}

pub fn handle_check_complete(
    mut signal: Signal<AppState>,
    auth_state: Option<AuthState>,
) -> Result<(), String> {
    signal.with_mut(|state| {
        state.auth_status = match auth_state {
            Some(auth) => AuthStatus::Authenticated(auth),
            None => AuthStatus::NotAuthenticated,
        };
    });
    Ok(())
}

pub fn handle_login_requested(
    mut signal: Signal<AppState>,
    provider: Provider,
    oauth_callbacks: OAuthCallbacks,
) -> Result<(), String> {
    signal.with_mut(|state| {
        state.auth_status = AuthStatus::Loading;
    });

    spawn(async move {
        let result = crate::auth::login_with_provider_native(provider, oauth_callbacks)
            .await
            .map_err(|e| e.to_string());
        let _ = handle_login_complete(signal, result);
    });

    Ok(())
}

pub fn handle_login_complete(
    mut signal: Signal<AppState>,
    result: Result<AuthState, String>,
) -> Result<(), String> {
    signal.with_mut(|state| match result {
        Ok(auth_state) => {
            state.auth_status = AuthStatus::Authenticated(auth_state);
            state.error = None;
        }
        Err(error) => {
            state.auth_status = AuthStatus::Error(format!("Login failed: {error}"));
        }
    });
    Ok(())
}

pub fn handle_logout_requested(mut signal: Signal<AppState>) -> Result<(), String> {
    signal.with_mut(|state| {
        state.auth_status = AuthStatus::Loading;
    });

    spawn(async move {
        // REUSE existing logout function
        let result = auth::logout().await.map_err(|e| e.to_string());
        let _ = handle_logout_complete(signal, result);
    });

    Ok(())
}

pub fn handle_logout_complete(
    mut signal: Signal<AppState>,
    result: Result<(), String>,
) -> Result<(), String> {
    signal.with_mut(|state| match result {
        Ok(()) => {
            state.auth_status = AuthStatus::NotAuthenticated;
            state.error = None;
        }
        Err(error) => {
            state.error = Some(format!("Logout failed: {error}"));
        }
    });
    Ok(())
}
