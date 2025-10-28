//! Main application logic with root reducer architecture

use super::auth_components::{AuthStatus, ErrorView, LoadingView, LoginView};
use super::main_components::MainView;
use crate::app::oauth::OAuthProvider;
use crate::app::reducer::state::AppStatus;
use crate::app::reducer::{AppAction, AppState, handle_app_action};
use dioxus::prelude::*;

#[component]
pub fn App() -> Element {
    // Root state - single source of truth
    let app_state = use_signal(AppState::default);

    // Create a mutable auth_status signal for legacy components that need it
    let mut auth_status_signal = use_signal(|| AuthStatus::Loading);

    // Sync auth_status_signal with app_state whenever app_state changes
    use_effect(move || {
        auth_status_signal.set(app_state.read().auth_status.clone());
    });

    // Dispatcher for child components
    let dispatch = use_callback(move |action: AppAction| {
        if let Err(e) = handle_app_action(app_state, action) {
            log::error!("Root action failed: {e}");
        }
    });

    // Initialize on mount
    use_effect(move || {
        dispatch(AppAction::InitializeEnvironment);
        dispatch(AppAction::CheckStoredAuth);
    });

    // Provide dispatcher via context
    use_context_provider(|| dispatch);

    let state = app_state.read();

    rsx! {
        Stylesheet { href: asset!("/assets/tailwind.css") }

        // Pure reactive rendering
        {match (&state.app_status, &state.auth_status) {
            (AppStatus::Initializing, _) | (_, AuthStatus::Loading) => {
                rsx! { LoadingView {} }
            }
            (AppStatus::EnvironmentError(error), _) => {
                rsx! { ErrorView { error: error.clone(), auth_status: auth_status_signal } }
            }
            (AppStatus::EnvironmentReady(env), AuthStatus::NotAuthenticated) => {
                use_context_provider(|| env.clone());
                rsx! {
                    OAuthProvider {
                        LoginView { auth_status: auth_status_signal }
                    }
                }
            }
            (AppStatus::EnvironmentReady(env), AuthStatus::Authenticated(auth_state)) => {
                use_context_provider(|| env.clone());
                rsx! {
                    MainView { auth_state: auth_state.clone() }
                }
            }
            (AppStatus::EnvironmentReady(_), AuthStatus::Error(error)) => {
                rsx! { ErrorView { error: error.clone(), auth_status: auth_status_signal } }
            }
        }}
    }
}
