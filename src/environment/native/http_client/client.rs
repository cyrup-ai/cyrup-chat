//! HTTP client core implementation

use super::{circuit_breaker::CircuitBreaker, errors::HttpClientError, stats::PoolStats};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::sync::Semaphore;

/// HTTP client with connection pooling and resilience patterns
#[derive(Clone)]
pub struct HttpClient {
    pub(super) client: reqwest::Client,
    pub(super) pool_semaphore: Arc<Semaphore>,
    pub(super) circuit_breakers: Arc<Mutex<HashMap<String, CircuitBreaker>>>,
    pub(super) stats: Arc<Mutex<PoolStats>>,
    pub(super) max_retries: u32,
    pub(super) base_delay_ms: u64,
    pub(super) max_delay_ms: u64,
}

impl HttpClient {
    /// Create a new HTTP client with connection pooling
    pub fn new(max_connections: usize) -> Result<Self, HttpClientError> {
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(max_connections / 4)
            .pool_idle_timeout(Duration::from_secs(30))
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10))
            .tcp_keepalive(Duration::from_secs(60))
            .http2_keep_alive_interval(Duration::from_secs(30))
            .http2_keep_alive_timeout(Duration::from_secs(10))
            .http2_keep_alive_while_idle(true)
            .build()
            .map_err(|e| HttpClientError::NetworkError(format!("Failed to create client: {e}")))?;

        let stats = PoolStats {
            active_connections: 0,
            idle_connections: 0,
            total_requests: 0,
            failed_requests: 0,
            average_response_time_ms: 0.0,
        };

        Ok(Self {
            client,
            pool_semaphore: Arc::new(Semaphore::new(max_connections)),
            circuit_breakers: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(stats)),
            max_retries: 3,
            base_delay_ms: 100,
            max_delay_ms: 5000,
        })
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new(100).unwrap_or_else(|e| {
            log::error!("Failed to create default HTTP client: {e}");
            panic!("Failed to create default HTTP client");
        })
    }
}