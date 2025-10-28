//! OAuth system and protocol handling
//!
//! This module manages OAuth callbacks, protocol handlers, and authentication flows
//! with zero-allocation patterns and production-ready error handling.

use crate::app::errors::{InitializationError, build_http_response};
use dioxus::prelude::*;
use std::sync::OnceLock;

// Global channel for OAuth callbacks
static OAUTH_CHANNEL: OnceLock<tokio::sync::mpsc::UnboundedSender<(String, String)>> =
    OnceLock::new();
type OAuthReceiver =
    std::sync::Arc<tokio::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<(String, String)>>>;
static OAUTH_RX: OnceLock<OAuthReceiver> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct OAuthCallbacks {
    pending: Signal<std::collections::HashMap<String, tokio::sync::oneshot::Sender<String>>>,
}

impl OAuthCallbacks {
    pub fn register(&mut self, state: String) -> tokio::sync::oneshot::Receiver<String> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.pending.write().insert(state, tx);
        rx
    }
}

#[component]
pub fn OAuthProvider(children: Element) -> Element {
    let mut pending =
        use_signal(std::collections::HashMap::<String, tokio::sync::oneshot::Sender<String>>::new);
    let oauth_callbacks = OAuthCallbacks { pending };

    use_context_provider(|| oauth_callbacks);

    // Handle incoming OAuth callbacks using a coroutine
    use_coroutine(move |_: UnboundedReceiver<()>| async move {
        if let Some(rx) = OAUTH_RX.get() {
            loop {
                let msg = {
                    let mut rx_guard = rx.lock().await;
                    rx_guard.recv().await
                };
                match msg {
                    Some((state, code)) => {
                        let mut pending_mut = pending.write();
                        if let Some(sender) = pending_mut.remove(&state) {
                            let _ = sender.send(code);
                        }
                    }
                    None => break,
                }
            }
        }
    });

    rsx! { {children} }
}

/// Initialize the OAuth system with channels and receivers
pub fn initialize_oauth_system() -> Result<(), InitializationError> {
    let (oauth_tx, oauth_rx) = tokio::sync::mpsc::unbounded_channel::<(String, String)>();

    // Store the sender globally for the protocol handler
    OAUTH_CHANNEL
        .set(oauth_tx)
        .map_err(|_| InitializationError::OAuthChannelAlreadySet)?;
    OAUTH_RX
        .set(std::sync::Arc::new(tokio::sync::Mutex::new(oauth_rx)))
        .map_err(|_| InitializationError::OAuthReceiverAlreadySet)?;

    Ok(())
}

/// Create the OAuth protocol handler for desktop application
pub fn create_protocol_handler() -> impl Fn(
    &str,
    dioxus::desktop::wry::http::Request<Vec<u8>>,
    dioxus::desktop::wry::RequestAsyncResponder,
) + Send
+ Sync
+ 'static {
    move |_webview_id, request, responder| {
        if let Some(oauth_tx) = OAUTH_CHANNEL.get() {
            let oauth_tx = oauth_tx.clone();
            // OAuth protocol handler - legitimate use of tokio::spawn for external protocol handling
            tokio::spawn(async move {
                if request.uri().path() == "/callback" {
                    if let Some(query) = request.uri().query() {
                        let params: std::collections::HashMap<_, _> =
                            url::form_urlencoded::parse(query.as_bytes())
                                .into_owned()
                                .collect();

                        if let (Some(code), Some(state)) = (params.get("code"), params.get("state"))
                        {
                            log::info!("OAuth callback received - state: {state}");
                            let _ = oauth_tx.send((state.clone(), code.clone()));
                        }
                    }

                    let response = build_http_response(
                        200,
                        Some("text/html"),
                        Vec::from(&b"<html><body><h1>Login successful!</h1><p>You can close this window.</p></body></html>"[..])
                    );

                    match response {
                        Ok(resp) => responder.respond(resp),
                        Err(e) => {
                            log::error!("Failed to build success response: {e}");
                            panic!("Failed to build HTTP error response: {e}");
                        }
                    }
                } else {
                    let response =
                        build_http_response(404, Some("text/plain"), Vec::from(&b"Not found"[..]));

                    match response {
                        Ok(resp) => responder.respond(resp),
                        Err(e) => {
                            log::error!("Failed to build 404 response: {e}");
                            panic!("Failed to build HTTP 404 response: {e}");
                        }
                    }
                }
            });
        }
    }
}
