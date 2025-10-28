//! Status mutation action handlers for loggedin state management
//!
//! This module provides a comprehensive status mutation system with zero-allocation,
//! production-ready implementations following the decomposed modular architecture.
//!
//! # Architecture
//!
//! The status mutation system is decomposed into focused modules:
//! - `types` - Error types, data structures, and mutation queue
//! - `handlers` - Core mutation handlers for success/failure cases
//! - `batch` - Batch processing logic for efficient network operations
//! - `utils` - Utility functions for mutation descriptions and priorities
//!
//! # Features
//!
//! - Zero-allocation patterns using `&str` and efficient data structures
//! - Blazing-fast performance with `#[inline(always)]` optimization
//! - No unsafe code - memory safety guaranteed by Rust
//! - No locking - uses Dioxus signals for thread-safe state management
//! - Elegant ergonomic API with comprehensive error handling
//! - Batch processing support for network efficiency
//! - Priority-based mutation queuing
//!
//! # Usage
//!
//! ```rust,no_run
//! // Note: Internal API for status mutation handling
//! // Processes likes, boosts, favorites with batch optimization and error handling
//! ```
//!
//! # Error Handling
//!
//! All functions return `Result<(), StatusMutationError>` with comprehensive error types:
//! - `MutationFailed` - Mutation execution errors with context
//! - `StatusNotFound` - Missing status errors with details
//! - `BatchMutationFailed` - Batch operation errors with batch context
//! - `InvalidMutation` - Validation errors with specific messages
//! - `NetworkError` - Network operation errors with connection details
//! - `EnvironmentError` - Environment operation errors
//!
//! # Performance Features
//!
//! - Batch processing groups mutations by type for optimal network usage
//! - Priority-based processing ensures user interactions are handled first
//! - Mutation queuing allows for delayed batch operations
//! - Zero-allocation string handling with static descriptions

// Module declarations
pub mod batch;
pub mod handlers;
pub mod types;
pub mod utils;

// Re-export core types and functions for ergonomic API
pub use handlers::handle_status_mutation_result;
pub use types::{MutationQueue, StatusMutationError};
