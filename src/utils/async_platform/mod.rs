//! Async platform operations module with decomposed functionality
//!
//! This module coordinates async platform implementations across multiple
//! specialized submodules for better maintainability and code organization.

mod core;
mod cross_platform_impl;
mod macos_impl;
mod public_api;

// Re-export the main types and functionality
pub use core::{AsyncPlatform, CursorPosition, TextAreaConfig};

// The implementation methods are automatically available through the impl blocks
// in the respective modules due to Rust's module system
