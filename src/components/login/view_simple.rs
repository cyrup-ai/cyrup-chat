use crate::auth::{Provider, login_with_provider};
use crate::environment::Environment;
use dioxus::prelude::*;

#[component]
pub fn LoginApp(environment: Signal<Environment>, should_show_login: Signal<bool>) -> Element {
    let mut is_loading = use_signal(|| false);
    let mut error_message = use_signal(|| None::<String>);

    let mut handle_oauth_login = move |provider: Provider| {
        // Clear previous errors
        error_message.set(None);
        is_loading.set(true);

        // Start real OAuth authentication
        use_future(move || async move {
            match login_with_provider(provider).await {
                Ok(_auth_state) => {
                    log::info!("OAuth login successful for provider: {:?}", provider);

                    // Access token is now stored in vault, not in auth_state
                    // Use auth_state.get_valid_token() if needed

                    // OAuth providers (Google/GitHub) don't specify Mastodon instance
                    // Return error indicating instance URL selection is required
                    error_message.set(Some(
                        "OAuth login successful, but Mastodon instance selection not implemented. \
                        Please use direct Mastodon OAuth instead."
                            .to_string(),
                    ));
                }
                Err(e) => {
                    log::error!("OAuth login failed: {}", e);
                    error_message.set(Some(format!("Login failed: {}", e)));
                }
            }
            is_loading.set(false);
        });
    };

    rsx! {
        div {
            class: "login-container",
            div {
                class: "login-box",
                h1 { "Welcome to CYRUP Chat" }
                p { "Sign in to continue" }

                if let Some(error) = error_message.read().as_ref() {
                    div {
                        class: "error-message",
                        "{error}"
                    }
                }

                div {
                    class: "oauth-buttons",

                    button {
                        class: "oauth-button google-login-button",
                        disabled: *is_loading.read(),
                        onclick: move |_| { handle_oauth_login(Provider::Google); },
                        if *is_loading.read() {
                            "Signing In..."
                        } else {
                            "Sign in with Google"
                        }
                    }

                    button {
                        class: "oauth-button github-login-button",
                        disabled: *is_loading.read(),
                        onclick: move |_| { handle_oauth_login(Provider::GitHub); },
                        if *is_loading.read() {
                            "Signing In..."
                        } else {
                            "Sign in with GitHub"
                        }
                    }
                }
            }
        }
    }
}
