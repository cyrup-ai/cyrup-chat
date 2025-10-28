//! Zero-allocation internationalization system for CYRUP Chat
//!
//! This module provides compile-time optimized i18n with zero runtime allocation
//! and blazing-fast text lookups using const generics and static strings.

mod accessibility;
mod api;
mod core;
mod languages;
mod translations;

// Re-export public API
pub use accessibility::A11y;
pub use api::{current_locale, init_i18n, set_locale, t, t_locale, t_safe};
pub use core::{I18n, I18nError, Locale, TextKey};
