use crate::{
    error::OAuthError, future::WrappedFuture, types::UserInfo as UserInfoResponse, Result,
};
use zeroize::Zeroizing;

pub struct UserInfo;

impl UserInfo {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self
    }

    pub fn token(access_token: impl Into<String>) -> UserInfoExecuteBuilder {
        UserInfoExecuteBuilder {
            access_token: Zeroizing::new(access_token.into()),
        }
    }
}

pub struct UserInfoExecuteBuilder {
    access_token: Zeroizing<String>,
}

impl UserInfoExecuteBuilder {
    pub fn get_info(self) -> WrappedFuture<Result<UserInfoResponse>> {
        WrappedFuture::new(async move {
            let client = reqwest::Client::new();
            let url = "https://www.googleapis.com/oauth2/v2/userinfo";

            let response = client
                .get(url)
                .header("Authorization", format!("Bearer {}", &*self.access_token))
                .send()
                .await?;

            if !response.status().is_success() {
                if response.status() == 401 {
                    return Err(OAuthError::Authorization(
                        "Invalid or expired access token".to_string(),
                    ));
                }
                let status_code = response.status().as_u16();
                let error_text = response.text().await?;
                // Security: Sanitize error messages to prevent information disclosure
                let sanitized_error = sanitize_api_error(&error_text, status_code);
                return Err(OAuthError::Authorization(sanitized_error));
            }

            let user_info: UserInfoResponse = response.json().await?;
            Ok(user_info)
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
                        desc if desc.contains("invalid_token") => {
                            "Invalid or expired access token".to_string()
                        }
                        desc if desc.contains("insufficient_scope") => {
                            "Insufficient permissions for requested resource".to_string()
                        }
                        desc if desc.contains("invalid_request") => {
                            "Invalid request parameters".to_string()
                        }
                        _ => "User info request failed".to_string(),
                    };
                }
            }
            if let Some(error_type) = error_obj.as_str() {
                return match error_type {
                    "invalid_token" => "Invalid or expired access token".to_string(),
                    "insufficient_scope" => "Insufficient permissions for requested resource".to_string(),
                    "invalid_request" => "Invalid request parameters".to_string(),
                    _ => "User info request failed".to_string(),
                };
            }
        }
    }

    // Fallback based on HTTP status code
    match status_code {
        400 => "Bad request - invalid parameters".to_string(),
        401 => "Unauthorized - invalid or expired access token".to_string(),
        403 => "Forbidden - insufficient permissions".to_string(),
        429 => "Rate limit exceeded - please try again later".to_string(),
        500..=599 => "Google API temporarily unavailable".to_string(),
        _ => "User info request failed".to_string(),
    }
}
