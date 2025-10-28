use crate::{error::OAuthError, future::WrappedFuture, types::TokenResponse, Result};
use zeroize::Zeroizing;

pub struct Refresh;

impl Refresh {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self
    }

    pub fn from_env() -> RefreshTokenBuilder {
        RefreshTokenBuilder {
            client_id: None,
            client_secret: None,
            from_env: true,
        }
    }

    pub fn client_id(id: impl Into<String>) -> RefreshClientSecretBuilder {
        RefreshClientSecretBuilder {
            client_id: id.into(),
        }
    }
}

pub struct RefreshClientSecretBuilder {
    client_id: String,
}

impl RefreshClientSecretBuilder {
    pub fn client_secret(self, secret: impl Into<String>) -> RefreshTokenBuilder {
        RefreshTokenBuilder {
            client_id: Some(self.client_id),
            client_secret: Some(Zeroizing::new(secret.into())),
            from_env: false,
        }
    }
}

pub struct RefreshTokenBuilder {
    client_id: Option<String>,
    client_secret: Option<Zeroizing<String>>,
    from_env: bool,
}

impl RefreshTokenBuilder {
    pub fn token(self, refresh_token: impl Into<String>) -> RefreshExecuteBuilder {
        RefreshExecuteBuilder {
            client_id: self.client_id,
            client_secret: self.client_secret,
            from_env: self.from_env,
            refresh_token: Zeroizing::new(refresh_token.into()),
        }
    }
}

pub struct RefreshExecuteBuilder {
    client_id: Option<String>,
    client_secret: Option<Zeroizing<String>>,
    from_env: bool,
    refresh_token: Zeroizing<String>,
}

impl RefreshExecuteBuilder {
    pub fn refresh(self) -> WrappedFuture<Result<TokenResponse>> {
        WrappedFuture::new(async move {
            // Get credentials
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

            #[derive(serde::Serialize)]
            struct RefreshTokenRequest {
                client_id: String,
                client_secret: String,
                refresh_token: String,
                grant_type: String,
            }

            let request = RefreshTokenRequest {
                client_id,
                client_secret: client_secret.as_str().to_string(),
                refresh_token: self.refresh_token.as_str().to_string(),
                grant_type: "refresh_token".to_string(),
            };

            let client = reqwest::Client::new();
            let response = client
                .post("https://www.googleapis.com/oauth2/v4/token")
                .json(&request)
                .send()
                .await?;

            if !response.status().is_success() {
                let status_code = response.status().as_u16();
                let error_text = response.text().await?;
                // Security: Sanitize error messages to prevent information disclosure
                let sanitized_error = sanitize_api_error(&error_text, status_code);
                return Err(OAuthError::TokenExchange(sanitized_error));
            }

            let token_response: TokenResponse = response.json().await?;
            Ok(token_response)
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
                            "Invalid refresh token".to_string()
                        }
                        desc if desc.contains("invalid_request") => {
                            "Invalid request parameters".to_string()
                        }
                        desc if desc.contains("unsupported_grant_type") => {
                            "Unsupported grant type".to_string()
                        }
                        _ => "Token refresh failed".to_string(),
                    };
                }
            }
            if let Some(error_type) = error_obj.as_str() {
                return match error_type {
                    "invalid_client" => "Invalid client credentials".to_string(),
                    "invalid_grant" => "Invalid refresh token".to_string(),
                    "invalid_request" => "Invalid request parameters".to_string(),
                    "unsupported_grant_type" => "Unsupported grant type".to_string(),
                    _ => "Token refresh failed".to_string(),
                };
            }
        }
    }

    // Fallback based on HTTP status code
    match status_code {
        400 => "Bad request - invalid refresh parameters".to_string(),
        401 => "Unauthorized - invalid client credentials".to_string(),
        403 => "Forbidden - client not authorized".to_string(),
        429 => "Rate limit exceeded - please try again later".to_string(),
        500..=599 => "OAuth service temporarily unavailable".to_string(),
        _ => "Token refresh failed".to_string(),
    }
}
