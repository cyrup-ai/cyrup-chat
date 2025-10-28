use anyhow::{Context, Result};
use chrono::{Duration, Utc};
use keyring::Entry;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Simple keyring-based credential vault
///
/// This provides a secure credential storage system using the OS keyring.
pub struct CredentialVault {
    service_name: String,
    http_client: Client,
    provider_registry_key: String,
}

/// Types of credentials that can be stored in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CredentialType {
    /// OAuth access token
    AccessToken,
    /// OAuth refresh token
    RefreshToken,
    /// User information
    UserInfo,
    /// Client credentials (ID and secret)
    ClientCredentials,
    /// Application secret
    ApplicationSecret,
}

impl CredentialType {
    /// Get the key suffix for this credential type
    pub fn key_suffix(&self) -> &'static str {
        match self {
            CredentialType::AccessToken => "access_token",
            CredentialType::RefreshToken => "refresh_token",
            CredentialType::UserInfo => "user_info",
            CredentialType::ClientCredentials => "client_credentials",
            CredentialType::ApplicationSecret => "app_secret",
        }
    }
}

/// Stored credential with metadata
#[derive(Debug, Serialize, Deserialize)]
struct StoredCredential {
    value: String,
    expires_at: Option<chrono::DateTime<Utc>>,
    stored_at: chrono::DateTime<Utc>,
}

/// User information stored in the vault
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: String,
    pub avatar_url: Option<String>,
    pub provider: String,
}

/// OAuth token refresh response
#[derive(Debug, Deserialize)]
struct TokenRefreshResponse {
    access_token: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
}

impl CredentialVault {
    /// Create a new CredentialVault instance with keyring storage
    pub async fn new() -> Result<Self> {
        let service_name = "cyrup-chat".to_string();
        let http_client = Client::new();

        log::info!("CredentialVault initialized successfully with keyring storage");

        Ok(Self {
            service_name,
            http_client,
            provider_registry_key: "providers_index".to_string(),
        })
    }

    /// Create a vault with custom service name (for testing)
    pub async fn with_config(service_name: &str) -> Result<Self> {
        log::debug!("Initializing vault with service name: {}", service_name);

        let http_client = Client::new();

        Ok(Self {
            service_name: service_name.to_string(),
            http_client,
            provider_registry_key: "providers_index".to_string(),
        })
    }

    /// Generate a credential key for the given provider and credential type
    fn credential_key(&self, provider: &str, credential_type: &CredentialType) -> String {
        format!("{}_{}", provider, credential_type.key_suffix())
    }

    /// Store a credential in the secure vault
    pub async fn store_credential(
        &self,
        provider: &str,
        credential_type: CredentialType,
        credential: String,
        expires_at: Option<chrono::DateTime<Utc>>,
    ) -> Result<()> {
        let key = self.credential_key(provider, &credential_type);
        let stored_credential = StoredCredential {
            value: credential,
            expires_at,
            stored_at: Utc::now(),
        };

        let serialized =
            serde_json::to_string(&stored_credential).context("Failed to serialize credential")?;

        let entry =
            Entry::new(&self.service_name, &key).context("Failed to create keyring entry")?;

        entry
            .set_password(&serialized)
            .context("Failed to store credential in keyring")?;

        log::debug!(
            "Stored credential for provider '{}', type '{:?}'",
            provider,
            credential_type
        );

        self.register_provider(provider)?;

        Ok(())
    }

    /// Retrieve a credential from the secure vault
    pub async fn get_credential(
        &self,
        provider: &str,
        credential_type: &CredentialType,
    ) -> Result<Option<String>> {
        let key = self.credential_key(provider, credential_type);

        let entry =
            Entry::new(&self.service_name, &key).context("Failed to create keyring entry")?;

        match entry.get_password() {
            Ok(serialized) => {
                let stored_credential: StoredCredential = serde_json::from_str(&serialized)
                    .context("Failed to deserialize stored credential")?;

                // Check if credential has expired
                if let Some(expires_at) = stored_credential.expires_at
                    && Utc::now() > expires_at
                {
                    log::debug!(
                        "Credential for provider '{}', type '{:?}' has expired",
                        provider,
                        credential_type
                    );
                    return Ok(None);
                }

                Ok(Some(stored_credential.value))
            }
            Err(keyring::Error::NoEntry) => {
                log::debug!(
                    "No credential found for provider '{}', type '{:?}'",
                    provider,
                    credential_type
                );
                Ok(None)
            }
            Err(e) => Err(anyhow::anyhow!("Failed to retrieve credential: {}", e)),
        }
    }

