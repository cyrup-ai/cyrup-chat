//! Request execution with resilience patterns

use super::{client::HttpClient, errors::HttpClientError};
use std::time::{Duration, Instant};

impl HttpClient {
    /// Execute request with full resilience patterns
    pub(super) async fn execute_with_resilience<F>(&self, url: &str, request_builder: F) -> Result<reqwest::Response, HttpClientError>
    where
        F: Fn() -> reqwest::RequestBuilder + Clone,
    {
        let start_time = Instant::now();
        
        // Acquire connection from pool
        let _permit = self.pool_semaphore
            .acquire()
            .await
            .map_err(|_| HttpClientError::PoolExhausted)?;

        // Extract host for circuit breaker
        let host = super::circuit_breaker::utils::extract_host_with_fallback(url).to_string();
        
        // Check circuit breaker
        {
            let mut breakers = self.circuit_breakers.lock().unwrap_or_else(|e| {
                log::error!("Circuit breaker lock poisoned: {e}");
                panic!("Circuit breaker lock poisoned");
            });
            let breaker = breakers.entry(host.clone()).or_insert_with(|| {
                super::circuit_breaker::CircuitBreaker::new(5, Duration::from_secs(60))
            });
            
            if !breaker.can_execute() {
                return Err(HttpClientError::CircuitBreakerOpen);
            }
        }

        // Execute with retry logic
        let mut last_error = None;
        for attempt in 0..=self.max_retries {
            let request = request_builder().build()
                .map_err(|e| HttpClientError::InvalidRequest(e.to_string()))?;
                
            log::debug!("HTTP request attempt {}: {} {}", 
                       attempt + 1, request.method(), request.url());

            match self.client.execute(request).await {
                Ok(response) => {
                    let status = response.status();
                    let elapsed = start_time.elapsed();
                    
                    // Update statistics
                    self.update_stats(elapsed, status.is_success());
                    
                    // Update circuit breaker
                    if status.is_success() || status.is_client_error() {
                        self.record_circuit_breaker_success(&host);
                        
                        if status.is_client_error() {
                            return Err(HttpClientError::ClientError(
                                status.as_u16(),
                                format!("Client error: {status}")
                            ));
                        }
                        
                        log::debug!("HTTP request successful: {} in {:?}", status, elapsed);
                        return Ok(response);
                    } else if status.is_server_error() {
                        self.record_circuit_breaker_failure(&host);
                        last_error = Some(HttpClientError::ServerError(
                            status.as_u16(),
                            format!("Server error: {status}")
                        ));
                    }
                }
                Err(e) => {
                    self.record_circuit_breaker_failure(&host);
                    last_error = Some(HttpClientError::NetworkError(e.to_string()));
                    log::warn!("HTTP request failed (attempt {}): {e}", attempt + 1);
                }
            }

            // Exponential backoff delay
            if attempt < self.max_retries {
                let delay_ms = std::cmp::min(
                    self.base_delay_ms * 2_u64.pow(attempt),
                    self.max_delay_ms
                );
                log::debug!("Retrying in {}ms", delay_ms);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            }
        }

        // All retries exhausted
        self.update_stats(start_time.elapsed(), false);
        Err(last_error.unwrap_or(HttpClientError::NetworkError("Unknown error".to_string())))
    }
}