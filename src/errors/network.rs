// Network Error Types - Production Implementation

/// Network-related errors
#[derive(Debug, thiserror::Error)]
pub enum NetworkError {
    #[error("Request failed: {status_code}")]
    RequestFailed { status_code: u16 },

    #[error("Connection timeout")]
    Timeout,

    #[error("DNS resolution failed: {hostname}")]
    DnsResolutionFailed { hostname: String },

    #[error("TLS/SSL error: {reason}")]
    TlsError { reason: String },

    #[error("Invalid URL: {url}")]
    InvalidUrl { url: String },

    #[error("Serialization error: {reason}")]
    SerializationError { reason: String },

    #[error("Rate limit exceeded")]
    RateLimitExceeded,
}

impl NetworkError {
    /// Create a new request failed error
    #[inline]
    pub fn request_failed(status_code: u16) -> Self {
        Self::RequestFailed { status_code }
    }

    /// Create a new DNS resolution failed error
    #[inline]
    pub fn dns_resolution_failed(hostname: impl Into<String>) -> Self {
        Self::DnsResolutionFailed {
            hostname: hostname.into(),
        }
    }
}
