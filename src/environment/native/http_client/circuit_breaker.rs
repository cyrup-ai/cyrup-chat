//! Circuit breaker implementation with proper URL parsing and host extraction
//!
//! Zero-allocation URL parsing and efficient host string sharing using Arc<str>
//! for optimal circuit breaker key extraction and management.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use url::Url;
use crate::errors::ui::UiError;

/// Circuit breaker key extractor trait for host-based circuit breaking
pub trait CircuitBreakerKeyExtractor {
    /// Extract circuit breaker key from URL with zero allocation where possible
    fn extract_key(&self, url: &str) -> Result<Arc<str>, CircuitBreakerError>;
    
    /// Extract key from parsed URL for efficiency
    fn extract_key_from_parsed(&self, url: &Url) -> Arc<str>;
    
    /// Get cached key for common hosts
    fn get_cached_key(&self, host: &str) -> Option<Arc<str>>;
    
    /// Cache key for future use
    fn cache_key(&mut self, host: String, key: Arc<str>);
}

/// Circuit breaker error types
#[derive(Debug, thiserror::Error)]
pub enum CircuitBreakerError {
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Missing host in URL: {0}")]
    MissingHost(String),
    #[error("URL parsing failed: {0}")]
    ParseError(String),
    #[error("Circuit breaker state error: {0}")]
    StateError(String),
}

impl From<CircuitBreakerError> for UiError {
    fn from(error: CircuitBreakerError) -> Self {
        UiError::platform_error(&error.to_string())
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    /// Circuit is closed, requests are allowed
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, testing if service recovered
    HalfOpen,
}

/// Circuit breaker configuration
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold to open circuit
    pub failure_threshold: u32,
    /// Success threshold to close circuit from half-open
    pub success_threshold: u32,
    /// Timeout before transitioning from open to half-open
    pub timeout: Duration,
    /// Window size for failure counting
    pub window_size: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            success_threshold: 3,
            timeout: Duration::from_secs(60),
            window_size: Duration::from_secs(300),
        }
    }
}

/// Circuit breaker statistics
#[derive(Debug, Clone)]
pub struct CircuitBreakerStats {
    /// Current state
    pub state: CircuitBreakerState,
    /// Total failures in current window
    pub failures: u32,
    /// Total successes in current window
    pub successes: u32,
    /// Last failure time
    pub last_failure: Option<Instant>,
    /// Last success time
    pub last_success: Option<Instant>,
    /// State transition time
    pub state_changed_at: Instant,
}

/// Circuit breaker implementation
#[derive(Debug)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    stats: CircuitBreakerStats,
}

impl CircuitBreaker {
    /// Create new circuit breaker with configuration
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            stats: CircuitBreakerStats {
                state: CircuitBreakerState::Closed,
                failures: 0,
                successes: 0,
                last_failure: None,
                last_success: None,
                state_changed_at: Instant::now(),
            },
        }
    }
    
    /// Check if request should be allowed
    pub fn should_allow_request(&mut self) -> bool {
        self.update_state();
        
        match self.stats.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::Open => false,
            CircuitBreakerState::HalfOpen => {
                // Allow limited requests in half-open state
                self.stats.successes < self.config.success_threshold
            }
        }
    }
    
    /// Record successful request
    pub fn record_success(&mut self) {
        self.stats.successes += 1;
        self.stats.last_success = Some(Instant::now());
        
        // Transition to closed if enough successes in half-open state
        if self.stats.state == CircuitBreakerState::HalfOpen 
            && self.stats.successes >= self.config.success_threshold {
            self.transition_to_closed();
        }
    }
    
    /// Record failed request
    pub fn record_failure(&mut self) {
        self.stats.failures += 1;
        self.stats.last_failure = Some(Instant::now());
        
        // Transition to open if too many failures
        if self.stats.state == CircuitBreakerState::Closed 
            && self.stats.failures >= self.config.failure_threshold {
            self.transition_to_open();
        } else if self.stats.state == CircuitBreakerState::HalfOpen {
            // Reset to open if failure in half-open state
            self.transition_to_open();
        }
    }
    
    /// Get current statistics
    pub fn stats(&self) -> &CircuitBreakerStats {
        &self.stats
    }
    
    /// Update circuit breaker state based on timeouts
    fn update_state(&mut self) {
        let now = Instant::now();
        
        match self.stats.state {
            CircuitBreakerState::Open => {
                // Check if timeout elapsed to transition to half-open
                if now.duration_since(self.stats.state_changed_at) >= self.config.timeout {
                    self.transition_to_half_open();
                }
            }
            CircuitBreakerState::Closed | CircuitBreakerState::HalfOpen => {
                // Reset counters if window expired
                if now.duration_since(self.stats.state_changed_at) >= self.config.window_size {
                    self.reset_counters();
                }
            }
        }
    }
    
    /// Transition to closed state
    fn transition_to_closed(&mut self) {
        self.stats.state = CircuitBreakerState::Closed;
        self.stats.state_changed_at = Instant::now();
        self.reset_counters();
    }
    
    /// Transition to open state
    fn transition_to_open(&mut self) {
        self.stats.state = CircuitBreakerState::Open;
        self.stats.state_changed_at = Instant::now();
    }
    
    /// Transition to half-open state
    fn transition_to_half_open(&mut self) {
        self.stats.state = CircuitBreakerState::HalfOpen;
        self.stats.state_changed_at = Instant::now();
        self.stats.successes = 0; // Reset success counter for half-open testing
    }
    
    /// Reset failure and success counters
    fn reset_counters(&mut self) {
        self.stats.failures = 0;
        self.stats.successes = 0;
    }
}

