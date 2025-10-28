use thiserror::Error;

#[derive(Error, Debug)]
pub enum OAuthError {
    #[error("Missing client ID")]
    MissingClientId,

    #[error("Missing client secret")]
    MissingClientSecret,

    #[error("Missing refresh token")]
    MissingRefreshToken,

    #[error("Missing access token")]
    MissingAccessToken,

    #[error("Invalid redirect URI: {0}")]
    InvalidRedirectUri(String),

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("URL parse error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Token exchange failed: {0}")]
    TokenExchange(String),

    #[error("Environment variable not found: {0}")]
    EnvVar(String),

    #[error("Timeout waiting for authorization")]
    Timeout,

    #[error("Invalid state parameter")]
    InvalidState,

    #[error("Authorization denied")]
    AuthorizationDenied,

    #[error("Server error: {0}")]
    Server(String),

    #[error("PKCE generation failed: {0}")]
    PkceGenerationFailed(String),

    #[error("Invalid code challenge: {0}")]
    InvalidCodeChallenge(String),

    #[error("PKCE verification failed: {0}")]
    PkceVerificationFailed(String),
}
