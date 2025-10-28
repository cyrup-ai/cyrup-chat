use crate::{
    error::OAuthError,
    future::WrappedFuture,
    pkce::PkceChallenge,
    server,
    types::{AccessType, OAuthResponse},
    Result,
};
use zeroize::Zeroizing;

pub struct AuthFlow {
    client_id: String,
    client_secret: Zeroizing<String>,
    redirect_uri: String,
    scopes: Vec<String>,
    state: Option<String>,
    access_type: AccessType,
    pkce_challenge: PkceChallenge,
}

impl AuthFlow {
    pub fn new(
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
        redirect_uri: impl Into<String>,
    ) -> Result<Self> {
        let pkce_challenge = PkceChallenge::new()?;
        Ok(Self {
            client_id: client_id.into(),
            client_secret: Zeroizing::new(client_secret.into()),
            redirect_uri: redirect_uri.into(),
            scopes: vec!["https://www.googleapis.com/auth/tasks".to_string()],
            state: None,
            access_type: AccessType::Online,
            pkce_challenge,
        })
    }

    pub fn with_scopes(mut self, scopes: Vec<String>) -> Self {
        self.scopes = scopes;
        self
    }

    pub fn with_state(mut self, state: impl Into<String>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn with_access_type(mut self, access_type: AccessType) -> Self {
        self.access_type = access_type;
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

    pub fn auth_url(&self) -> String {
        let scope = self.scopes.join(" ");
        let default_state;
        let state = match self.state.as_deref() {
            Some(s) => s,
            None => {
                default_state = uuid::Uuid::new_v4().to_string();
                &default_state
            }
        };

        let mut auth_url = format!(
            "https://accounts.google.com/o/oauth2/v2/auth?scope={}&access_type={}&include_granted_scopes=true&response_type=code&state={}&redirect_uri={}&client_id={}",
            urlencoding::encode(&scope),
            self.access_type.as_str(),
            urlencoding::encode(state),
            urlencoding::encode(&self.redirect_uri),
            urlencoding::encode(&self.client_id)
        );

        // Add PKCE parameters (always enabled for enhanced security)
        auth_url.push_str(&format!(
            "&code_challenge={}&code_challenge_method={}",
            urlencoding::encode(&self.pkce_challenge.code_challenge),
            self.pkce_challenge.challenge_method()
        ));

        auth_url
    }

    pub fn handle_callback(&self, callback_url: &str) -> WrappedFuture<Result<OAuthResponse>> {
        let code = match server::extract_callback_code(callback_url) {
            Ok(code) => code,
            Err(e) => return WrappedFuture::new(async move { Err(e) }),
        };

        let client_id = self.client_id.clone();
        let client_secret = self.client_secret.clone();
        let redirect_uri = self.redirect_uri.clone();
        let pkce_challenge = self.pkce_challenge.clone();

        WrappedFuture::new(async move {
            let mut params = vec![
                ("code", code),
                ("client_id", client_id),
                ("client_secret", client_secret.as_str().to_string()),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code".to_string()),
            ];

            // Add code_verifier (PKCE always enabled for enhanced security)
            params.push(("code_verifier", pkce_challenge.code_verifier.as_str().to_string()));

            let client = reqwest::Client::new();
            let response = client
                .post("https://www.googleapis.com/oauth2/v4/token")
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
