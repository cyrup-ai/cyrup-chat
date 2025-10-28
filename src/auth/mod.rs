use anyhow::Result;
// Removed unused DateTime and Utc imports
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

pub mod embedded_vault;
pub mod google_config;
pub mod native_oauth;
pub mod providers;
pub mod services;
pub mod vault;
pub mod vault_auth_manager;

pub use embedded_vault::{
    SecureString, get_github_oauth_client_id, get_github_oauth_client_secret,
    get_github_private_key, get_google_oauth_client_id, get_google_oauth_client_secret,
    initialize_vault,
};
pub use native_oauth::login_with_provider_native;
pub use providers::{Provider, login_with_provider};
pub use vault::{CredentialType, CredentialVault};
pub use vault_auth_manager::{AuthResult, AuthStatus, VaultAuthManager};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthState {
    pub provider: Provider,
    pub user: UserInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserInfo {
    pub email: String,
    pub name: String,
    pub picture: String,
    pub id: String,
    pub username: Option<String>, // GitHub has usernames, Google doesn't
}

// StoredTokens struct removed - credentials now live in vault only

impl AuthState {
    /// Get the path for storing auth state (legacy file-based storage)
    fn auth_file_path() -> Result<PathBuf> {
        let home =
            dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
        Ok(home.join(".cyrup").join("auth.json"))
    }

    /// Save auth state to persistent storage (user info only - tokens handled by vault directly)
    pub async fn save(&self) -> Result<()> {
        self.save_to_vault().await
    }

    /// Save auth state to secure vault (user info only - tokens handled separately)
    pub async fn save_to_vault(&self) -> Result<()> {
        let vault = CredentialVault::new().await?;
        let provider_name = self.provider.to_string().to_lowercase();

        // Store user info using the vault's UserInfo type
        let vault_user_info = vault::UserInfo {
            id: self.user.id.clone(),
            email: self.user.email.clone(),
            name: self.user.name.clone(),
            avatar_url: if self.user.picture.is_empty() {
                None
            } else {
                Some(self.user.picture.clone())
            },
            provider: provider_name.clone(),
        };

        vault
            .store_user_info(&provider_name, &vault_user_info)
            .await?;

        log::info!(
            "Saved auth state to secure vault for provider: {}",
            provider_name
        );
        Ok(())
    }

    /// Save auth state to legacy file (for backward compatibility)
    pub async fn save_to_file(&self) -> Result<()> {
        let path = Self::auth_file_path()?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let json = serde_json::to_string_pretty(self)?;
        tokio::fs::write(path, json).await?;
        Ok(())
    }

    /// Load auth state from secure vault or legacy file
    pub async fn load() -> Result<Option<Self>> {
        // Try loading from vault first
        if let Some(state) = Self::load_from_vault().await? {
            return Ok(Some(state));
        }

        // Fallback to legacy file storage
        if let Some(state) = Self::load_from_file().await? {
            // Migrate to vault and clean up file
            if let Err(e) = state.save_to_vault().await {
                log::warn!("Failed to migrate auth state to vault: {}", e);
            } else {
                log::info!("Migrated auth state from file to secure vault");
                if let Err(e) = Self::delete_file().await {
                    log::warn!("Failed to clean up legacy auth file: {}", e);
                }
            }
            return Ok(Some(state));
        }

        Ok(None)
    }

    /// Load auth state from secure vault (user info only - tokens accessed via vault)
    pub async fn load_from_vault() -> Result<Option<Self>> {
        let vault = CredentialVault::new().await?;

        // Try both Google and GitHub providers
        for provider in [Provider::Google, Provider::GitHub] {
            let provider_name = provider.to_string().to_lowercase();

            // Check if user is authenticated with this provider
            if vault.is_authenticated(&provider_name).await? {
                // Get user info from vault
                if let Some(vault_user_info) = vault.get_user_info(&provider_name).await? {
                    let user = UserInfo {
                        id: vault_user_info.id,
                        email: vault_user_info.email,
                        name: vault_user_info.name,
                        picture: vault_user_info.avatar_url.unwrap_or_default(),
                        username: None, // Will be populated by provider-specific logic if needed
                    };

                    log::info!(
                        "Loaded auth state from secure vault for provider: {}",
                        provider_name
                    );
                    return Ok(Some(AuthState { provider, user }));
                }
            }
        }

        Ok(None)
    }

    /// Load auth state from legacy file
    pub async fn load_from_file() -> Result<Option<Self>> {
        let path = Self::auth_file_path()?;
        if !path.exists() {
            return Ok(None);
        }
        let content = tokio::fs::read_to_string(path).await?;
        let state: AuthState = serde_json::from_str(&content)?;
        Ok(Some(state))
    }

    /// Delete auth state from both vault and legacy file
    pub async fn delete() -> Result<()> {
        let vault = CredentialVault::new().await?;

        // Delete from vault for all providers
        for provider in [Provider::Google, Provider::GitHub] {
            let provider_name = provider.to_string().to_lowercase();
            let _ = vault.delete_provider_credentials(&provider_name).await;
        }

        // Delete legacy file
        Self::delete_file().await?;

        log::info!("Deleted all auth state from vault and file");
        Ok(())
    }

    /// Delete legacy auth file
    pub async fn delete_file() -> Result<()> {
        let path = Self::auth_file_path()?;
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
        Ok(())
    }

    /// Check if token needs refresh (uses vault to check expiration)
    pub async fn needs_refresh(&self) -> Result<bool> {
        let vault = CredentialVault::new().await?;
        let provider_name = self.provider.to_string().to_lowercase();

        // Check if access token is valid (not expired)
        let is_valid = vault
            .is_credential_valid(&provider_name, &CredentialType::AccessToken)
            .await?;
        Ok(!is_valid)
    }

    /// Refresh the access token (uses vault methods)
    pub async fn refresh_token(&self) -> Result<()> {
        let vault = CredentialVault::new().await?;
        let provider_name = self.provider.to_string().to_lowercase();
        vault.refresh_token_if_needed(&provider_name).await
    }

    /// Get a valid access token (refreshing if needed)
    pub async fn get_valid_token(&self) -> Result<String> {
        let vault = CredentialVault::new().await?;
        let provider_name = self.provider.to_string().to_lowercase();
        vault.get_valid_access_token(&provider_name).await
    }
}

/// Backward compatible Google OAuth login (deprecated - use login_with_provider instead)
pub async fn login() -> Result<AuthState> {
    login_with_provider(Provider::Google).await
}

/// Logout by deleting stored credentials
pub async fn logout() -> Result<()> {
    AuthState::delete().await
}
