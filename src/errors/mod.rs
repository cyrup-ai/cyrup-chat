// Error Types - Production Implementation
// Comprehensive error handling with zero allocation where possible

pub mod auth;
pub mod network;
pub mod ui;

// Re-export common error types
pub use auth::AuthError;
pub use network::NetworkError;
pub use ui::UiError;

/// Result type alias for common operations
pub type AppResult<T> = Result<T, AppError>;

/// Main application error enum
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Authentication error: {0}")]
    Auth(#[from] AuthError),

    #[error("UI error: {0}")]
    Ui(#[from] UiError),

    #[error("Network error: {0}")]
    Network(#[from] NetworkError),

    #[error("Configuration error: {message}")]
    Config { message: String },

    #[error("Internal error: {message}")]
    Internal { message: String },
}
