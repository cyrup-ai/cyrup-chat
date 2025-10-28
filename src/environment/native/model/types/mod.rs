//! Core types and error handling for model operations
//!
//! This module provides fundamental types, error definitions, and wrapper types
//! for the model layer with zero-allocation patterns.

mod account_wrapper;
mod data_wrappers;
mod error_types;
mod model_core;

// Re-export megalodon types for convenience
pub use megalodon::entities::Card;
pub use megalodon::entities::attachment::*;
pub use megalodon::streaming::Message;

// Re-export our types
pub use account_wrapper::Account;
pub use data_wrappers::{AppData, TokenData};
pub use error_types::{ModelError, ResultExt};
pub use model_core::Model;
