use crate::{
    pkce::PkceChallenge,
    types::{AccessType, CallbackMode},
    Result,
};
use std::time::Duration;
use zeroize::Zeroizing;

/// Main entry point for OAuth login flow
pub struct Login;

impl Login {
    /// Create a new Login instance (hidden from docs)
    #[doc(hidden)]
    pub fn new() -> Self {
        Self
    }

    /// Create login flow using environment variables for credentials
    /// 
    /// Expects GOOGLE_CLIENT_ID and GOOGLE_CLIENT_SECRET environment variables.
    pub fn from_env() -> LoginScopesBuilder {
        LoginScopesBuilder {
            client_id: None,
            client_secret: None,
            from_env: true,
        }
    }

    /// Create login flow with explicit client ID
    /// 
    /// This requires calling `.client_secret()` next to provide the secret.
    pub fn client_id(id: impl Into<String>) -> LoginClientSecretBuilder {
        LoginClientSecretBuilder {
            client_id: id.into(),
        }
    }
}

/// Builder for providing client secret after client ID
pub struct LoginClientSecretBuilder {
    client_id: String,
}

impl LoginClientSecretBuilder {
    /// Provide the client secret
    /// 
    /// This transitions to the scopes builder where you must specify OAuth scopes.
    pub fn client_secret(self, secret: impl Into<String>) -> LoginScopesBuilder {
        LoginScopesBuilder {
            client_id: Some(self.client_id),
            client_secret: Some(Zeroizing::new(secret.into())),
            from_env: false,
        }
    }
}

/// Builder for configuring OAuth scopes
/// 
/// You must call either `.scopes()` or `.add_scope()` to proceed to configuration.
pub struct LoginScopesBuilder {
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<Zeroizing<String>>,
    pub(crate) from_env: bool,
}

impl LoginScopesBuilder {
    /// Set multiple OAuth scopes at once
    /// 
    /// # Arguments
    /// * `scopes` - An iterator of scope strings or convertible types
    /// 
    /// # Returns
    /// A `Result<LoginConfigBuilder>` for further configuration or execution
    pub fn scopes(self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Result<LoginConfigBuilder> {
        let pkce_challenge = PkceChallenge::new()?;
        Ok(LoginConfigBuilder {
            client_id: self.client_id,
            client_secret: self.client_secret,
            from_env: self.from_env,
            scopes: scopes.into_iter().map(Into::into).collect(),
            callback_mode: CallbackMode::server(8080),
            redirect_uri: None,
            state: None,
            access_type: AccessType::Online,
            timeout: Duration::from_secs(300),
            pkce_challenge,
        })
    }

    /// Add a single OAuth scope
    /// 
    /// # Arguments
    /// * `scope` - The OAuth scope string to add
    /// 
    /// # Returns
    /// A `Result<LoginConfigBuilder>` for further configuration or execution
    pub fn add_scope(self, scope: impl Into<String>) -> Result<LoginConfigBuilder> {
        let pkce_challenge = PkceChallenge::new()?;
        Ok(LoginConfigBuilder {
            client_id: self.client_id,
            client_secret: self.client_secret,
            from_env: self.from_env,
            scopes: vec![scope.into()],
            callback_mode: CallbackMode::server(8080),
            redirect_uri: None,
            state: None,
            access_type: AccessType::Online,
            timeout: Duration::from_secs(300),
            pkce_challenge,
        })
    }
}

/// Main configuration builder for OAuth login flow
/// 
/// Configure various OAuth parameters before calling `.login()` to execute.
pub struct LoginConfigBuilder {
    pub(crate) client_id: Option<String>,
    pub(crate) client_secret: Option<Zeroizing<String>>,
    pub(crate) from_env: bool,
    pub(crate) scopes: Vec<String>,
    pub(crate) callback_mode: CallbackMode,
    pub(crate) redirect_uri: Option<String>,
    pub(crate) state: Option<String>,
    pub(crate) access_type: AccessType,
    pub(crate) timeout: Duration,
    pub(crate) pkce_challenge: PkceChallenge,
}

impl LoginConfigBuilder {
    /// Add an additional scope to the existing scopes
    #[inline]
    pub fn add_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    /// Set the local port for the OAuth callback server
    /// 
    /// # Arguments
    /// * `port` - Port number for the local HTTP server (typically 8080)
    #[inline]
    pub fn port(mut self, port: u16) -> Self {
        self.callback_mode = CallbackMode::server(port);
        self
    }

    /// Set the callback mode for OAuth redirect handling
    /// 
    /// # Arguments
    /// * `mode` - The callback mode (server or custom protocol)
    #[inline]
    pub fn callback_mode(mut self, mode: CallbackMode) -> Self {
        self.callback_mode = mode;
        self
    }

    /// Set a custom redirect URI (optional)
    /// 
    /// # Arguments
    /// * `uri` - The redirect URI to use instead of the default localhost
    #[inline]
    pub fn redirect_uri(mut self, uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(uri.into());
        self
    }

    /// Set the CSRF state parameter for security
    /// 
    /// # Arguments
    /// * `state` - Random string to prevent CSRF attacks
    #[inline]
    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    /// Set the OAuth access type (online vs offline)
    /// 
    /// # Arguments
    /// * `access_type` - AccessType::Online (default) or AccessType::Offline for refresh tokens
    #[inline]
    pub fn access_type(mut self, access_type: AccessType) -> Self {
        self.access_type = access_type;
        self
    }

    /// Set the timeout for the OAuth flow
    /// 
    /// # Arguments
    /// * `timeout` - Maximum duration to wait for user authorization
    #[inline]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Use a custom PKCE challenge/verifier pair
    /// 
    /// This allows you to provide your own PKCE challenge, which can be useful
    /// for testing or when you need to manage the challenge lifecycle yourself.
    /// 
    /// # Arguments
    /// * `challenge` - Pre-generated PKCE challenge/verifier pair
    #[inline]
    pub fn with_pkce_challenge(mut self, challenge: PkceChallenge) -> Self {
        self.pkce_challenge = challenge;
        self
    }
}