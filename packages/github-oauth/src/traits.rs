use crate::future::WrappedFuture;
use crate::{
    types::{OAuthResponse, UserInfo},
    Result,
};
use std::time::Duration;

/// Common trait for OAuth providers that can be used generically in applications
pub trait OAuthProvider {
    /// The name of the OAuth provider (e.g., "GitHub", "Google")
    fn provider_name() -> &'static str;

    /// Default scopes for this provider
    fn default_scopes() -> Vec<&'static str>;

    /// Authorization endpoint URL
    fn auth_endpoint() -> &'static str;

    /// Token exchange endpoint URL  
    fn token_endpoint() -> &'static str;

    /// User info endpoint URL
    fn user_info_endpoint() -> &'static str;
}

/// Common trait for OAuth login builders
pub trait OAuthLogin {
    type Builder;

    /// Create a new login builder from environment variables
    fn from_env() -> Self::Builder;

    /// Create a new login builder with explicit client ID
    fn client_id(id: impl Into<String>) -> Self::Builder;
}

/// Common trait for OAuth configuration builders
pub trait OAuthConfigBuilder {
    /// Add a scope to the OAuth request
    fn add_scope(self, scope: impl Into<String>) -> Self;

    /// Set scopes for the OAuth request
    fn scopes(self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Result<Self>
    where
        Self: Sized;

    /// Set the local port for the callback server
    fn port(self, port: u16) -> Self;

    /// Set custom redirect URI
    fn redirect_uri(self, uri: impl Into<String>) -> Self;

    /// Set custom state parameter
    fn state(self, state: impl Into<String>) -> Self;

    /// Set timeout for the OAuth flow
    fn timeout(self, timeout: Duration) -> Self;

    /// Execute the OAuth login flow
    fn login(self) -> WrappedFuture<Result<OAuthResponse>>;
}

/// Common trait for user info retrieval
pub trait OAuthUserInfo {
    /// Create a new user info builder with an access token
    fn with_token(token: impl Into<String>) -> Self;

    /// Execute the user info request
    fn execute(self) -> WrappedFuture<Result<UserInfo>>;
}

/// Common trait for token refresh
pub trait OAuthRefresh {
    /// Create a new refresh builder with a refresh token
    fn with_refresh_token(token: impl Into<String>) -> Self;

    /// Execute the token refresh
    fn execute(self) -> WrappedFuture<Result<OAuthResponse>>;
}
