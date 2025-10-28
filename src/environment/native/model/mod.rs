//! Model layer - agent chat operations
//!
//! Core model layer for managing agent conversations through:
//! - agent_manager: Spawning and messaging agent sessions
//! - client: High-level conversation operations
//! - types: Model struct and error types
//!
//! # Architecture
//!
//! Model wraps:
//! - ModelAgentManager: Session lifecycle and messaging
//! - Database: Persistent conversation/message storage
//!
//! # Key Operations
//!
//! - list_conversations(): Get all conversations for timeline
//! - send_message(): Send message with lazy agent spawn
//! - get_messages(): Retrieve conversation history
//!
//! # Features
//!
//! - Zero-allocation patterns using `&str` and efficient data structures
//! - Blazing-fast performance with `#[inline(always)]` optimization
//! - No unsafe code - memory safety guaranteed by Rust
//! - Thread-safe state management with Arc<Mutex<T>>
//! - Elegant ergonomic API with comprehensive error handling
//! - Lazy spawn pattern for efficient resource usage
//! - Production-ready agent conversation management
//!
//! # Error Handling
//!
//! All functions return appropriate `Result` types with detailed error messages:
//! - `ModelError` - Model creation and operation errors
//! - Comprehensive logging for debugging and monitoring
//! - Graceful error recovery and fallback handling

// Module declarations
pub mod agent_manager;
pub mod archive_manager;
pub mod client;
pub mod types;

// Re-export core types and functions for ergonomic API
pub use agent_manager::ModelAgentManager;
pub use types::{Account, AppData, Model, ModelError, TokenData};

// Compatibility layer: Re-export megalodon types for gradual migration
pub use megalodon::entities::Notification;
pub use megalodon::entities::Status;
pub use megalodon::entities::StatusVisibility;
pub use megalodon::entities::UploadMedia;
pub use megalodon::entities::notification::NotificationType;
pub use megalodon::streaming::Message;

// Re-export Instance from environment types
pub use crate::environment::types::Instance;

// Re-export Visibility type
pub use crate::view_model::Visibility;

// Note: Mastodon-specific re-exports kept temporarily for compatibility
// During transition from Mastodon to agent chat, these provide type compatibility