    /// Delete a credential from the secure vault
    pub async fn delete_credential(
        &self,
        provider: &str,
        credential_type: &CredentialType,
    ) -> Result<bool> {
        let key = self.credential_key(provider, credential_type);

        let entry =
            Entry::new(&self.service_name, &key).context("Failed to create keyring entry")?;

        match entry.delete_credential() {
            Ok(_) => {
                log::debug!(
                    "Deleted credential for provider '{}', type '{:?}'",
                    provider,
                    credential_type
                );
                Ok(true)
            }
            Err(keyring::Error::NoEntry) => {
                log::debug!(
                    "No credential to delete for provider '{}', type '{:?}'",
                    provider,
                    credential_type
                );
                Ok(false)
            }
            Err(e) => Err(anyhow::anyhow!("Failed to delete credential: {}", e)),
        }
    }

    /// Delete all credentials for a provider
    pub async fn delete_provider_credentials(&self, provider: &str) -> Result<u32> {
        let credential_types = [
            CredentialType::AccessToken,
            CredentialType::RefreshToken,
            CredentialType::UserInfo,
            CredentialType::ClientCredentials,
            CredentialType::ApplicationSecret,
        ];

        let mut deleted_count = 0;
        for credential_type in credential_types {
            if self.delete_credential(provider, &credential_type).await? {
                deleted_count += 1;
            }
        }

        log::info!(
            "Deleted {} credentials for provider '{}'",
            deleted_count,
            provider
        );
        if deleted_count > 0
            && let Err(err) = self.unregister_provider(provider)
        {
            log::warn!("Failed to update provider registry after delete: {}", err);
        }
        Ok(deleted_count)
    }

    /// Check if a credential exists and is not expired
    pub async fn is_credential_valid(
        &self,
        provider: &str,
        credential_type: &CredentialType,
    ) -> Result<bool> {
        match self.get_credential(provider, credential_type).await? {
            Some(_credential) => Ok(true), // get_credential already checks expiration
            None => Ok(false),
        }
    }

    /// List all stored providers
    pub fn list_providers(&self) -> Result<Vec<String>> {
        let mut providers = self.read_provider_registry()?;
        if providers.is_empty() {
            providers.extend(["google".to_string(), "github".to_string()]);
        }
        let mut providers: Vec<_> = providers.into_iter().collect();
        providers.sort();
        Ok(providers)
    }

    /// Get a valid access token for the provider, refreshing if necessary
    pub async fn get_valid_access_token(&self, provider: &str) -> Result<String> {
        // First check if we have a valid access token
        if let Some(token) = self
            .get_credential(provider, &CredentialType::AccessToken)
            .await?
        {
            log::debug!(
                "Using cached valid access token for provider '{}'",
                provider
            );
            return Ok(token);
        }

        // Token is missing or expired, try to refresh
        self.refresh_token_if_needed(provider).await?;

        // Get the refreshed token
        if let Some(token) = self
            .get_credential(provider, &CredentialType::AccessToken)
            .await?
        {
            Ok(token)
        } else {
            Err(anyhow::anyhow!(
                "No valid access token available for provider '{}' after refresh attempt",
                provider
            ))
        }
    }

    /// Refresh the access token using the stored refresh token
    pub async fn refresh_token_if_needed(&self, provider: &str) -> Result<()> {
        // Get the refresh token
        let refresh_token = match self
            .get_credential(provider, &CredentialType::RefreshToken)
            .await?
        {
            Some(token) => token,
            None => {
                return Err(anyhow::anyhow!(
                    "No refresh token available for provider '{}'",
                    provider
                ));
            }
        };

        // Get client credentials for the token refresh
        let (client_id, client_secret) = match self
            .get_credential(provider, &CredentialType::ClientCredentials)
            .await?
        {
            Some(creds_str) => {
                let creds: serde_json::Value = serde_json::from_str(&creds_str)
                    .context("Failed to parse client credentials")?;
                let client_id = creds["client_id"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing client_id in credentials"))?;
                let client_secret = creds["client_secret"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("Missing client_secret in credentials"))?;
                (client_id.to_string(), client_secret.to_string())
            }
            None => {
                return Err(anyhow::anyhow!(
                    "No client credentials available for provider '{}'",
                    provider
                ));
            }
        };

        // Determine the token endpoint based on provider
        let token_endpoint = match provider {
            "google" => "https://oauth2.googleapis.com/token",
            "github" => "https://github.com/login/oauth/access_token",
            _ => return Err(anyhow::anyhow!("Unsupported provider: {}", provider)),
        };

        // Prepare refresh request
        let mut params = HashMap::new();
        params.insert("grant_type", "refresh_token");
        params.insert("refresh_token", &refresh_token);
        params.insert("client_id", &client_id);
        params.insert("client_secret", &client_secret);

        log::debug!("Refreshing access token for provider '{}'", provider);

        // Make the refresh request
        let response = self
            .http_client
            .post(token_endpoint)
            .form(&params)
            .send()
            .await
            .context("Failed to send token refresh request")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!(
                "Token refresh failed with status {}: {}",
                status,
                error_text
            ));
        }

