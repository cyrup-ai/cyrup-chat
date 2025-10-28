//! Error types for application initialization and HTTP responses
//!
//! This module defines error types used throughout the application
//! for initialization failures and HTTP response building.

use dioxus::desktop::wry;

/// Error type for application initialization failures
#[derive(Debug)]
pub enum InitializationError {
    /// OAuth channel was already initialized
    OAuthChannelAlreadySet,
    /// OAuth receiver was already initialized  
    OAuthReceiverAlreadySet,
    /// Failed to initialize OAuth callback system
    OAuthInitializationFailed,
}

impl std::fmt::Display for InitializationError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OAuthChannelAlreadySet => write!(f, "OAuth channel was already initialized"),
            Self::OAuthReceiverAlreadySet => write!(f, "OAuth receiver was already initialized"),
            Self::OAuthInitializationFailed => {
                write!(f, "Failed to initialize OAuth callback system")
            }
        }
    }
}

impl std::error::Error for InitializationError {}

/// Error type for HTTP response building failures
#[derive(Debug)]
pub enum HttpResponseError {
    /// Failed to build HTTP response
    ResponseBuildFailed(String),
}

impl std::fmt::Display for HttpResponseError {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ResponseBuildFailed(msg) => write!(f, "Failed to build HTTP response: {msg}"),
        }
    }
}

impl std::error::Error for HttpResponseError {}

/// Build a safe HTTP response with proper error handling
#[inline]
pub fn build_http_response(
    status: u16,
    content_type: Option<&str>,
    body: Vec<u8>,
) -> Result<wry::http::Response<Vec<u8>>, HttpResponseError> {
    let mut builder = wry::http::Response::builder().status(status);

    if let Some(ct) = content_type {
        builder = builder.header("Content-Type", ct);
    }

    builder
        .body(body)
        .map_err(|e| HttpResponseError::ResponseBuildFailed(e.to_string()))
}
