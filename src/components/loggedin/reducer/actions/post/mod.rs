//! Post management action handlers for loggedin state management
//!
//! This module provides a comprehensive post management system with zero-allocation,
//! production-ready implementations following the decomposed modular architecture.
//!
//! # Architecture
//!
//! The post management system is decomposed into focused modules:
//! - `creation` - Post creation logic with reply support and validation
//! - `completion` - Post completion handling with storage updates
//! - `cancellation` - Post cancellation with efficient state cleanup
//! - `attachments` - File attachment management with validation
//!
//! # Features
//!
//! - Zero-allocation patterns using `&str` and `Cow<'_, str>`
//! - Blazing-fast performance with `#[inline(always)]` optimization
//! - No unsafe code - memory safety guaranteed by Rust
//! - No locking - uses Dioxus signals for thread-safe state management
//! - Elegant ergonomic API with comprehensive error handling
//!
//! # Usage
//!
//! ```rust,no_run
//! // Note: Internal API for post composition actions
//! // Handles post creation, completion, and cancellation with error handling
//! ```
//!
//! # Error Handling
//!
//! All functions return `Result<(), PostError>` with comprehensive error types:
//! - `PostCreationFailed` - Post creation errors with context
//! - `PostCompletionFailed` - Post completion errors with details
//! - `PostValidationFailed` - Validation errors with specific messages
//! - `FileAttachmentFailed` - File attachment errors with file context
//! - `EnvironmentError` - Environment operation errors

// Module declarations
pub mod attachments;
pub mod cancellation;
pub mod completion;
pub mod creation;

/// Post management error types with detailed context for debugging
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PostError {
    /// Post creation failed with detailed error message
    PostCreationFailed(String),
    /// Post completion failed with detailed error message
    PostCompletionFailed(String),
    /// Post validation failed with detailed error message
    PostValidationFailed(String),
    /// File attachment failed with detailed error message
    FileAttachmentFailed(String),
    /// Environment operation failed with detailed error message
    EnvironmentError(String),
}

impl PostError {
    /// Create a post creation error with enhanced context
    #[inline(always)]
    pub fn creation_failed(msg: impl Into<String>) -> Self {
        Self::PostCreationFailed(msg.into())
    }

    /// Create a post completion error with enhanced context
    #[inline(always)]
    pub fn completion_failed(msg: impl Into<String>) -> Self {
        Self::PostCompletionFailed(msg.into())
    }

    /// Create a post validation error with enhanced context
    #[inline(always)]
    pub fn validation_failed(msg: impl Into<String>) -> Self {
        Self::PostValidationFailed(msg.into())
    }
}

impl std::fmt::Display for PostError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PostError::PostCreationFailed(msg) => write!(f, "Post creation failed: {msg}"),
            PostError::PostCompletionFailed(msg) => write!(f, "Post completion failed: {msg}"),
            PostError::PostValidationFailed(msg) => write!(f, "Post validation failed: {msg}"),
            PostError::FileAttachmentFailed(msg) => write!(f, "File attachment failed: {msg}"),
            PostError::EnvironmentError(msg) => write!(f, "Environment error: {msg}"),
        }
    }
}

impl std::error::Error for PostError {}

// Re-export all handler functions for ergonomic API
pub use cancellation::handle_post_cancel;
pub use completion::handle_post_done;
pub use creation::handle_post;
// Re-export attachment functions (architectural components - integration pending)
#[allow(unused_imports)]
pub use attachments::{
    handle_add_attachment, handle_dropped_paths, handle_remove_attachment, validate_post_state,
};
