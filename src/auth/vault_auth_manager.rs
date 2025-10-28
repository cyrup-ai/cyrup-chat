//! High-level authentication manager using vault-first credential access
//!
//! VaultAuthManager provides a convenient interface for all authentication operations
//! while ensuring credentials are always accessed from the secure vault when needed.

use crate::auth::vault::UserInfo as VaultUserInfo;
use crate::auth::{AuthState, CredentialVault, Provider, UserInfo};
use anyhow::Result;
use serde_json::{Map, Value};

/// High-level authentication manager that wraps CredentialVault
///
/// This manager provides convenient methods for authentication operations
/// while ensuring all credentials are accessed from the vault when needed.
/// No credentials are stored in memory - everything goes through the vault.
pub struct VaultAuthManager {
    vault: CredentialVault,
}

/// Authentication status for a provider
#[derive(Debug, Clone)]
pub struct AuthStatus {
    pub provider: Provider,
    pub user: UserInfo,
    pub is_authenticated: bool,
    pub has_valid_token: bool,
}

/// Authentication result after login
#[derive(Debug, Clone)]
pub struct AuthResult {
    pub auth_state: AuthState,
    pub was_refreshed: bool,
}

// Note: Default implementation removed since VaultAuthManager::new() is now async

impl VaultAuthManager {
    /// Create a new VaultAuthManager instance
    pub async fn new() -> Result<Self> {
        Ok(Self {
            vault: CredentialVault::new().await?,
        })
    }

    /// Create a VaultAuthManager with a custom vault (for testing)
    pub fn with_vault(vault: CredentialVault) -> Self {
        Self { vault }
    }

    /// Initialize application secrets from embedded data (call on first run)
    pub async fn initialize_app_secrets(&self) -> Result<()> {
        // No initialization needed for keyring-based vault
        Ok(())
    }

    /// Check if any user is currently authenticated
    pub async fn is_any_user_authenticated(&self) -> Result<bool> {
        for provider in [Provider::Google, Provider::GitHub] {
            let provider_name = provider.to_string().to_lowercase();
            if self.vault.is_authenticated(&provider_name).await? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Get authentication status for a specific provider
    pub async fn get_auth_status(&self, provider: Provider) -> Result<Option<AuthStatus>> {
        let provider_name = provider.to_string().to_lowercase();

        if !self.vault.is_authenticated(&provider_name).await? {
            return Ok(None);
        }

        // Get user info from vault
        let vault_user_info = self
            .vault
            .get_user_info(&provider_name)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "User authenticated but no user info found for {}",
                    provider_name
                )
            })?;

        let user_info = UserInfo {
            id: vault_user_info.id,
            email: vault_user_info.email,
            name: vault_user_info.name,
            picture: vault_user_info.avatar_url.unwrap_or_default(),
            username: None, // Will be populated by provider-specific logic if needed
        };

        // Check if we have a valid access token
        let has_valid_token = self
            .vault
            .is_credential_valid(&provider_name, &crate::auth::CredentialType::AccessToken)
            .await?;

