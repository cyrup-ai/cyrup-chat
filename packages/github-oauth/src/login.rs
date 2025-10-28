use crate::{
    error::OAuthError,
    future::WrappedFuture,
    pkce::PkceChallenge,
    traits::{OAuthConfigBuilder, OAuthLogin},
    types::{AccessType, CallbackMode, OAuthResponse},
    Result,
};
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use zeroize::Zeroizing;

pub struct Login;

impl Login {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self
    }

    pub fn from_env() -> LoginScopesBuilder {
        LoginScopesBuilder {
            client_id: None,
            client_secret: None,
            from_env: true,
        }
    }

    pub fn client_id(id: impl Into<String>) -> LoginClientSecretBuilder {
        LoginClientSecretBuilder {
            client_id: id.into(),
        }
    }
}

pub struct LoginClientSecretBuilder {
    client_id: String,
}

impl LoginClientSecretBuilder {
    pub fn client_secret(self, secret: impl Into<String>) -> LoginScopesBuilder {
        LoginScopesBuilder {
            client_id: Some(self.client_id),
            client_secret: Some(Zeroizing::new(secret.into())),
            from_env: false,
        }
    }
}

pub struct LoginScopesBuilder {
    client_id: Option<String>,
    client_secret: Option<Zeroizing<String>>,
    from_env: bool,
}

impl LoginScopesBuilder {
    pub fn client_secret(self, secret: impl Into<String>) -> Self {
        Self {
            client_id: self.client_id,
            client_secret: Some(Zeroizing::new(secret.into())),
            from_env: self.from_env,
        }
    }

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

pub struct LoginConfigBuilder {
    client_id: Option<String>,
    client_secret: Option<Zeroizing<String>>,
    from_env: bool,
    scopes: Vec<String>,
    callback_mode: CallbackMode,
    redirect_uri: Option<String>,
    state: Option<String>,
    access_type: AccessType,
    timeout: Duration,
    pkce_challenge: PkceChallenge,
}

impl LoginConfigBuilder {
    pub fn add_scope(mut self, scope: impl Into<String>) -> Self {
        self.scopes.push(scope.into());
        self
    }

    pub fn port(mut self, port: u16) -> Self {
        self.callback_mode = CallbackMode::server(port);
        self
    }

    pub fn callback_mode(mut self, mode: CallbackMode) -> Self {
        self.callback_mode = mode;
        self
    }

    pub fn redirect_uri(mut self, uri: impl Into<String>) -> Self {
        self.redirect_uri = Some(uri.into());
        self
    }

    pub fn state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn access_type(mut self, access_type: AccessType) -> Self {
        self.access_type = access_type;
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }


    /// Use a custom PKCE challenge/verifier pair
    /// 
    /// This allows you to provide your own PKCE challenge, which can be useful
    /// for testing or when you need to manage the challenge lifecycle yourself.
    pub fn with_pkce_challenge(mut self, challenge: PkceChallenge) -> Self {
        self.pkce_challenge = challenge;
        self
    }

