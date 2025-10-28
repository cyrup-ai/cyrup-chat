//! Authentication-related UI components
//!
//! This module contains UI components for authentication states including
//! loading, login, and error views.

use super::super::icons::{GitHubIcon, GoogleIcon};
use crate::app::reducer::AppAction;
use crate::auth::{AuthState, Provider, UserInfo};
use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
pub enum AuthStatus {
    Loading,
    NotAuthenticated,
    Authenticated(AuthState),
    Error(String),
}

#[component]
pub fn LoadingView() -> Element {
    rsx! {
        div {
            class: "flex items-center justify-center h-screen bg-gradient-to-br from-[#1a1a2e]/80 via-[#16213e]/80 to-[#0f0f1e]/80 bg-[length:400%_400%] animate-[gradientShift_45s_ease-in-out_infinite] relative",
            div {
                class: "backdrop-blur-xl bg-black/20 p-16 rounded-3xl border border-white/10 w-[400px] text-center shadow-2xl relative z-[2]",
                h1 {
                    class: "text-[3.5em] font-black bg-gradient-to-br from-[#00a8ff] via-[#0078ff] to-[#5856d6] bg-clip-text text-transparent tracking-tighter mb-4",
                    "CYRUP"
                }
                div {
                    class: "flex items-center justify-center gap-2 p-6",
                    div { class: "w-3 h-3 bg-[var(--g-accentColor)] rounded-full animate-pulse-dot" }
                    div { class: "w-3 h-3 bg-[var(--g-accentColor)] rounded-full animate-pulse-dot-delay-1" }
                    div { class: "w-3 h-3 bg-[var(--g-accentColor)] rounded-full animate-pulse-dot-delay-2" }
                }
                p {
                    class: "text-[var(--g-secondaryLabelColor)] italic mt-2",
                    "Initializing..."
                }
            }
        }
    }
}

#[component]
pub fn LoginView(auth_status: Signal<AuthStatus>) -> Element {
    let dispatch = use_context::<Callback<AppAction>>();

    rsx! {
        div {
            class: "flex items-center justify-center h-screen bg-gradient-to-br from-[#1a1a2e]/80 via-[#16213e]/80 to-[#0f0f1e]/80 bg-[length:400%_400%] animate-[gradientShift_45s_ease-in-out_infinite] relative",
            div {
                class: "backdrop-blur-xl bg-black/20 p-16 rounded-3xl border border-white/10 w-[400px] text-center shadow-2xl relative z-[2]",
                img {
                    src: asset!("/assets/img/cyrup_logo.png"),
                    alt: "CYRUP Logo",
                    class: "w-[200px] h-[200px] mx-auto mb-10"
                }

                div {
                    class: "flex flex-col gap-4 w-full mx-auto",
                    // Google Login Button (DEV MODE - Bypasses OAuth)
                    button {
                        class: "bg-white text-[#3c4043] border border-[#dadce0] hover:bg-[#f8f9fa] px-6 py-3 rounded-lg font-medium text-base cursor-pointer transition-all duration-200 flex items-center gap-4 justify-center w-full shadow-md hover:shadow-lg hover:-translate-y-px active:translate-y-0",
                        onclick: move |_| {
                            let auth_state = AuthState {
                                provider: Provider::Google,
                                user: UserInfo {
                                    email: "dev@cyrup.ai".to_string(),
                                    name: "Dev User".to_string(),
                                    picture: "".to_string(),
                                    id: "dev-google-user".to_string(),
                                    username: None,
                                },
                            };
                            dispatch(AppAction::LoginComplete(Ok(auth_state)));
                        },
                        GoogleIcon {}
                        span { "Sign in with Google" }
                    }

                    // GitHub Login Button (DEV MODE - Bypasses OAuth)
                    button {
                        class: "bg-[#24292f] text-white hover:bg-[#32383f] px-6 py-3 rounded-lg font-medium text-base cursor-pointer transition-all duration-200 flex items-center gap-4 justify-center w-full shadow-md hover:shadow-lg hover:-translate-y-px active:translate-y-0",
                        onclick: move |_| {
                            let auth_state = AuthState {
                                provider: Provider::GitHub,
                                user: UserInfo {
                                    email: "dev@cyrup.ai".to_string(),
                                    name: "Dev User".to_string(),
                                    picture: "".to_string(),
                                    id: "dev-github-user".to_string(),
                                    username: Some("devuser".to_string()),
                                },
                            };
                            dispatch(AppAction::LoginComplete(Ok(auth_state)));
                        },
                        GitHubIcon {}
                        span { "Sign in with GitHub" }
                    }
                }
            }
        }
    }
}

#[component]
pub fn ErrorView(error: String, auth_status: Signal<AuthStatus>) -> Element {
    let dispatch = use_context::<Callback<AppAction>>();

    rsx! {
        div {
            class: "flex items-center justify-center h-screen bg-gradient-to-br from-[#1a1a2e]/80 via-[#16213e]/80 to-[#0f0f1e]/80 bg-[length:400%_400%] animate-[gradientShift_45s_ease-in-out_infinite] relative",
            div {
                class: "backdrop-blur-xl bg-black/20 p-16 rounded-3xl border border-[#ff4444]/50 w-[400px] text-center shadow-2xl relative z-[2]",
                h1 {
                    class: "text-3xl font-bold text-white mb-4",
                    "Authentication Error"
                }
                p { class: "text-[#ff6666] mb-6", "{error}" }

                button {
                    class: "px-6 py-3 bg-[var(--g-accentColor)] text-white rounded-lg font-semibold cursor-pointer transition-all duration-200 mt-2 hover:bg-[var(--g-accentColorHighlight)] hover:-translate-y-px active:translate-y-0",
                    onclick: move |_| {
                        dispatch(AppAction::ClearError);
                    },
                    "Try Again"
                }
            }
        }
    }
}
