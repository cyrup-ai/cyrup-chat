//! Core async platform definitions and structures
//!
//! This module contains the fundamental data structures and enums
//! for async platform operations across different operating systems.

/// Platform-specific cursor positioning
#[derive(Debug, Clone, Copy)]
pub enum CursorPosition {
    Start,
    End,
    Position(usize),
    SelectAll,
}

/// Text area configuration for async operations
#[derive(Debug, Clone)]
pub struct TextAreaConfig {
    pub auto_focus: bool,
    pub cursor_position: CursorPosition,
    pub readonly: bool,
    pub multiline: bool,
    pub delay_ms: Option<u32>,
    pub select_all_on_focus: bool,
}

/// Async platform operations abstraction
pub struct AsyncPlatform;

impl Default for TextAreaConfig {
    fn default() -> Self {
        Self {
            auto_focus: false,
            cursor_position: CursorPosition::End,
            readonly: false,
            multiline: true,
            delay_ms: Some(150), // Default delay for UI stability
            select_all_on_focus: false,
        }
    }
}
