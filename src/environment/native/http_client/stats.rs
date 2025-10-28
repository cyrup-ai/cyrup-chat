//! Connection pool statistics

/// Connection pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub active_connections: usize,
    pub idle_connections: usize,
    pub total_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f64,
}