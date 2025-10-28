// Authentication Error Types - Production Implementation

/// Authentication-related errors
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid credentials provided")]
    InvalidCredentials,

    #[error("OAuth2 flow failed: {reason}")]
    OAuth2Failed { reason: String },

    #[error("Token validation failed: {reason}")]
    TokenValidationFailed { reason: String },

    #[error("Session expired")]
    SessionExpired,

    #[error("Authentication service unavailable")]
    ServiceUnavailable,

    #[error("Network error during authentication: {0}")]
    NetworkError(String),

    #[error("Configuration error: {message}")]
    ConfigError { message: String },

    #[error("Keychain access failed: {reason}")]
    KeychainError { reason: String },
}

impl AuthError {
    /// Create a new OAuth2 failed error
    #[inline]
    pub fn oauth2_failed(reason: impl Into<String>) -> Self {
        Self::OAuth2Failed {
            reason: reason.into(),
        }
    }

    /// Create a new token validation failed error
    #[inline]
    pub fn token_validation_failed(reason: impl Into<String>) -> Self {
        Self::TokenValidationFailed {
            reason: reason.into(),
        }
    }

    /// Create a new config error
    #[inline]
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError {
            message: message.into(),
        }
    }

    /// Create a new keychain error
    #[inline]
    pub fn keychain_error(reason: impl Into<String>) -> Self {
        Self::KeychainError {
            reason: reason.into(),
        }
    }
}
