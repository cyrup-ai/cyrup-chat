//! Status mutation types and data structures
//!
//! This module defines core types for status mutation operations with
//! zero-allocation patterns and comprehensive error handling.

use crate::StatusMutation;
use crate::view_model::{StatusId, StatusViewModel};
use std::collections::VecDeque;

/// Status mutation error types with detailed context for debugging
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusMutationError {
    /// Status mutation failed with detailed error message
    MutationFailed(String),
    /// Status not found for mutation
    StatusNotFound(String),
    /// Invalid mutation parameters
    InvalidMutation(String),
    /// Network operation failed
    NetworkError(String),
    /// Batch mutation operation failed
    #[allow(dead_code)] // Architectural component - batch processing implementation pending
    BatchMutationFailed(String),
    /// Environment error (platform, storage, or system error)
    #[allow(dead_code)] // Architectural component - environment error handling pending
    EnvironmentError(String),
}

impl StatusMutationError {
    /// Create a mutation failure error with enhanced context
    #[inline(always)]
    pub fn mutation_failed(msg: impl Into<String>) -> Self {
        Self::MutationFailed(msg.into())
    }

    /// Create a status not found error with enhanced context
    #[inline(always)]
    pub fn status_not_found(msg: impl Into<String>) -> Self {
        Self::StatusNotFound(msg.into())
    }

    /// Create an invalid mutation error with enhanced context
    #[inline(always)]
    pub fn invalid_mutation(msg: impl Into<String>) -> Self {
        Self::InvalidMutation(msg.into())
    }

    /// Create a network error with enhanced context
    #[inline(always)]
    pub fn network_error(msg: impl Into<String>) -> Self {
        Self::NetworkError(msg.into())
    }

    /// Create a batch mutation failure error with enhanced context
    #[allow(dead_code)] // Architectural component - batch processing implementation pending
    #[inline(always)]
    pub fn batch_mutation_failed(msg: impl Into<String>) -> Self {
        Self::BatchMutationFailed(msg.into())
    }

    /// Create an environment error with enhanced context  
    #[allow(dead_code)] // Architectural component - environment error handling pending
    #[inline(always)]
    pub fn environment_error(msg: impl Into<String>) -> Self {
        Self::EnvironmentError(msg.into())
    }
}

impl std::fmt::Display for StatusMutationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusMutationError::MutationFailed(msg) => write!(f, "Status mutation failed: {msg}"),
            StatusMutationError::StatusNotFound(msg) => write!(f, "Status not found: {msg}"),
            StatusMutationError::InvalidMutation(msg) => write!(f, "Invalid mutation: {msg}"),
            StatusMutationError::NetworkError(msg) => write!(f, "Network error: {msg}"),
            StatusMutationError::BatchMutationFailed(msg) => {
                write!(f, "Batch mutation failed: {msg}")
            }
            StatusMutationError::EnvironmentError(msg) => write!(f, "Environment error: {msg}"),
        }
    }
}

impl std::error::Error for StatusMutationError {}

/// Batch mutation operation for efficient processing
#[derive(Debug, Clone, PartialEq)]
pub struct BatchMutation {
    /// Status ID to mutate
    pub status_id: StatusId,
    /// Mutation to apply
    pub mutation: StatusMutation,
    /// Status view model context
    pub status: StatusViewModel,
}

impl BatchMutation {
    /// Create a new BatchMutation
    #[inline(always)]
    pub fn new(
        mutation: StatusMutation,
        status: StatusViewModel,
        _timestamp: std::time::Instant,
        _priority: u8,
    ) -> Self {
        Self {
            status_id: status.id.clone(),
            mutation,
            status,
        }
    }
}

/// Batch mutation result tracking
#[derive(Debug, Clone)]
pub struct BatchResult {
    /// Successfully processed mutations
    pub successful: Vec<(StatusId, StatusMutation)>,
    /// Failed mutations with error messages
    pub failed: Vec<(StatusId, StatusMutation, String)>,
    /// Total processing time in milliseconds
    pub processing_time_ms: u64,
}

impl BatchResult {
    /// Create a new empty batch result
    #[inline(always)]
    pub fn new() -> Self {
        Self {
            successful: Vec::new(),
            failed: Vec::new(),
            processing_time_ms: 0,
        }
    }

    /// Check if all mutations in the batch were successful
    #[inline(always)]
    pub fn is_fully_successful(&self) -> bool {
        self.failed.is_empty()
    }

    /// Get the total number of mutations processed
    #[inline(always)]
    pub fn total_mutations(&self) -> usize {
        self.successful.len() + self.failed.len()
    }

    /// Get the count of failed mutations
    #[inline(always)]
    pub fn failed_count(&self) -> usize {
        self.failed.len()
    }
}

/// Create a queue for optimized mutation processing
///
/// This provides a way to queue mutations for batch processing
/// to improve performance and reduce network requests.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct MutationQueue {
    queue: VecDeque<BatchMutation>,
    max_batch_size: usize,
    flush_interval_ms: u64,
}

impl MutationQueue {
    /// Create a new mutation queue with specified parameters
    #[inline(always)]
    pub fn new(max_batch_size: usize, flush_interval_ms: u64) -> Self {
        Self {
            queue: VecDeque::new(),
            max_batch_size,
            flush_interval_ms,
        }
    }

    /// Add a mutation to the queue
    #[inline(always)]
    pub fn enqueue(&mut self, mutation: BatchMutation) {
        self.queue.push_back(mutation);
    }

    /// Check if the queue should be flushed
    #[inline(always)]
    pub fn should_flush(&self) -> bool {
        self.queue.len() >= self.max_batch_size
    }

    /// Flush the queue and return all pending mutations
    #[inline(always)]
    pub fn flush(&mut self) -> Vec<BatchMutation> {
        self.queue.drain(..).collect()
    }

    /// Get the current queue length
    #[inline(always)]
    pub fn len(&self) -> usize {
        self.queue.len()
    }

    /// Check if the queue is empty
    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}
