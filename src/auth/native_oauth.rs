use crate::app::OAuthCallbacks;
use crate::auth::providers::Provider;
use crate::auth::{AuthState, UserInfo};
use anyhow::Result;
use chrono::{Duration, Utc};

use github_oauth::{Login, UserInfoBuilder as GitHubUserInfo};
use google_oauth::{Login as GoogleLogin, UserInfoBuilder as GoogleUserInfo};

/// Login with provider using native OAuth flow
pub async fn login_with_provider_native(
    provider: Provider,
    oauth_callbacks: OAuthCallbacks,
) -> Result<AuthState> {
    match provider {
        Provider::Google => login_google_native(oauth_callbacks).await,
        Provider::GitHub => login_github_native(oauth_callbacks).await,
    }
}

async fn login_google_native(_oauth_callbacks: OAuthCallbacks) -> Result<AuthState> {
    log::info!("Starting native Google OAuth login with embedded vault credentials");

    // Get Google OAuth credentials from embedded vault
    let client_id = crate::auth::get_google_oauth_client_id()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get Google OAuth client ID from vault: {}", e))?;
    let client_secret = crate::auth::get_google_oauth_client_secret()
        .await
        .map_err(|e| {
            anyhow::anyhow!("Failed to get Google OAuth client secret from vault: {}", e)
        })?;

    // Use real google-oauth library with vault credentials
    let oauth_response = GoogleLogin::client_id(client_id.as_str())
        .client_secret(client_secret.as_str())
        .add_scope("https://www.googleapis.com/auth/userinfo.email")?
        .add_scope("https://www.googleapis.com/auth/userinfo.profile")
        .port(33999)
        .timeout(std::time::Duration::from_secs(300))
        .login()
        .await
        .map_err(|e| anyhow::anyhow!("Google OAuth login failed: {}", e))?;

    log::info!("Successfully completed Google OAuth flow, fetching user info");

    // Fetch real user info using google-oauth library
    let user_info = GoogleUserInfo::token(&*oauth_response.access_token)
        .get_info()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch Google user info: {}", e))?;

    log::info!(
        "Successfully fetched Google user info for: {}",
        user_info.email
    );

    // Calculate token expiration from oauth_response
    let expires_at = Some(Utc::now() + Duration::seconds(oauth_response.expires_in as i64));

    // Create AuthState with real user data and tokens from google-oauth library
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
    let vault = crate::auth::CredentialVault::new().await?;
    let provider_name = "google";

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

    auth_state.save().await?;
    Ok(auth_state)
}

async fn login_github_native(_oauth_callbacks: OAuthCallbacks) -> Result<AuthState> {
    log::info!("Starting native GitHub OAuth login with embedded vault credentials");

    // Get GitHub OAuth credentials from embedded vault
    let client_id = crate::auth::get_github_oauth_client_id()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to get GitHub OAuth client ID from vault: {}", e))?;
    let client_secret = crate::auth::get_github_oauth_client_secret()
        .await
        .map_err(|e| {
            anyhow::anyhow!("Failed to get GitHub OAuth client secret from vault: {}", e)
        })?;

    // Use real github-oauth library with vault credentials
    let oauth_response = Login::client_id(client_id.as_str())
        .client_secret(client_secret.as_str())
        .scopes(["user:email", "read:user"])?
        .port(33999)
        .timeout(std::time::Duration::from_secs(300))
        .login()
        .await
        .map_err(|e| anyhow::anyhow!("GitHub OAuth login failed: {}", e))?;

    log::info!("Successfully completed GitHub OAuth flow, fetching user info");

    // Fetch real user info using github-oauth library
    let user_info = GitHubUserInfo::token(oauth_response.access_token.as_str())
        .get_info()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to fetch GitHub user info: {}", e))?;

    log::info!(
        "Successfully fetched GitHub user info for: {}",
        user_info.login
    );

    // GitHub tokens don't expire, so no expiration time
    // Handle optional email (GitHub users can make email private)
    let user_email = user_info
        .email
        .unwrap_or_else(|| format!("{}@users.noreply.github.com", user_info.login));
    let user_name = user_info.name.unwrap_or_else(|| user_info.login.clone());

    // Create AuthState with real user data
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
    let vault = crate::auth::CredentialVault::new().await?;
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

    auth_state.save().await?;
    Ok(auth_state)
}
