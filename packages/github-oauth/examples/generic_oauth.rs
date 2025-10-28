use github_oauth::{
    GitHubProvider, Login, OAuthProvider, OAuthResponse, Result, UserInfo as UserInfoResponse,
    UserInfoBuilder,
};
use std::time::Duration;

/// Example of generic OAuth function that works with any OAuth provider
/// This same function could work with Google OAuth by just changing the type parameters
async fn authenticate_with_provider<P>() -> Result<(OAuthResponse, UserInfoResponse)>
where
    P: OAuthProvider,
{
    println!("Authenticating with {}", P::provider_name());

    // Use the provider's default scopes
    let default_scopes = P::default_scopes();
    println!("Using default scopes: {:?}", default_scopes);

    // Create login builder and configure it
    let oauth_response = Login::from_env()
        .scopes(default_scopes.iter().map(|s| s.to_string()))?  // PKCE auto-enabled
        .port(8080)
        .timeout(Duration::from_secs(300))
        .login()
        .await?;

    println!("Got access token: {}", oauth_response.access_token.as_str());

    // Get user info using the access token
    let user_info = UserInfoBuilder::token(oauth_response.access_token.as_str())
        .get_info()
        .await?;

    println!("User info: {:?}", user_info);

    Ok((oauth_response, user_info))
}

/// Example showing how cyrup-chat could use this generically
#[derive(Clone)]
pub enum SupportedProvider {
    GitHub,
    Google, // Would be available when Google OAuth crate is also included
}

impl SupportedProvider {
    pub fn name(&self) -> &'static str {
        match self {
            SupportedProvider::GitHub => "GitHub",
            SupportedProvider::Google => "Google",
        }
    }
}

/// Generic OAuth handler that cyrup-chat could use
pub struct OAuthHandler {
    provider: SupportedProvider,
}

impl OAuthHandler {
    pub fn new(provider: SupportedProvider) -> Self {
        Self { provider }
    }

    /// Authenticate with the configured provider
    pub async fn authenticate(&self) -> Result<String> {
        match self.provider {
            SupportedProvider::GitHub => {
                let (oauth_response, user_info) =
                    authenticate_with_provider::<GitHubProvider>().await?;
                Ok(format!(
                    "Authenticated {} as {}",
                    oauth_response.access_token.as_str(), user_info.login
                ))
            }
            SupportedProvider::Google => {
                // This would work with the Google OAuth crate:
                // let (oauth_response, user_info) = authenticate_with_provider::<GoogleProvider>().await?;
                // Ok(format!("Authenticated {} as {}", oauth_response.access_token, user_info.email))
                Err(github_oauth::OAuthError::Authorization(
                    "Google provider not implemented in this example".to_string(),
                ))
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Example usage for cyrup-chat

    // GitHub authentication
    let github_handler = OAuthHandler::new(SupportedProvider::GitHub);
    match github_handler.authenticate().await {
        Ok(result) => println!("GitHub auth successful: {}", result),
        Err(e) => println!("GitHub auth failed: {}", e),
    }

    // This demonstrates how the same pattern would work for Google
    let google_handler = OAuthHandler::new(SupportedProvider::Google);
    match google_handler.authenticate().await {
        Ok(result) => println!("Google auth successful: {}", result),
        Err(e) => println!("Google auth not available: {}", e),
    }

    Ok(())
}
