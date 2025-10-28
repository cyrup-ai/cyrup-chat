use crate::{
    error::OAuthError, future::WrappedFuture, traits::OAuthRefresh, types::TokenResponse, Result,
};
use serde::Serialize;
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

            #[derive(serde::Serialize)]
            struct RefreshTokenRequest {
                client_id: String,
                #[serde(serialize_with = "serialize_zeroizing")]
                client_secret: Zeroizing<String>,
                #[serde(serialize_with = "serialize_zeroizing")]
                refresh_token: Zeroizing<String>,
                grant_type: String,
            }

            let _request = RefreshTokenRequest {
                client_id,
                client_secret,
                refresh_token: self.refresh_token,
                grant_type: "refresh_token".to_string(),
            };

            // Note: GitHub doesn't support refresh tokens in the traditional sense
            // This implementation is for compatibility, but GitHub tokens don't expire
            // and don't need refreshing. Apps should re-authorize if needed.
            return Err(OAuthError::Authorization(
                "GitHub does not support refresh tokens. Re-authorization required.".to_string(),
            ));
        })
    }
}

// Helper function for serializing Zeroizing<String>
fn serialize_zeroizing<S>(value: &Zeroizing<String>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.as_str().serialize(serializer)
}

// Trait implementations for generic OAuth usage
impl OAuthRefresh for RefreshExecuteBuilder {
    fn with_refresh_token(token: impl Into<String>) -> Self {
        RefreshExecuteBuilder {
            client_id: None,
            client_secret: None,
            from_env: true,
            refresh_token: Zeroizing::new(token.into()),
        }
    }

    fn execute(self) -> WrappedFuture<Result<crate::types::OAuthResponse>> {
        WrappedFuture::new(async move {
            // Convert TokenResponse to OAuthResponse
            let token_response = self.refresh().await?;
            Ok(crate::types::OAuthResponse {
                access_token: token_response.access_token,
                expires_in: Some(token_response.expires_in),
                scope: token_response.scope,
                token_type: token_response.token_type,
                refresh_token: None,
            })
        })
    }
}