        let token_response: TokenRefreshResponse = response
            .json()
            .await
            .context("Failed to parse token refresh response")?;

        // Calculate expiration time
        let expires_at = token_response.expires_in.map(|expires_in| {
            Utc::now() + Duration::seconds(expires_in as i64 - 300) // 5 minute buffer
        });

        // Store the new access token
        self.store_credential(
            provider,
            CredentialType::AccessToken,
            token_response.access_token,
            expires_at,
        )
        .await?;

        // Update refresh token if provided
        if let Some(new_refresh_token) = token_response.refresh_token {
            self.store_credential(
                provider,
                CredentialType::RefreshToken,
                new_refresh_token,
                None, // Refresh tokens typically don't expire
            )
            .await?;
        }

        log::info!(
            "Successfully refreshed access token for provider '{}'",
            provider
        );
        Ok(())
    }

    /// Check if the user is authenticated with the given provider
    pub async fn is_authenticated(&self, provider: &str) -> Result<bool> {
        // Check if we have both access and refresh tokens
        let has_access_token = self
            .is_credential_valid(provider, &CredentialType::AccessToken)
            .await?;
        let has_refresh_token = self
            .get_credential(provider, &CredentialType::RefreshToken)
            .await?
            .is_some();

        // User is authenticated if they have a valid access token OR a refresh token
        // (refresh token allows us to get a new access token)
        Ok(has_access_token || has_refresh_token)
    }

    /// Get stored user information for the provider
    pub async fn get_user_info(&self, provider: &str) -> Result<Option<UserInfo>> {
        if let Some(credential_str) = self
            .get_credential(provider, &CredentialType::UserInfo)
            .await?
        {
            let user_info: UserInfo =
                serde_json::from_str(&credential_str).context("Failed to deserialize user info")?;
            Ok(Some(user_info))
        } else {
            Ok(None)
        }
    }

    /// Store user information for the provider
    pub async fn store_user_info(&self, provider: &str, user_info: &UserInfo) -> Result<()> {
        let serialized =
            serde_json::to_string(user_info).context("Failed to serialize user info")?;

        self.store_credential(provider, CredentialType::UserInfo, serialized, None)
            .await
    }

    /// Store an application secret (embedded in binary)
    pub async fn store_application_secret(
        &self,
        provider: &str,
        secret_name: &str,
        secret_value: &str,
    ) -> Result<()> {
        let key = format!("{}_{}", provider, secret_name);
        let entry = Entry::new(&self.service_name, &key)
            .context("Failed to create keyring entry for application secret")?;

        entry
            .set_password(secret_value)
            .context("Failed to store application secret in keyring")?;

        log::debug!(
            "Stored application secret '{}' for provider '{}'",
            secret_name,
            provider
        );
        self.register_provider(provider)?;
        Ok(())
    }

    /// Get an application secret
    pub async fn get_application_secret(
        &self,
        provider: &str,
        secret_name: &str,
    ) -> Result<Option<String>> {
        let key = format!("{}_{}", provider, secret_name);
        let entry = Entry::new(&self.service_name, &key)
            .context("Failed to create keyring entry for application secret")?;

        match entry.get_password() {
            Ok(secret) => Ok(Some(secret)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(anyhow::anyhow!(
                "Failed to retrieve application secret: {}",
                e
            )),
        }
    }

    /// Seed application secrets from environment variables for known OAuth providers.
    ///
    /// This method reads OAuth client credentials from environment variables and stores them
    /// securely in the OS keyring. Secrets are only written if they don't already exist in
    /// the keyring, preserving manually configured values.
    ///
    /// # Environment Variables
    ///
    /// **Google OAuth:**
    /// - `GOOGLE_CLIENT_ID` - OAuth 2.0 client ID from Google Cloud Console
    /// - `GOOGLE_CLIENT_SECRET` - OAuth 2.0 client secret from Google Cloud Console
    ///
    /// **GitHub OAuth:**
    /// - `GITHUB_CLIENT_ID` - OAuth App client ID from GitHub Developer Settings
    /// - `GITHUB_CLIENT_SECRET` - OAuth App client secret from GitHub Developer Settings
    /// - `GITHUB_PRIVATE_KEY` - Private key for GitHub App authentication (PEM format)
    ///
    /// # Production Deployment
    ///
    /// In production environments, provision secrets via:
    ///
    /// 1. **Container/Cloud deployment:**
    ///    - Set environment variables in container orchestration (Docker, Kubernetes)
    ///    - Use secret management services (AWS Secrets Manager, Azure Key Vault, GCP Secret Manager)
    ///    - Inject secrets at runtime via init containers or sidecar patterns
    ///
    /// 2. **Native desktop deployment:**
    ///    - Pre-populate secrets in OS keyring during installation
    ///    - Use deployment scripts that call `store_application_secret()` directly
    ///    - Set environment variables in app launcher/service configuration
    ///
    /// 3. **Development:**
    ///    - Use `.env` files loaded by process manager (not committed to git)
    ///    - Set variables in IDE launch configurations
    ///    - Export in shell profile for local development
    ///
    /// **Security Notes:**
    /// - Never hardcode secrets in application binaries
    /// - Never commit secrets to version control
    /// - Rotate secrets regularly and update keyring entries
    /// - Use separate credentials for dev/staging/production environments
    ///
    /// # Behavior
    ///
    /// - Empty environment variables are ignored (no error)
    /// - Existing keyring entries are preserved (no overwrite)
    /// - Missing environment variables are silently skipped
    /// - Only writes to keyring when both env var exists and keyring is empty
    pub async fn seed_application_secrets_from_env(&self) -> Result<()> {
        self.try_seed_secret("google", "client_id", "GOOGLE_CLIENT_ID")
            .await?;
        self.try_seed_secret("google", "client_secret", "GOOGLE_CLIENT_SECRET")
            .await?;
        self.try_seed_secret("github", "client_id", "GITHUB_CLIENT_ID")
            .await?;
        self.try_seed_secret("github", "client_secret", "GITHUB_CLIENT_SECRET")
            .await?;
        self.try_seed_secret("github", "private_key", "GITHUB_PRIVATE_KEY")
            .await?;
        Ok(())
    }

    async fn try_seed_secret(
        &self,
        provider: &str,
        secret_name: &str,
        env_key: &str,
    ) -> Result<()> {
        if let Ok(value) = std::env::var(env_key) {
            if value.is_empty() {
                return Ok(());
            }

            if let Some(existing) = self.get_application_secret(provider, secret_name).await?
                && !existing.is_empty()
            {
                return Ok(());
            }

            self.store_application_secret(provider, secret_name, &value)
                .await?;
            log::info!(
                "Seeded {} secret for provider '{}' from environment",
                secret_name,
                provider
            );
        }
        Ok(())
    }

    fn register_provider(&self, provider: &str) -> Result<()> {
        let mut providers = self.read_provider_registry()?;
        providers.insert(provider.to_string());
        self.write_provider_registry(&providers)
    }

    fn unregister_provider(&self, provider: &str) -> Result<()> {
        let mut providers = self.read_provider_registry()?;
        providers.remove(provider);
        self.write_provider_registry(&providers)
    }

    fn read_provider_registry(&self) -> Result<HashSet<String>> {
        let entry = Entry::new(&self.service_name, &self.provider_registry_key)
            .context("Failed to create keyring entry for provider registry")?;

        match entry.get_password() {
            Ok(serialized) => {
                let providers: HashSet<String> = serde_json::from_str(&serialized)
                    .context("Failed to deserialize provider registry")?;
                Ok(providers)
            }
            Err(keyring::Error::NoEntry) => Ok(HashSet::new()),
            Err(e) => Err(anyhow::anyhow!("Failed to read provider registry: {}", e)),
        }
    }

    fn write_provider_registry(&self, providers: &HashSet<String>) -> Result<()> {
        let entry = Entry::new(&self.service_name, &self.provider_registry_key)
            .context("Failed to create keyring entry for provider registry")?;

        if providers.is_empty() {
            match entry.delete_credential() {
                Ok(_) | Err(keyring::Error::NoEntry) => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(anyhow::anyhow!(
                        "Failed to delete provider registry entry: {}",
                        e
                    ));
                }
            }
        }

        let serialized =
            serde_json::to_string(providers).context("Failed to serialize provider registry")?;
        entry
            .set_password(&serialized)
            .context("Failed to store provider registry")
    }

    /// Close the vault connection
    pub async fn close(&self) -> Result<()> {
        // No explicit cleanup needed for keyring storage
        log::info!("Vault connection closed");
        Ok(())
    }
}
