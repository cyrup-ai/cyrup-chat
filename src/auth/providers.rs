use crate::auth::{AuthState, CredentialVault, UserInfo};
use anyhow::Result;
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};

/// Supported OAuth providers
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provider {
    Google,
    GitHub,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Provider::Google => write!(f, "Google"),
            Provider::GitHub => write!(f, "GitHub"),
        }
    }
}

impl Provider {
    pub fn name(&self) -> &'static str {
        match self {
            Provider::Google => "Google",
            Provider::GitHub => "GitHub",
        }
    }

    pub fn icon_class(&self) -> &'static str {
        match self {
            Provider::Google => "google-icon",
            Provider::GitHub => "github-icon",
        }
    }

    pub fn button_class(&self) -> &'static str {
        match self {
            Provider::Google => "google-login-button",
            Provider::GitHub => "github-login-button",
        }
    }

    pub fn all() -> Vec<Provider> {
        vec![Provider::Google, Provider::GitHub]
    }
}

/// Generic OAuth login function that works with any provider
pub async fn login_with_provider(provider: Provider) -> Result<AuthState> {
    log::debug!("login_with_provider called for: {}", provider.name());
    log::info!("Starting OAuth login for provider: {}", provider.name());

    let result = match provider {
        Provider::Google => {
            log::debug!("Calling login_google()...");
            login_google().await
        }
        Provider::GitHub => {
            log::debug!("Calling login_github()...");
            login_github().await
        }
    };

    match &result {
        Ok(_) => log::info!("OAuth login successful for {}", provider.name()),
        Err(e) => log::error!("OAuth login failed for {}: {}", provider.name(), e),
    }

    result
}

/// Google OAuth login implementation using google-oauth library
async fn login_google() -> Result<AuthState> {
    use google_oauth::{AccessType, Login, UserInfoBuilder as GoogleUserInfo};

    log::info!("Starting Google OAuth login");

    // Use the google-oauth library exactly as designed - it handles everything
    let oauth_response = Login::from_env()
        .scopes([
            "https://www.googleapis.com/auth/userinfo.email",
            "https://www.googleapis.com/auth/userinfo.profile",
        ])?
        .access_type(AccessType::Offline) // Get refresh token
        .port(33999) // Library starts local server automatically
        .timeout(std::time::Duration::from_secs(300))
        .login()
        .await
        .map_err(|e| anyhow::anyhow!("Google OAuth failed: {}", e))?;

    log::info!("Successfully completed Google OAuth flow");

    // Get real user info using the access token
    let user_info = GoogleUserInfo::token(&*oauth_response.access_token)
        .get_info()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get Google user info: {}", e))?;

    log::info!(
        "Successfully fetched Google user info for: {}",
        user_info.email
    );

    // Create AuthState with real data from Google OAuth
    let auth_state = AuthState {
        provider: Provider::Google,
        user: UserInfo {
            email: user_info.email,
            name: user_info.name.unwrap_or_default(),
            picture: user_info.picture.unwrap_or_default(),
            id: user_info.id,
            username: None, // Google doesn't have usernames
        },
    };

    // Store tokens directly to vault
    let vault = CredentialVault::new().await?;
    let provider_name = "google";
    let expires_at = Some(Utc::now() + Duration::seconds(oauth_response.expires_in as i64));

    vault
        .store_credential(
            provider_name,
            crate::auth::CredentialType::AccessToken,
            oauth_response.access_token.to_string(),
            expires_at,
        )
        .await?;

    if let Some(refresh_token) = oauth_response.refresh_token {
        vault
            .store_credential(
                provider_name,
                crate::auth::CredentialType::RefreshToken,
                refresh_token.to_string(),
                None,
            )
            .await?;
    }

    // Save to secure vault only on new login (credentials definitely changed)
    auth_state.save().await?;

    log::info!("Successfully saved new Google OAuth credentials to secure vault");

    Ok(auth_state)
}