/// URL-based circuit breaker key extractor with host caching
pub struct UrlCircuitBreakerKeyExtractor {
    /// Cache for common hosts to avoid repeated allocations
    host_cache: HashMap<String, Arc<str>>,
    /// Configuration for key extraction
    include_port: bool,
}

impl UrlCircuitBreakerKeyExtractor {
    /// Create new URL-based key extractor
    pub fn new(include_port: bool) -> Self {
        Self {
            host_cache: HashMap::new(),
            include_port,
        }
    }
    
    /// Create with common host pre-caching
    pub fn with_common_hosts(include_port: bool, common_hosts: Vec<String>) -> Self {
        let mut extractor = Self::new(include_port);
        
        // Pre-cache common hosts
        for host in common_hosts {
            let key: Arc<str> = if include_port {
                host.clone().into()
            } else {
                // Remove port if present
                host.split(':').next().unwrap_or(&host).to_string().into()
            };
            extractor.host_cache.insert(host, key);
        }
        
        extractor
    }
    
    /// Parse URL with zero allocation where possible
    fn parse_url(&self, url_str: &str) -> Result<Url, CircuitBreakerError> {
        Url::parse(url_str)
            .map_err(|e| CircuitBreakerError::ParseError(e.to_string()))
    }
    
    /// Extract host from URL with port handling
    fn extract_host_from_url(&self, url: &Url) -> Result<String, CircuitBreakerError> {
        let host = url.host_str()
            .ok_or_else(|| CircuitBreakerError::MissingHost(url.to_string()))?;
        
        if self.include_port {
            match url.port() {
                Some(port) => Ok(format!("{}:{}", host, port)),
                None => Ok(host.to_string()),
            }
        } else {
            Ok(host.to_string())
        }
    }
}

impl CircuitBreakerKeyExtractor for UrlCircuitBreakerKeyExtractor {
    fn extract_key(&self, url: &str) -> Result<Arc<str>, CircuitBreakerError> {
        // Try to parse URL
        let parsed_url = self.parse_url(url)?;
        Ok(self.extract_key_from_parsed(&parsed_url))
    }
    
    fn extract_key_from_parsed(&self, url: &Url) -> Arc<str> {
        let host = match self.extract_host_from_url(url) {
            Ok(host) => host,
            Err(_) => {
                // Fallback to URL string if host extraction fails
                return url.to_string().into();
            }
        };
        
        // Check cache first
        if let Some(cached_key) = self.host_cache.get(&host) {
            return cached_key.clone();
        }
        
        // Create new key
        host.into()
    }
    
    fn get_cached_key(&self, host: &str) -> Option<Arc<str>> {
        self.host_cache.get(host).cloned()
    }
    
    fn cache_key(&mut self, host: String, key: Arc<str>) {
        self.host_cache.insert(host, key);
    }
}

/// Circuit breaker manager for multiple hosts
pub struct CircuitBreakerManager {
    /// Circuit breakers per host
    breakers: Mutex<HashMap<Arc<str>, CircuitBreaker>>,
    /// Key extractor for URL parsing
    key_extractor: Mutex<UrlCircuitBreakerKeyExtractor>,
    /// Default configuration for new breakers
    default_config: CircuitBreakerConfig,
}

impl CircuitBreakerManager {
    /// Create new circuit breaker manager
    pub fn new(config: CircuitBreakerConfig, include_port: bool) -> Self {
        Self {
            breakers: Mutex::new(HashMap::new()),
            key_extractor: Mutex::new(UrlCircuitBreakerKeyExtractor::new(include_port)),
            default_config: config,
        }
    }
    
    /// Create with common hosts pre-cached
    pub fn with_common_hosts(
        config: CircuitBreakerConfig,
        include_port: bool,
        common_hosts: Vec<String>,
    ) -> Self {
        Self {
            breakers: Mutex::new(HashMap::new()),
            key_extractor: Mutex::new(UrlCircuitBreakerKeyExtractor::with_common_hosts(include_port, common_hosts)),
            default_config: config,
        }
    }
    
    /// Extract circuit breaker key from URL
    pub fn extract_circuit_breaker_key(&self, url: &str) -> Result<Arc<str>, CircuitBreakerError> {
        let extractor = self.key_extractor.lock()
            .map_err(|e| CircuitBreakerError::StateError(e.to_string()))?;
        extractor.extract_key(url)
    }
    
