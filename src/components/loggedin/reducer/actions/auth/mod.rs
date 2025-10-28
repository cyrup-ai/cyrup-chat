//! Authentication action handlers for loggedin state management
//!
//! This module provides zero-allocation, production-ready authentication
//! handlers with comprehensive error recovery and optimal performance.

mod error_types;
mod login_handlers;
mod logout_handlers;
mod stream_processing;

#[allow(unused_imports)] // Public API - used by environment model and other auth components
pub use error_types::{AuthError, ResultExt};
pub use login_handlers::{handle_logged_in, handle_login};
pub use logout_handlers::{handle_logout, handle_logout_done};
#[allow(unused_imports)] // Public API - used by login handlers for stream setup
pub use stream_processing::{process_stream_message, setup_user_stream};
