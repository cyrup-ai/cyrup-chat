//! HTTP client error types

/// HTTP client error types
#[derive(Debug)]
pub enum HttpClientError {
    /// Connection pool exhausted
    PoolExhausted,
    /// Request timeout
    Timeout,
    /// Circuit breaker is open
    CircuitBreakerOpen,
    /// Network error
    NetworkError(String),
    /// Invalid request
    InvalidRequest(String),
    /// Server error (5xx)
    ServerError(u16, String),
    /// Client error (4xx)
    ClientError(u16, String),
}

impl std::fmt::Display for HttpClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PoolExhausted => write!(f, "Connection pool exhausted"),
            Self::Timeout => write!(f, "Request timeout"),
            Self::CircuitBreakerOpen => write!(f, "Circuit breaker is open"),
            Self::NetworkError(msg) => write!(f, "Network error: {msg}"),
            Self::InvalidRequest(msg) => write!(f, "Invalid request: {msg}"),
            Self::ServerError(code, msg) => write!(f, "Server error {code}: {msg}"),
            Self::ClientError(code, msg) => write!(f, "Client error {code}: {msg}"),
        }
    }
}

impl std::error::Error for HttpClientError {}