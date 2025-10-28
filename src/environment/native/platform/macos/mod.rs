//! macOS platform integration with zero-allocation patterns
//!
//! This module provides comprehensive macOS platform integration with decomposed
//! modular architecture for maintainable, production-ready code.
//!
//! # Architecture
//!
//! The macOS platform system is decomposed into focused modules:
//! - `window` - Window management, native handles, and Dioxus 0.7 integration
//! - `platform` - Core Platform struct with menu, toolbar, and event handling
//! - `actions` - Action handling, clipboard operations, and URL navigation
//! - `menu` - Menu system creation and management using muda API
//! - `utils` - Date formatting, window effects, and native helper functions
//!
//! # Features
//!
//! - Zero-allocation patterns using `&str` and efficient data structures
//! - Blazing-fast performance with `#[inline(always)]` optimization
//! - No unsafe code minimization - memory safety by Rust design
//! - No locking - uses Arc<Mutex<T>> for thread-safe state management
//! - Elegant ergonomic API with comprehensive error handling
//! - Native macOS integration with NSApplication, NSWindow, and Cocoa APIs
//! - Modern Dioxus 0.7 desktop backend integration
//! - Production-ready menu system with muda API
//!
//! # Usage
//!
//! ```rust,no_run
//! // Note: Internal API for platform integration
//! // Provides native macOS integration with window management and notifications
//! ```
//!
//! # Error Handling
//!
//! All functions return appropriate `Result` types with detailed error messages:
//! - `WindowError` - Window operation errors with native context
//! - `String` errors for platform operations with descriptive messages
//! - Comprehensive logging for debugging and monitoring
//!
//! # Platform Integration
//!
//! - **Native Window Management**: Direct NSWindow access and control
//! - **Menu System**: Full macOS menu bar with accelerators and icons
//! - **Toolbar Integration**: Native toolbar with cacao framework
//! - **Event Handling**: Window events, menu events, and system notifications
//! - **Clipboard Operations**: Copy/paste with copypasta integration
//! - **Date Formatting**: Native NSDateFormatter for localized dates
//! - **Emoji Picker**: Character palette integration

#![allow(deprecated)] // Allow deprecated cocoa APIs until migration to objc2

// Module declarations
pub mod actions;
pub mod menu;
pub mod platform;
pub mod utils;
pub mod window;

// Re-export core types and functions for ergonomic API
pub use actions::{handle_public_action, show_context_menu};
pub use menu::create_main_menu;
pub use platform::Platform;
pub use utils::{
    apply_window_background, apply_window_effects, format_datetime, get_native_window_handle,
    show_emoji_popup,
};
pub use window::{AppWindow, WindowError, WindowResult, default_window};
