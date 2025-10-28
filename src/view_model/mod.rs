//! View model module with focused domain separation
//!
//! This module provides view model functionality decomposed into focused
//! domain-specific modules with zero-allocation patterns and comprehensive
//! API compatibility through re-exports.

// Mastodon types (existing)
pub mod account;
pub mod media;
pub mod notification;
pub mod status;
pub mod types;

// Agent chat types (AGENT_2)
pub mod agent;
pub mod conversation;
pub mod message;
pub mod token_budget;

// Re-export Mastodon types for API compatibility
pub use account::*;
pub use media::*;
pub use notification::*;
pub use status::*;
pub use types::*;

// Re-export agent chat types
pub use agent::*;
pub use conversation::*;
pub use message::*;
pub use token_budget::*;
