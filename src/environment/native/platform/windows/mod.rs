//! Windows platform module with decomposed functionality
//!
//! This module coordinates the Windows platform implementation across multiple
//! specialized submodules for better maintainability and code organization.

mod actions_navigation;
mod core;
mod menu_toolbar;
mod notifications;
mod webview_helpers;
mod window_management;

// Re-export the main types and functions
pub use core::{AppWindow, Platform, WindowsNotification, default_window};
pub use window_management::apply_window_background;

// The implementation methods are automatically available through the impl blocks
// in the respective modules due to Rust's module system
