//! Loggedin state action handlers with zero-allocation dispatch
//!
//! This module provides efficient action routing with compile-time optimizations
//! and comprehensive error handling for all loggedin state management operations.

pub mod auth;
pub mod core;
pub mod handlers;
pub mod post;
pub mod selection;
pub mod status_mutation;

// Re-export core API
pub use core::{Action, ActionError, ReducerState, handle_action};

// Re-export post functions for convenience
#[allow(unused_imports)]
pub use post::{
    PostError, handle_add_attachment, handle_post, handle_post_cancel, handle_post_done,
    handle_remove_attachment,
};

// Re-export only used status_mutation items for clean API
// StatusMutationError is used in ActionError enum and From trait implementation
