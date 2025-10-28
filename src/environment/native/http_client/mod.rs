//! Production-grade HTTP client with connection pooling and zero allocations
//!
//! This module provides a high-performance HTTP client implementation with:
//! - Connection pooling for optimal resource usage
//! - Automatic retry logic with exponential backoff
//! - Circuit breaker pattern for resilience
//! - Request/response logging for observability
//! - Zero-allocation optimizations where possible

pub mod circuit_breaker;
pub mod client;
pub mod errors;
pub mod methods;
pub mod resilience;
pub mod stats;
pub mod utils;
pub mod circuit_breaker;
pub mod url_extractor;

// Re-export main types for easy access
pub use client::HttpClient;
pub use errors::HttpClientError;
pub use stats::PoolStats;