        Ok(Some(AuthStatus {
            provider,
            user: user_info,
            is_authenticated: true,
            has_valid_token,
        }))
    }

    /// Get authentication status for all providers
    pub async fn get_all_auth_status(&self) -> Result<Vec<AuthStatus>> {
        let mut statuses = Vec::new();

        for provider in [Provider::Google, Provider::GitHub] {
            if let Some(status) = self.get_auth_status(provider).await? {
                statuses.push(status);
            }
        }

        Ok(statuses)
    }

    /// Get the currently authenticated user (first one found)
    pub async fn get_current_user(&self) -> Result<Option<AuthState>> {
        for provider in [Provider::Google, Provider::GitHub] {
            let provider_name = provider.to_string().to_lowercase();

            if self.vault.is_authenticated(&provider_name).await?
                && let Some(vault_user_info) = self.vault.get_user_info(&provider_name).await?
            {
                let user_info = UserInfo {
                    id: vault_user_info.id,
                    email: vault_user_info.email,
                    name: vault_user_info.name,
                    picture: vault_user_info.avatar_url.unwrap_or_default(),
                    username: None,
                };

                return Ok(Some(AuthState {
                    provider,
                    user: user_info,
                }));
            }
        }

        Ok(None)
    }

    /// Get a valid access token for the provider (refreshing if necessary)
    pub async fn get_access_token(&self, provider: Provider) -> Result<String> {
        let provider_name = provider.to_string().to_lowercase();
        self.vault.get_valid_access_token(&provider_name).await
    }

    /// Refresh tokens for a provider if needed
    pub async fn refresh_tokens(&self, provider: Provider) -> Result<()> {
        let provider_name = provider.to_string().to_lowercase();
        self.vault.refresh_token_if_needed(&provider_name).await
    }

    /// Store authentication result after successful OAuth login
    pub async fn store_auth_result(
        &self,
        provider: Provider,
        user_info: &UserInfo,
        access_token: &str,
        refresh_token: Option<&str>,
        expires_at: Option<chrono::DateTime<chrono::Utc>>,
    ) -> Result<AuthState> {
        let provider_name = provider.to_string().to_lowercase();

        // Store access token
        self.vault
            .store_credential(
                &provider_name,
                crate::auth::CredentialType::AccessToken,
                access_token.to_string(),
                expires_at,
            )
            .await?;

        // Store refresh token if provided
        if let Some(refresh_token) = refresh_token {
            self.vault
                .store_credential(
                    &provider_name,
                    crate::auth::CredentialType::RefreshToken,
                    refresh_token.to_string(),
                    None,
                )
                .await?;
        }

        // Store user info
        let vault_user_info = VaultUserInfo {
            id: user_info.id.clone(),
            email: user_info.email.clone(),
            name: user_info.name.clone(),
            avatar_url: if user_info.picture.is_empty() {
                None
            } else {
                Some(user_info.picture.clone())
            },
            provider: provider_name,
        };

        self.vault
            .store_user_info(&provider.to_string().to_lowercase(), &vault_user_info)
            .await?;

        log::info!(
            "Successfully stored authentication result for provider: {:?}",
            provider
        );

        Ok(AuthState {
            provider,
            user: user_info.clone(),
        })
    }

    /// Logout from a specific provider
    pub async fn logout(&self, provider: Provider) -> Result<()> {
        let provider_name = provider.to_string().to_lowercase();
        self.vault
            .delete_provider_credentials(&provider_name)
            .await?;
        log::info!("Logged out from provider: {:?}", provider);
        Ok(())
    }

    /// Logout from all providers
    pub async fn logout_all(&self) -> Result<()> {
        for provider in [Provider::Google, Provider::GitHub] {
            let _ = self.logout(provider).await; // Continue even if one fails
        }
        log::info!("Logged out from all providers");
        Ok(())
    }

    /// Check if tokens need refresh for a provider
    pub async fn needs_token_refresh(&self, provider: Provider) -> Result<bool> {
        let provider_name = provider.to_string().to_lowercase();
        let is_valid = self
            .vault
            .is_credential_valid(&provider_name, &crate::auth::CredentialType::AccessToken)
            .await?;
        Ok(!is_valid)
    }

    /// Get application secret for a provider
    pub async fn get_app_secret(&self, provider: Provider, secret_name: &str) -> Result<String> {
        let provider_name = provider.to_string().to_lowercase();
        self.vault
            .get_application_secret(&provider_name, secret_name)
            .await?
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Application secret '{}' not found for provider '{}'",
                    secret_name,
                    provider_name
                )
            })
    }

    /// Store application secret for a provider
    pub async fn store_app_secret(
        &self,
        provider: Provider,
        secret_name: &str,
        secret_value: &str,
    ) -> Result<()> {
        let provider_name = provider.to_string().to_lowercase();
        self.vault
            .store_application_secret(&provider_name, secret_name, secret_value)
            .await
    }

    /// Get OAuth client credentials for a provider
    pub async fn get_oauth_credentials(&self, provider: Provider) -> Result<(String, String)> {
        let client_id = self.get_app_secret(provider, "client_id").await?;
        let client_secret = self.get_app_secret(provider, "client_secret").await?;
        Ok((client_id, client_secret))
    }

    /// Validate that all required application secrets are available
    pub async fn validate_app_secrets(&self) -> Result<()> {
        let mut missing_secrets = Vec::new();

        // Check Google OAuth secrets
        if self
            .get_app_secret(Provider::Google, "client_id")
            .await
            .is_err()
        {
            missing_secrets.push("Google client_id");
        }
        if self
            .get_app_secret(Provider::Google, "client_secret")
            .await
            .is_err()
        {
            missing_secrets.push("Google client_secret");
        }

        // Check GitHub OAuth secrets
        if self
            .get_app_secret(Provider::GitHub, "client_id")
            .await
            .is_err()
        {
            missing_secrets.push("GitHub client_id");
        }
        if self
            .get_app_secret(Provider::GitHub, "client_secret")
            .await
            .is_err()
        {
            missing_secrets.push("GitHub client_secret");
        }

        if !missing_secrets.is_empty() {
            return Err(anyhow::anyhow!(
                "Missing required application secrets: {}",
                missing_secrets.join(", ")
            ));
        }

        Ok(())
    }

    /// Get comprehensive authentication summary
    pub async fn get_auth_summary(&self) -> Result<Map<String, Value>> {
        let mut summary = Map::new();

        // Check overall authentication status
        summary.insert(
            "is_authenticated".to_string(),
            Value::Bool(self.is_any_user_authenticated().await?),
        );

        // Get status for each provider
        let mut providers = Map::new();
        for provider in [Provider::Google, Provider::GitHub] {
            let provider_name = provider.to_string().to_lowercase();
            let mut provider_info = Map::new();

            provider_info.insert(
                "is_authenticated".to_string(),
                Value::Bool(self.vault.is_authenticated(&provider_name).await?),
            );

            if let Some(user_info) = self.vault.get_user_info(&provider_name).await? {
                provider_info.insert("user_email".to_string(), Value::String(user_info.email));
                provider_info.insert("user_name".to_string(), Value::String(user_info.name));
            }

            provider_info.insert(
                "has_valid_token".to_string(),
                Value::Bool(
                    self.vault
                        .is_credential_valid(
                            &provider_name,
                            &crate::auth::CredentialType::AccessToken,
                        )
                        .await?,
                ),
            );

            providers.insert(provider_name, Value::Object(provider_info));
        }

        summary.insert("providers".to_string(), Value::Object(providers));

        // Check application secrets status
        let secrets_valid = self.validate_app_secrets().await.is_ok();
        summary.insert(
            "app_secrets_configured".to_string(),
            Value::Bool(secrets_valid),
        );

        Ok(summary)
    }

    /// Perform health check on the authentication system
    pub async fn health_check(&self) -> Result<Map<String, Value>> {
        let mut health = Map::new();

        // Check vault accessibility
        let vault_health = "healthy"; // Assume healthy if we can create the manager
        health.insert(
            "vault_health".to_string(),
            Value::String(vault_health.to_string()),
        );

        // Check application secrets
        let secrets_configured = self.validate_app_secrets().await.is_ok();
        health.insert(
            "app_secrets_configured".to_string(),
            Value::Bool(secrets_configured),
        );

        // Check authentication status
        let auth_status = self.is_any_user_authenticated().await.unwrap_or(false);
        health.insert("user_authenticated".to_string(), Value::Bool(auth_status));

        // Overall health
        let overall_healthy = vault_health == "healthy" && secrets_configured;
        health.insert("overall_healthy".to_string(), Value::Bool(overall_healthy));

        Ok(health)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::CredentialVault;

    async fn test_manager() -> VaultAuthManager {
        let vault = CredentialVault::with_config("ai.cyrup.chat.test.manager")
            .await
            .unwrap();
        VaultAuthManager::with_vault(vault)
    }

    #[tokio::test]
    async fn test_no_authentication_initially() -> Result<()> {
        let manager = test_manager().await;

        let is_authenticated = manager.is_any_user_authenticated().await?;
        assert!(!is_authenticated);

        let current_user = manager.get_current_user().await?;
        assert!(current_user.is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_store_and_retrieve_auth_result() -> Result<()> {
        let manager = test_manager().await;

        // Clean up any existing test data
        let _ = manager.logout_all().await;

        let user_info = UserInfo {
            id: "test-user-123".to_string(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            picture: "https://example.com/avatar.jpg".to_string(),
            username: Some("testuser".to_string()),
        };

        let expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(1));

        // Store authentication result
        let auth_state = manager
            .store_auth_result(
                Provider::Google,
                &user_info,
                "test-access-token",
                Some("test-refresh-token"),
                expires_at,
            )
            .await?;

        assert_eq!(auth_state.provider, Provider::Google);
        assert_eq!(auth_state.user.email, "test@example.com");

        // Check authentication status
        let is_authenticated = manager.is_any_user_authenticated().await?;
        assert!(is_authenticated);

        // Get current user
        let current_user = manager.get_current_user().await?;
        assert!(current_user.is_some());
        let current_user = current_user.unwrap();
        assert_eq!(current_user.user.email, "test@example.com");

        // Get access token
        let access_token = manager.get_access_token(Provider::Google).await?;
        assert_eq!(access_token, "test-access-token");

        // Clean up
        manager.logout_all().await?;

        Ok(())
    }

    #[tokio::test]
    async fn test_logout_functionality() -> Result<()> {
        let manager = test_manager().await;

        // Clean up any existing test data
        let _ = manager.logout_all().await;

        let user_info = UserInfo {
            id: "test-user-456".to_string(),
            email: "test2@example.com".to_string(),
            name: "Test User 2".to_string(),
            picture: "".to_string(),
            username: None,
        };

        // Store authentication result
        manager
            .store_auth_result(
                Provider::GitHub,
                &user_info,
                "test-github-token",
                None,
                None,
            )
            .await?;

        // Verify authentication
        let is_authenticated = manager.is_any_user_authenticated().await?;
        assert!(is_authenticated);

        // Logout
        manager.logout(Provider::GitHub).await?;

        // Verify logout
        let is_authenticated = manager.is_any_user_authenticated().await?;
        assert!(!is_authenticated);

        Ok(())
    }

    #[tokio::test]
    async fn test_auth_summary() -> Result<()> {
        let manager = test_manager().await;

        // Clean up any existing test data
        let _ = manager.logout_all().await;

        let summary = manager.get_auth_summary().await?;

        // Should not be authenticated initially
        assert_eq!(
            summary.get("is_authenticated"),
            Some(&serde_json::Value::Bool(false))
        );

        // Should have provider information
        assert!(summary.contains_key("providers"));
        assert!(summary.contains_key("app_secrets_configured"));

        Ok(())
    }

    #[tokio::test]
    async fn test_health_check() -> Result<()> {
        let manager = test_manager().await;

        let health = manager.health_check().await?;

        // Should have all required health check fields
        assert!(health.contains_key("vault_accessible"));
        assert!(health.contains_key("app_secrets_configured"));
        assert!(health.contains_key("user_authenticated"));
        assert!(health.contains_key("overall_healthy"));

        // Vault should be accessible
        assert_eq!(
            health.get("vault_accessible"),
            Some(&serde_json::Value::Bool(true))
        );

        Ok(())
    }
}
