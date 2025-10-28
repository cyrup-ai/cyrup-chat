// Authentication Services Module

use crate::environment::native::model::types::TokenData;
use serde::{Deserialize, Serialize};

/// OAuth token refresh response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenRefreshResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub scope: Option<String>,
}

/// Refresh OAuth access token using refresh token
pub async fn refresh_access_token(
    instance_url: &str,
    refresh_token: &str,
    client_id: &str,
    client_secret: &str,
) -> Result<TokenData, Box<dyn std::error::Error + Send + Sync>> {
    let client = reqwest::Client::new();

    let token_url = format!("{}/oauth/token", instance_url.trim_end_matches('/'));

    let params = [
        ("grant_type", "refresh_token"),
        ("refresh_token", refresh_token),
        ("client_id", client_id),
        ("client_secret", client_secret),
    ];

    let response = client
        .post(&token_url)
        .form(&params)
        .header("Accept", "application/json")
        .header("Content-Type", "application/x-www-form-urlencoded")
        .send()
        .await
        .map_err(|e| format!("Token refresh request failed: {}", e))?;

    if !response.status().is_success() {
        let status = response.status();
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        return Err(format!(
            "Token refresh failed with status {}: {}",
            status, error_text
        )
        .into());
    }

    let refresh_response: TokenRefreshResponse = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse token refresh response: {}", e))?;

    let current_time = chrono::Utc::now().timestamp() as u64;
    let expires_in = refresh_response.expires_in.map(|exp| current_time + exp);

    // Create megalodon TokenData and wrap it
    let megalodon_token = megalodon::oauth::TokenData {
        access_token: refresh_response.access_token,
        token_type: refresh_response
            .token_type
            .unwrap_or_else(|| "Bearer".to_string()),
        expires_in,
        refresh_token: refresh_response.refresh_token,
        scope: refresh_response.scope,
        created_at: Some(current_time),
    };

    Ok(TokenData::from(megalodon_token))
}
