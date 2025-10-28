//! Navigation and selection action handlers for loggedin state management
//!
//! This module provides zero-allocation, production-ready navigation
//! handlers with optimized account/timeline selection and comprehensive error recovery.

pub mod account;
pub mod conversation;
pub mod errors;
pub mod more;
pub mod notifications;
pub mod validation;

// Re-export all public items for easy access
#[allow(unused_imports)] // Used by navigation handlers and core dispatcher
pub use account::handle_select_account;
#[allow(unused_imports)] // Used by navigation handlers and core dispatcher
pub use conversation::handle_select_conversation;
pub use errors::SelectionError;
#[allow(unused_imports)] // Used by navigation handlers and core dispatcher
pub use more::handle_select_more;
#[allow(unused_imports)] // Used by navigation handlers and core dispatcher
pub use notifications::handle_select_notifications;
#[allow(unused_imports)] // Used by navigation handlers for state validation
pub use validation::validate_navigation_state;
