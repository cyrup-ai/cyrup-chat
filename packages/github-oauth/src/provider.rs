use crate::traits::OAuthProvider;

/// GitHub OAuth provider implementation
pub struct GitHubProvider;

impl OAuthProvider for GitHubProvider {
    fn provider_name() -> &'static str {
        "GitHub"
    }

    fn default_scopes() -> Vec<&'static str> {
        vec!["user:email"]
    }

    fn auth_endpoint() -> &'static str {
        "https://github.com/login/oauth/authorize"
    }

    fn token_endpoint() -> &'static str {
        "https://github.com/login/oauth/access_token"
    }

    fn user_info_endpoint() -> &'static str {
        "https://api.github.com/user"
    }
}
