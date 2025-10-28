use crate::auth::embedded_vault::{get_google_oauth_client_id, get_google_oauth_client_secret};
use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Standard Google OAuth2 credential file format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleOAuthCredentials {
    pub installed: InstalledApp,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledApp {
    pub client_id: String,
    pub client_secret: String,
    pub auth_uri: String,
    pub token_uri: String,
    pub auth_provider_x509_cert_url: Option<String>,
    pub redirect_uris: Vec<String>,
}

impl GoogleOAuthCredentials {
    /// Load Google OAuth credentials from embedded vault
    pub async fn load() -> Result<Self> {
        log::debug!("Loading Google OAuth credentials from embedded vault");

        let client_id = get_google_oauth_client_id().await?;
        let client_secret = get_google_oauth_client_secret().await?;

        let installed = InstalledApp {
            client_id: client_id.as_str().to_string(),
            client_secret: client_secret.as_str().to_string(),
            auth_uri: "https://accounts.google.com/o/oauth2/auth".to_string(),
            token_uri: "https://oauth2.googleapis.com/token".to_string(),
            auth_provider_x509_cert_url: Some(
                "https://www.googleapis.com/oauth2/v1/certs".to_string(),
            ),
            redirect_uris: vec!["http://localhost".to_string()],
        };

        Ok(GoogleOAuthCredentials { installed })
    }

    /// Get client ID and secret
    pub fn client_credentials(&self) -> (&str, &str) {
        (&self.installed.client_id, &self.installed.client_secret)
    }
}
