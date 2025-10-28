//! macOS platform integration with modular architecture
//!
//! This module provides the core Platform struct for macOS with comprehensive
//! menu, toolbar, and window management functionality split across focused modules.

pub mod actions_delegation;
pub mod core;
pub mod notifications;
pub mod toolbar_management;
pub mod window_management;

pub use core::{Platform, ToolbarHandlerUpdateCell, ToolbarType};
