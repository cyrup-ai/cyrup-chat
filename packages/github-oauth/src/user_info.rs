use crate::{
    error::OAuthError, future::WrappedFuture, traits::OAuthUserInfo,
    types::UserInfo as UserInfoResponse, Result,
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
            let response = client
                .get("https://api.github.com/user")
                .header("Authorization", format!("Bearer {}", self.access_token.as_str()))
                .header("User-Agent", "github-oauth-rust")
                .send()
                .await?;

            if !response.status().is_success() {
                if response.status() == 401 {
                    return Err(OAuthError::Authorization(
                        "Invalid or expired access token".to_string(),
                    ));
                }
                let error_text = response.text().await?;
                return Err(OAuthError::Authorization(error_text));
            }

            let user_info: UserInfoResponse = response.json().await?;
            Ok(user_info)
        })
    }
}

// Trait implementations for generic OAuth usage
impl OAuthUserInfo for UserInfoExecuteBuilder {
    fn with_token(token: impl Into<String>) -> Self {
        UserInfoExecuteBuilder {
            access_token: Zeroizing::new(token.into()),
        }
    }

    fn execute(self) -> WrappedFuture<Result<UserInfoResponse>> {
        self.get_info()
    }
}