    pub fn login(self) -> WrappedFuture<Result<OAuthResponse>> {
        WrappedFuture::new(async move {
            // Get credentials
            let (client_id, client_secret) = if self.from_env {
                let id = std::env::var("GITHUB_CLIENT_ID")
                    .map_err(|_| OAuthError::EnvVar("GITHUB_CLIENT_ID".to_string()))?;
                let secret = std::env::var("GITHUB_CLIENT_SECRET")
                    .map_err(|_| OAuthError::EnvVar("GITHUB_CLIENT_SECRET".to_string()))?;
                (id, Zeroizing::new(secret))
            } else {
                (
                    self.client_id.ok_or(OAuthError::MissingClientId)?,
                    self.client_secret.ok_or(OAuthError::MissingClientSecret)?,
                )
            };

            let redirect_uri = if let Some(uri) = self.redirect_uri {
                uri
            } else {
                match &self.callback_mode {
                    CallbackMode::Server { port } => format!("http://localhost:{}/callback", port),
                }
            };

            let scope = if self.scopes.is_empty() {
                "user:email".to_string()
            } else {
                self.scopes.join(" ")
            };

            let state = self
                .state
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

            // Build auth URL with optional PKCE parameters
            let mut auth_url = format!(
                "https://github.com/login/oauth/authorize?scope={}&response_type=code&state={}&redirect_uri={}&client_id={}",
                urlencoding::encode(&scope),
                urlencoding::encode(&state),
                urlencoding::encode(&redirect_uri),
                urlencoding::encode(&client_id)
            );

            // Add PKCE parameters (always enabled in 2025)
            auth_url.push_str(&format!(
                "&code_challenge={}&code_challenge_method={}",
                urlencoding::encode(&self.pkce_challenge.code_challenge),
                self.pkce_challenge.challenge_method()
            ));

            // Open browser
            if webbrowser::open(&auth_url).is_err() {
                eprintln!(
                    "Failed to open browser. Please visit this URL manually:\n{}",
                    auth_url
                );
            }

            // Handle callback based on mode
            let code = match self.callback_mode {
                CallbackMode::Server { port } => {
                    // Start local server to receive callback
                    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
                        .await
                        .map_err(|e| {
                            OAuthError::Server(format!("Failed to bind to port {}: {}", port, e))
                        })?;

                    // Wait for callback with timeout
                    tokio::time::timeout(self.timeout, wait_for_callback(listener, &state))
                        .await
                        .map_err(|_| OAuthError::Timeout)?
                        .map_err(|e| OAuthError::Server(e.to_string()))?
                }
            };

            // Exchange code for token
            let mut params = vec![
                ("code", code.as_str().to_string()),
                ("client_id", client_id),
                ("client_secret", client_secret.as_str().to_string()),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code".to_string()),
            ];

            // Add code_verifier (PKCE always enabled in 2025)
            params.push(("code_verifier", self.pkce_challenge.code_verifier.as_str().to_string()));

            let client = reqwest::Client::new();
            let response = client
                .post("https://github.com/login/oauth/access_token")
                .header("Accept", "application/json")
                .form(&params)
                .send()
                .await?;

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

async fn wait_for_callback(
    listener: tokio::net::TcpListener,
    expected_state: &str,
) -> Result<Zeroizing<String>> {
    let (mut stream, _) = listener.accept().await?;

    // Security: Limit request size to prevent DoS attacks
    const MAX_REQUEST_SIZE: usize = 8192; // 8KB should be sufficient for OAuth callbacks
    let mut buffer = vec![0; MAX_REQUEST_SIZE];

    let n = match stream.read(&mut buffer).await {
        Ok(n) if n == MAX_REQUEST_SIZE => {
            // If we read exactly MAX_REQUEST_SIZE, the request might be larger
            return Err(OAuthError::Server("Request too large".to_string()));
        }
        Ok(n) => n,
        Err(e) => return Err(OAuthError::Io(e)),
    };

    let request = String::from_utf8_lossy(&buffer[..n]);

    // Extract the request line
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| OAuthError::Server("Invalid request".to_string()))?;

    // Parse the callback URL from GET request
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| OAuthError::Server("Invalid request path".to_string()))?;

    let callback_url = format!("http://localhost{}", path);
    let url = url::Url::parse(&callback_url)?;

    // Extract code and state
    let mut code = None;
    let mut state = None;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => {
                let error_desc = url
                    .query_pairs()
                    .find(|(k, _)| k == "error_description")
                    .map(|(_, v)| v.into_owned())
                    .unwrap_or_else(|| value.into_owned());

                // Send error response
                let html = crate::template::default_success_template(
                    &crate::template::TemplateContext::error(error_desc.clone()),
                )
                .into_string();
                let response = create_secure_http_response(&html);
                stream.write_all(response.as_bytes()).await?;
                stream.shutdown().await?;

                return Err(OAuthError::Authorization(error_desc));
            }
            _ => {}
        }
    }

    let code =
        code.ok_or_else(|| OAuthError::Authorization("No authorization code found".to_string()))?;
    let state =
        state.ok_or_else(|| OAuthError::Authorization("No state parameter found".to_string()))?;

    // Verify state matches expected
    if state != expected_state {
        // Send error response
        let html =
            crate::template::default_success_template(&crate::template::TemplateContext::error(
                "Invalid state parameter - possible CSRF attack",
            ))
            .into_string();
        let response = create_secure_http_response(&html);
        stream.write_all(response.as_bytes()).await?;
        stream.shutdown().await?;

        return Err(OAuthError::InvalidState);
    }

