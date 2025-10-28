use crate::{
    error::OAuthError,
    future::WrappedFuture,
    types::{CallbackMode, OAuthResponse},
    Result,
};
use super::{builders::LoginConfigBuilder, callback::wait_for_callback, security::sanitize_api_error};
use zeroize::Zeroizing;

impl LoginConfigBuilder {
    /// Execute the OAuth login flow
    /// 
    /// This opens a browser, starts a local server to receive the callback,
    /// and exchanges the authorization code for tokens.
    /// 
    /// # Returns
    /// A future that resolves to `Result<OAuthResponse>` containing access tokens
    pub fn login(self) -> WrappedFuture<Result<OAuthResponse>> {
        WrappedFuture::new(async move {
            // Get credentials from environment or builder
            let (client_id, client_secret) = if self.from_env {
                let id = std::env::var("GOOGLE_CLIENT_ID")
                    .map_err(|_| OAuthError::EnvVar("GOOGLE_CLIENT_ID".to_string()))?;
                let secret = std::env::var("GOOGLE_CLIENT_SECRET")
                    .map_err(|_| OAuthError::EnvVar("GOOGLE_CLIENT_SECRET".to_string()))?;
                (id, Zeroizing::new(secret))
            } else {
                (
                    self.client_id.ok_or(OAuthError::MissingClientId)?,
                    self.client_secret.ok_or(OAuthError::MissingClientSecret)?,
                )
            };

            // Determine redirect URI
            let redirect_uri = if let Some(uri) = self.redirect_uri {
                uri
            } else {
                match &self.callback_mode {
                    CallbackMode::Server { port } => format!("http://localhost:{}/callback", port),
                }
            };

            // Build scope string with fallback
            let scope = if self.scopes.is_empty() {
                "https://www.googleapis.com/auth/tasks".to_string()
            } else {
                self.scopes.join(" ")
            };

            // Generate or use provided state for CSRF protection
            let state = self
                .state
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            // Build OAuth authorization URL with PKCE parameters
            log::debug!("OAuth redirect_uri: {}", &redirect_uri);
            let mut auth_url = format!(
                "https://accounts.google.com/o/oauth2/v2/auth?scope={}&access_type={}&include_granted_scopes=true&response_type=code&state={}&redirect_uri={}&client_id={}",
                urlencoding::encode(&scope),
                self.access_type.as_str(),
                urlencoding::encode(&state),
                urlencoding::encode(&redirect_uri),
                urlencoding::encode(&client_id)
            );

            // Add PKCE parameters (always enabled for enhanced security)
            auth_url.push_str(&format!(
                "&code_challenge={}&code_challenge_method={}",
                urlencoding::encode(&self.pkce_challenge.code_challenge),
                self.pkce_challenge.challenge_method()
            ));
            
            log::debug!("OAuth auth URL: {}", &auth_url);

            // Open browser to authorization URL
            if webbrowser::open(&auth_url).is_err() {
                log::warn!("Failed to open browser. Please visit this URL manually: {}", auth_url);
            }

            // Handle OAuth callback based on configured mode
            let code = match self.callback_mode {
                CallbackMode::Server { port } => {
                    // Start local HTTP server to receive OAuth callback
                    log::debug!("Attempting to bind to 127.0.0.1:{}", port);
                    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
                        .await
                        .map_err(|e| {
                            OAuthError::Server(format!("Failed to bind to port {}: {}", port, e))
                        })?;
                    log::info!("Successfully bound to 127.0.0.1:{}", port);

                    // Wait for callback with configured timeout
                    log::debug!("Starting wait_for_callback with timeout: {:?}", self.timeout);
                    let result = tokio::time::timeout(self.timeout, wait_for_callback(listener, &state))
                        .await
                        .map_err(|_| {
                            log::error!("OAuth callback timed out after {:?}", self.timeout);
                            OAuthError::Timeout
                        })?
                        .map_err(|e| {
                            log::error!("OAuth callback error: {}", e);
                            OAuthError::Server(e.to_string())
                        })?;
                    log::debug!("OAuth callback completed successfully");
                    result
                }
            };

            // Exchange authorization code for access token using PKCE
            let mut params = vec![
                ("code", code),
                ("client_id", client_id),
                ("client_secret", client_secret.as_str().to_string()),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code".to_string()),
            ];

            // Add PKCE code_verifier (always enabled for enhanced security)
            params.push(("code_verifier", self.pkce_challenge.code_verifier.as_str().to_string()));

            // Make token exchange request
            let client = reqwest::Client::new();
            let response = client
                .post("https://www.googleapis.com/oauth2/v4/token")
                .form(&params)
                .send()
                .await?;

            // Handle token exchange response
            if !response.status().is_success() {
                let status_code = response.status().as_u16();
                let error_text = response.text().await?;
                // Security: Sanitize error messages to prevent information disclosure
                let sanitized_error = sanitize_api_error(&error_text, status_code);
                return Err(OAuthError::TokenExchange(sanitized_error));
            }

            let oauth_response: OAuthResponse = response.json().await?;
            Ok(oauth_response)
        })
    }
}