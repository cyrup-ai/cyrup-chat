//! Utility and statistics methods

use super::{client::HttpClient, errors::HttpClientError, stats::PoolStats};
use std::time::Duration;

impl HttpClient {
    /// Record circuit breaker success
    pub(super) fn record_circuit_breaker_success(&self, host: &str) {
        if let Ok(mut breakers) = self.circuit_breakers.lock() {
            if let Some(breaker) = breakers.get_mut(host) {
                breaker.record_success();
            }
        }
    }

    /// Record circuit breaker failure
    pub(super) fn record_circuit_breaker_failure(&self, host: &str) {
        if let Ok(mut breakers) = self.circuit_breakers.lock() {
            if let Some(breaker) = breakers.get_mut(host) {
                breaker.record_failure();
            }
        }
    }

    /// Update connection pool statistics
    pub(super) fn update_stats(&self, elapsed: Duration, success: bool) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_requests += 1;
            if !success {
                stats.failed_requests += 1;
            }
            
            // Update rolling average response time
            let elapsed_ms = elapsed.as_millis() as f64;
            stats.average_response_time_ms = 
                (stats.average_response_time_ms * (stats.total_requests - 1) as f64 + elapsed_ms) 
                / stats.total_requests as f64;
        }
    }

    /// Get current pool statistics
    pub fn get_stats(&self) -> PoolStats {
        self.stats.lock()
            .map(|stats| stats.clone())
            .unwrap_or_else(|_| PoolStats {
                active_connections: 0,
                idle_connections: 0,
                total_requests: 0,
                failed_requests: 0,
                average_response_time_ms: 0.0,
            })
    }

    /// Get circuit breaker status for a host
    pub fn get_circuit_breaker_status(&self, host: &str) -> Option<String> {
        self.circuit_breakers.lock().ok()
            .and_then(|breakers| breakers.get(host).map(|b| format!("{:?}", b.state)))
    }
}