//! Authentication error types with detailed context for debugging

/// Authentication error types with detailed context for debugging
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AuthError {
    /// Login operation failed with detailed error message
    LoginFailed(String),
    /// Logout operation failed with detailed error message
    LogoutFailed(String),
    /// User stream setup failed with detailed error message
    StreamSetupFailed(String),
    /// Environment operation failed with detailed error message
    EnvironmentError(String),
    /// Repository operation failed with detailed error message
    RepositoryError(String),
    /// Model operation failed with detailed error message
    ModelError(String),
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuthError::LoginFailed(msg) => write!(f, "Login failed: {msg}"),
            AuthError::LogoutFailed(msg) => write!(f, "Logout failed: {msg}"),
            AuthError::StreamSetupFailed(msg) => write!(f, "Stream setup failed: {msg}"),
            AuthError::EnvironmentError(msg) => write!(f, "Environment error: {msg}"),
            AuthError::RepositoryError(msg) => write!(f, "Repository error: {msg}"),
            AuthError::ModelError(msg) => write!(f, "Model error: {msg}"),
        }
    }
}

impl std::error::Error for AuthError {}

/// Trait extension for Result<(), String> to handle Option unwrap safely
pub trait ResultExt<T> {
    fn unwrap_err_or_else<F: FnOnce() -> T>(self, f: F) -> T;
}

impl<T> ResultExt<T> for Result<(), T> {
    fn unwrap_err_or_else<F: FnOnce() -> T>(self, f: F) -> T {
        match self {
            Ok(_) => f(),
            Err(err) => err,
        }
    }
}
