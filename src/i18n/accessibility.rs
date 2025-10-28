//! Accessibility helpers for i18n text
//!
//! This module provides accessibility-focused functions for ARIA labels,
//! screen reader announcements, and other a11y text needs.

use super::api::t_safe;
use super::core::TextKey;

/// Accessibility helper for ARIA labels and screen reader text
pub struct A11y;

impl A11y {
    /// Get ARIA label text for UI element
    pub fn aria_label(key: TextKey) -> &'static str {
        t_safe(key)
    }

    /// Get screen reader announcement text
    pub fn announce(key: TextKey) -> &'static str {
        t_safe(key)
    }

    /// Get button accessible name
    pub fn button_name(key: TextKey) -> &'static str {
        t_safe(key)
    }

    /// Get input field label
    pub fn input_label(key: TextKey) -> &'static str {
        t_safe(key)
    }
}