    /// Check if request should be allowed
    pub fn should_allow_request(&self, key: &Arc<str>) -> Result<bool, CircuitBreakerError> {
        let mut breakers = self.breakers.lock()
            .map_err(|e| CircuitBreakerError::StateError(e.to_string()))?;
        
        let breaker = breakers.entry(key.clone())
            .or_insert_with(|| CircuitBreaker::new(self.default_config.clone()));
        
        Ok(breaker.should_allow_request())
    }
    
    /// Record successful request
    pub fn record_success(&self, key: &Arc<str>) -> Result<(), CircuitBreakerError> {
        let mut breakers = self.breakers.lock()
            .map_err(|e| CircuitBreakerError::StateError(e.to_string()))?;
        
        if let Some(breaker) = breakers.get_mut(key) {
            breaker.record_success();
        }
        
        Ok(())
    }
    
    /// Record failed request
    pub fn record_failure(&self, key: &Arc<str>) -> Result<(), CircuitBreakerError> {
        let mut breakers = self.breakers.lock()
            .map_err(|e| CircuitBreakerError::StateError(e.to_string()))?;
        
        let breaker = breakers.entry(key.clone())
            .or_insert_with(|| CircuitBreaker::new(self.default_config.clone()));
        
        breaker.record_failure();
        Ok(())
    }
    
    /// Get circuit breaker statistics for a host
    pub fn get_stats(&self, key: &Arc<str>) -> Result<Option<CircuitBreakerStats>, CircuitBreakerError> {
        let breakers = self.breakers.lock()
            .map_err(|e| CircuitBreakerError::StateError(e.to_string()))?;
        
        Ok(breakers.get(key).map(|breaker| breaker.stats().clone()))
    }
    
    /// Get all circuit breaker statistics
    pub fn get_all_stats(&self) -> Result<HashMap<Arc<str>, CircuitBreakerStats>, CircuitBreakerError> {
        let breakers = self.breakers.lock()
            .map_err(|e| CircuitBreakerError::StateError(e.to_string()))?;
        
        Ok(breakers.iter()
            .map(|(key, breaker)| (key.clone(), breaker.stats().clone()))
            .collect())
    }
    
    /// Clear all circuit breakers (for testing/reset)
    pub fn clear_all(&self) -> Result<(), CircuitBreakerError> {
        let mut breakers = self.breakers.lock()
            .map_err(|e| CircuitBreakerError::StateError(e.to_string()))?;
        breakers.clear();
        Ok(())
    }
}

/// Utility functions for circuit breaker operations
pub mod utils {
    use super::*;
    
    /// Create default circuit breaker manager with common web hosts
    pub fn create_default_manager() -> CircuitBreakerManager {
        let common_hosts = vec![
            "api.example.com".to_string(),
            "localhost:3000".to_string(),
            "127.0.0.1:8080".to_string(),
        ];
        
        CircuitBreakerManager::with_common_hosts(
            CircuitBreakerConfig::default(),
            true, // include port
            common_hosts,
        )
    }
    
    /// Extract host from URL string with fallback
    pub fn extract_host_with_fallback(url: &str) -> Arc<str> {
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
    
    /// Check if URL is valid for circuit breaker key extraction
    pub fn is_valid_url(url: &str) -> bool {
        Url::parse(url).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_circuit_breaker_states() {
        let mut breaker = CircuitBreaker::new(CircuitBreakerConfig {
            failure_threshold: 2,
            success_threshold: 1,
            timeout: Duration::from_millis(100),
            window_size: Duration::from_secs(1),
        });
        
        // Initially closed
        assert_eq!(breaker.stats().state, CircuitBreakerState::Closed);
        assert!(breaker.should_allow_request());
        
        // Record failures to open circuit
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.stats().state, CircuitBreakerState::Open);
        assert!(!breaker.should_allow_request());
    }
    
    #[test]
    fn test_url_key_extraction() {
        let extractor = UrlCircuitBreakerKeyExtractor::new(true);
        
        let key = extractor.extract_key("https://api.example.com:8080/path").unwrap();
        assert_eq!(key.as_ref(), "api.example.com:8080");
        
        let key_no_port = extractor.extract_key("https://api.example.com/path").unwrap();
        assert_eq!(key_no_port.as_ref(), "api.example.com");
    }
    
    #[test]
    fn test_circuit_breaker_manager() {
        let manager = CircuitBreakerManager::new(CircuitBreakerConfig::default(), true);
        
        let key = manager.extract_circuit_breaker_key("https://api.example.com/test").unwrap();
        assert_eq!(key.as_ref(), "api.example.com");
        
        assert!(manager.should_allow_request(&key).unwrap());
        
        // Record success
        manager.record_success(&key).unwrap();
        
        // Should still allow requests
        assert!(manager.should_allow_request(&key).unwrap());
    }
}