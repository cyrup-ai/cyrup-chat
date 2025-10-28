use std::ops::Deref;

use anyhow::{Context, Result, anyhow};
use tokio::sync::OnceCell;
use zeroize::Zeroizing;

use crate::auth::CredentialVault;

/// Shared credential vault instance for embedded secret helpers.
static VAULT: OnceCell<CredentialVault> = OnceCell::const_new();

/// Secure string wrapper backed by zeroizing memory.
pub struct SecureString(Zeroizing<String>);

impl SecureString {
    pub fn new(value: String) -> Self {
        Self(Zeroizing::new(value))
    }

    pub fn expose(&self) -> &str {
        self.0.as_str()
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl Deref for SecureString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.0.as_str()
    }
}

async fn vault() -> Result<&'static CredentialVault> {
    VAULT
        .get_or_try_init(|| async { CredentialVault::new().await })
        .await
        .context("Failed to initialize credential vault")
}

async fn load_secret(provider: &str, secret_name: &str) -> Result<SecureString> {
    let vault = vault().await?;
    let secret = vault
        .get_application_secret(provider, secret_name)
        .await
        .with_context(|| {
            format!(
                "Failed to read {} secret for provider {}",
                secret_name, provider
            )
        })?
        .ok_or_else(|| anyhow!("Missing {} secret for provider {}", secret_name, provider))?;

    Ok(SecureString::new(secret))
}

/// Initialize the embedded vault by seeding application secrets from the environment.
pub async fn initialize_vault() -> Result<()> {
    let vault = vault().await?;
    vault
        .seed_application_secrets_from_env()
        .await
        .context("Failed to seed application secrets from environment")
}

/// Get Google OAuth client ID.
pub async fn get_google_oauth_client_id() -> Result<SecureString> {
    load_secret("google", "client_id").await
}

/// Get Google OAuth client secret.
pub async fn get_google_oauth_client_secret() -> Result<SecureString> {
    load_secret("google", "client_secret").await
}

/// Get GitHub OAuth client ID.
pub async fn get_github_oauth_client_id() -> Result<SecureString> {
    load_secret("github", "client_id").await
}

/// Get GitHub OAuth client secret.
pub async fn get_github_oauth_client_secret() -> Result<SecureString> {
    load_secret("github", "client_secret").await
}

/// Get GitHub private key (PEM).
pub async fn get_github_private_key() -> Result<SecureString> {
    load_secret("github", "private_key").await
}
