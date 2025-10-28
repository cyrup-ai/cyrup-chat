//! Core module for loggedin reducer actions

pub mod dispatcher;
pub mod errors;
pub mod types;

// Re-export main API
pub use dispatcher::handle_action;
pub use errors::ActionError;
pub use types::{Action, ReducerState};