    // Send success response
    let html =
        crate::template::default_success_template(&crate::template::TemplateContext::success())
            .into_string();
    let response = create_secure_http_response(&html);
    stream.write_all(response.as_bytes()).await?;
    stream.shutdown().await?;

    Ok(Zeroizing::new(code))
}

/// Security: Sanitize API error messages to prevent information disclosure
fn sanitize_api_error(error_text: &str, status_code: u16) -> String {
    // Parse JSON error if possible to extract safe information
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(error_text) {
        if let Some(error_obj) = json.get("error") {
            if let Some(error_desc) = error_obj.get("error_description") {
                if let Some(desc) = error_desc.as_str() {
                    return match desc {
                        desc if desc.contains("invalid_client") => {
                            "Invalid client credentials".to_string()
                        }
                        desc if desc.contains("invalid_grant") => {
                            "Invalid authorization code".to_string()
                        }
                        desc if desc.contains("invalid_request") => {
                            "Invalid request parameters".to_string()
                        }
                        desc if desc.contains("unsupported_grant_type") => {
                            "Unsupported grant type".to_string()
                        }
                        _ => "OAuth authentication failed".to_string(),
                    };
                }
            }
            if let Some(error_type) = error_obj.as_str() {
                return match error_type {
                    "invalid_client" => "Invalid client credentials".to_string(),
                    "invalid_grant" => "Invalid authorization code".to_string(),
                    "invalid_request" => "Invalid request parameters".to_string(),
                    "unsupported_grant_type" => "Unsupported grant type".to_string(),
                    _ => "OAuth authentication failed".to_string(),
                };
            }
        }
    }

    // Fallback based on HTTP status code
    match status_code {
        400 => "Bad request - invalid OAuth parameters".to_string(),
        401 => "Unauthorized - invalid client credentials".to_string(),
        403 => "Forbidden - client not authorized".to_string(),
        429 => "Rate limit exceeded - please try again later".to_string(),
        500..=599 => "OAuth service temporarily unavailable".to_string(),
        _ => "OAuth authentication failed".to_string(),
    }
}

/// Security: Generate HTTP response with proper security headers
fn create_secure_http_response(html: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         X-Frame-Options: DENY\r\n\
         X-Content-Type-Options: nosniff\r\n\
         X-XSS-Protection: 1; mode=block\r\n\
         Content-Security-Policy: default-src 'self'; script-src 'unsafe-inline'; style-src 'unsafe-inline'\r\n\
         Referrer-Policy: strict-origin-when-cross-origin\r\n\
         Cache-Control: no-cache, no-store, must-revalidate\r\n\
         Pragma: no-cache\r\n\
         Expires: 0\r\n\
         \r\n{}",
        html
    )
}

// Trait implementations for generic OAuth usage
impl OAuthLogin for Login {
    type Builder = LoginScopesBuilder;

    fn from_env() -> Self::Builder {
        Login::from_env()
    }

    fn client_id(id: impl Into<String>) -> Self::Builder {
        // Create a LoginScopesBuilder directly since we need the trait to return the same type
        LoginScopesBuilder {
            client_id: Some(id.into()),
            client_secret: None,
            from_env: false,
        }
    }
}

impl OAuthConfigBuilder for LoginConfigBuilder {
    fn add_scope(self, scope: impl Into<String>) -> Self {
        self.add_scope(scope)
    }

    fn scopes(self, scopes: impl IntoIterator<Item = impl Into<String>>) -> Result<Self> {
        LoginScopesBuilder::scopes(
            LoginScopesBuilder {
                client_id: self.client_id,
                client_secret: self.client_secret,
                from_env: self.from_env,
            },
            scopes,
        )
    }

    fn port(self, port: u16) -> Self {
        self.port(port)
    }

    fn redirect_uri(self, uri: impl Into<String>) -> Self {
        self.redirect_uri(uri)
    }

    fn state(self, state: impl Into<String>) -> Self {
        self.state(state)
    }

    fn timeout(self, timeout: Duration) -> Self {
        self.timeout(timeout)
    }

    fn login(self) -> WrappedFuture<Result<OAuthResponse>> {
        self.login()
    }
}
