//! URL extraction utilities for circuit breaker integration
//!
//! Provides utilities to extract URLs from HTTP requests for proper
//! circuit breaker key generation with zero-allocation patterns.

use std::sync::Arc;
use url::Url;
use super::circuit_breaker::{CircuitBreakerManager, CircuitBreakerError};

/// URL extractor for HTTP client integration
pub struct HttpUrlExtractor {
    circuit_breaker_manager: Arc<CircuitBreakerManager>,
}

impl HttpUrlExtractor {
    /// Create new URL extractor with circuit breaker manager
    pub fn new(circuit_breaker_manager: Arc<CircuitBreakerManager>) -> Self {
        Self {
            circuit_breaker_manager,
        }
    }
    
    /// Extract circuit breaker key from URL string
    pub fn extract_circuit_breaker_key(&self, url: &str) -> Result<Arc<str>, CircuitBreakerError> {
        self.circuit_breaker_manager.extract_circuit_breaker_key(url)
    }
    
    /// Check if request should be allowed based on circuit breaker state
    pub fn should_allow_request(&self, url: &str) -> Result<bool, CircuitBreakerError> {
        let key = self.extract_circuit_breaker_key(url)?;
        self.circuit_breaker_manager.should_allow_request(&key)
    }
    
    /// Record successful request for circuit breaker
    pub fn record_success(&self, url: &str) -> Result<(), CircuitBreakerError> {
        let key = self.extract_circuit_breaker_key(url)?;
        self.circuit_breaker_manager.record_success(&key)
    }
    
    /// Record failed request for circuit breaker
    pub fn record_failure(&self, url: &str) -> Result<(), CircuitBreakerError> {
        let key = self.extract_circuit_breaker_key(url)?;
        self.circuit_breaker_manager.record_failure(&key)
    }
    
    /// Validate URL format
    pub fn is_valid_url(&self, url: &str) -> bool {
        Url::parse(url).is_ok()
    }
    
    /// Extract host from URL with fallback
    pub fn extract_host(&self, url: &str) -> Arc<str> {
        match Url::parse(url) {
            Ok(parsed) => {
                if let Some(host) = parsed.host_str() {
                    match parsed.port() {
                        Some(port) => format!("{}:{}", host, port).into(),
                        None => host.into(),
                    }
                } else {
                    "unknown_host".into()
                }
            }
            Err(_) => "invalid_url".into(),
        }
    }
}

/// HTTP request context for circuit breaker integration
#[derive(Debug, Clone)]
pub struct HttpRequestContext {
    /// Original URL string
    pub url: String,
    /// Circuit breaker key
    pub circuit_breaker_key: Arc<str>,
    /// Request method
    pub method: String,
    /// Request timestamp
    pub timestamp: std::time::Instant,
}

impl HttpRequestContext {
    /// Create new request context
    pub fn new(url: String, method: String, extractor: &HttpUrlExtractor) -> Result<Self, CircuitBreakerError> {
        let circuit_breaker_key = extractor.extract_circuit_breaker_key(&url)?;
        
        Ok(Self {
            url,
            circuit_breaker_key,
            method,
            timestamp: std::time::Instant::now(),
        })
    }
    
    /// Get elapsed time since request creation
    pub fn elapsed(&self) -> std::time::Duration {
        self.timestamp.elapsed()
    }
    
    /// Check if request should be allowed
    pub fn should_allow(&self, extractor: &HttpUrlExtractor) -> Result<bool, CircuitBreakerError> {
        extractor.circuit_breaker_manager.should_allow_request(&self.circuit_breaker_key)
    }
    
    /// Record success for this request
    pub fn record_success(&self, extractor: &HttpUrlExtractor) -> Result<(), CircuitBreakerError> {
        extractor.circuit_breaker_manager.record_success(&self.circuit_breaker_key)
    }
    
    /// Record failure for this request
    pub fn record_failure(&self, extractor: &HttpUrlExtractor) -> Result<(), CircuitBreakerError> {
        extractor.circuit_breaker_manager.record_failure(&self.circuit_breaker_key)
    }
}

/// Factory for creating HTTP URL extractors
pub struct HttpUrlExtractorFactory;

impl HttpUrlExtractorFactory {
    /// Create default URL extractor with circuit breaker
    pub fn create_default() -> HttpUrlExtractor {
        let circuit_breaker_manager = Arc::new(
            super::circuit_breaker::utils::create_default_manager()
        );
        HttpUrlExtractor::new(circuit_breaker_manager)
    }
    
    /// Create URL extractor with custom circuit breaker manager
    pub fn create_with_manager(manager: Arc<CircuitBreakerManager>) -> HttpUrlExtractor {
        HttpUrlExtractor::new(manager)
    }
    
    /// Create URL extractor with common hosts pre-cached
    pub fn create_with_common_hosts(common_hosts: Vec<String>) -> HttpUrlExtractor {
        use super::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager};
        
        let circuit_breaker_manager = Arc::new(
            CircuitBreakerManager::with_common_hosts(
                CircuitBreakerConfig::default(),
                true, // include port
                common_hosts,
            )
        );
        HttpUrlExtractor::new(circuit_breaker_manager)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_url_extractor() {
        let extractor = HttpUrlExtractorFactory::create_default();
        
        let url = "https://api.example.com/test";
        assert!(extractor.is_valid_url(url));
        
        let key = extractor.extract_circuit_breaker_key(url).unwrap();
        assert_eq!(key.as_ref(), "api.example.com");
        
        assert!(extractor.should_allow_request(url).unwrap());
    }
    
    #[test]
    fn test_request_context() {
        let extractor = HttpUrlExtractorFactory::create_default();
        let url = "https://api.example.com/test".to_string();
        let method = "GET".to_string();
        
        let context = HttpRequestContext::new(url.clone(), method, &extractor).unwrap();
        assert_eq!(context.url, url);
        assert_eq!(context.circuit_breaker_key.as_ref(), "api.example.com");
        
        assert!(context.should_allow(&extractor).unwrap());
    }
    
    #[test]
    fn test_host_extraction() {
        let extractor = HttpUrlExtractorFactory::create_default();
        
        let host = extractor.extract_host("https://api.example.com:8080/path");
        assert_eq!(host.as_ref(), "api.example.com:8080");
        
        let host_no_port = extractor.extract_host("https://api.example.com/path");
        assert_eq!(host_no_port.as_ref(), "api.example.com");
        
        let invalid_host = extractor.extract_host("invalid-url");
        assert_eq!(invalid_host.as_ref(), "invalid_url");
    }
}