/// GitHub OAuth login implementation using github-oauth library
async fn login_github() -> Result<AuthState> {
    use github_oauth::{Login, UserInfoBuilder as GitHubUserInfo};

    log::info!("Starting GitHub OAuth login with real github-oauth library");

    // Use the github-oauth library exactly as designed - it handles everything
    let oauth_response = Login::from_env()
        .scopes(["user:email", "read:user"])?
        .port(33999) // Library starts local server automatically
        .timeout(std::time::Duration::from_secs(300))
        .login()
        .await
        .map_err(|e| anyhow::anyhow!("GitHub OAuth failed: {}", e))?;

    log::info!("Successfully completed GitHub OAuth flow");

    // Get real user info using the access token
    let user_info = GitHubUserInfo::token(oauth_response.access_token.as_str())
        .get_info()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get GitHub user info: {}", e))?;

    log::info!(
        "Successfully fetched GitHub user info for: {}",
        user_info.login
    );

    // Handle optional email (GitHub users can make email private)
    let user_email = user_info
        .email
        .unwrap_or_else(|| format!("{}@users.noreply.github.com", user_info.login));
    let user_name = user_info.name.unwrap_or_else(|| user_info.login.clone());

    // Create AuthState with real GitHub user data
    let auth_state = AuthState {
        provider: Provider::GitHub,
        user: UserInfo {
            email: user_email,
            name: user_name,
            picture: user_info.avatar_url,
            id: user_info.id.to_string(),
            username: Some(user_info.login),
        },
    };

    // Store tokens directly to vault
    let vault = CredentialVault::new().await?;
    let provider_name = "github";

    vault
        .store_credential(
            provider_name,
            crate::auth::CredentialType::AccessToken,
            oauth_response.access_token.to_string(),
            None, // GitHub tokens don't expire
        )
        .await?;

    if let Some(refresh_token) = oauth_response.refresh_token {
        vault
            .store_credential(
                provider_name,
                crate::auth::CredentialType::RefreshToken,
                refresh_token.to_string(),
                None,
            )
            .await?;
    }

    // Save to secure vault (this will also save to legacy file for backward compatibility)
    auth_state.save().await?;

    log::info!("Successfully saved GitHub OAuth credentials to secure vault");

    Ok(auth_state)
}

/// Refresh token for a specific provider
pub async fn refresh_token_for_provider(auth_state: &mut AuthState) -> Result<()> {
    match auth_state.provider {
        Provider::Google => refresh_google_token(auth_state).await,
        Provider::GitHub => refresh_github_token(auth_state).await,
    }
}

async fn refresh_google_token(auth_state: &mut AuthState) -> Result<()> {
    use reqwest::Client;
    use serde_json::json;

    // Get refresh token from vault instead of auth_state
    let vault = CredentialVault::new().await?;
    let provider_name = auth_state.provider.to_string().to_lowercase();
    let refresh_credential = vault
        .get_credential(&provider_name, &crate::auth::CredentialType::RefreshToken)
        .await?
        .ok_or_else(|| anyhow::anyhow!("No refresh token available for {} OAuth", provider_name))?;
    let refresh_token = &refresh_credential;

    // Get Google OAuth credentials from vault
    let client_id = crate::auth::embedded_vault::get_google_oauth_client_id()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get Google OAuth client ID from vault: {}", e))?;
    let client_secret = crate::auth::embedded_vault::get_google_oauth_client_secret()
        .await
        .map_err(|e| {
            anyhow::anyhow!("Failed to get Google OAuth client secret from vault: {}", e)
        })?;

    let client = Client::new();
    let params = json!({
        "client_id": client_id.as_str(),
        "client_secret": client_secret.as_str(),
        "refresh_token": refresh_token,
        "grant_type": "refresh_token"
    });

    let response = client
        .post("https://oauth2.googleapis.com/token")
        .json(&params)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to send refresh request: {}", e))?;

    if !response.status().is_success() {
        let error_body = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(anyhow::anyhow!(
            "Google token refresh failed: {}",
            error_body
        ));
    }

    let token_response: serde_json::Value = response
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse refresh response: {}", e))?;

    // Update tokens in vault with new access token
    if let Some(new_access_token) = token_response.get("access_token").and_then(|v| v.as_str()) {
        let vault = CredentialVault::new().await?;
        let provider_name = auth_state.provider.to_string().to_lowercase();

        // Calculate expiration time if provided
        let expires_at = token_response
            .get("expires_in")
            .and_then(|v| v.as_u64())
            .map(|expires_in| Utc::now() + Duration::seconds(expires_in as i64));

        vault
            .store_credential(
                &provider_name,
                crate::auth::CredentialType::AccessToken,
                new_access_token.to_string(),
                expires_at,
            )
            .await?;

        // Update refresh token if provided
        if let Some(new_refresh_token) =
            token_response.get("refresh_token").and_then(|v| v.as_str())
        {
            vault
                .store_credential(
                    &provider_name,
                    crate::auth::CredentialType::RefreshToken,
                    new_refresh_token.to_string(),
                    None,
                )
                .await?;
        }

        // Only save to vault if tokens actually changed
        log::info!("Successfully refreshed OAuth token - updated vault");
        Ok(())
    } else {
        Err(anyhow::anyhow!("No access_token in refresh response"))
    }
}

async fn refresh_github_token(_auth_state: &mut AuthState) -> Result<()> {
    // GitHub tokens don't expire, so no refresh needed
    // Return an error suggesting re-authentication if needed
    Err(anyhow::anyhow!(
        "GitHub tokens do not expire. Re-authentication required if access is denied."
    ))
}
