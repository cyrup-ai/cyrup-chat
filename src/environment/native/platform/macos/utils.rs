//! Utility functions for macOS platform with zero-allocation patterns
//!
//! This module provides helper functions for date formatting, window effects,
//! and native macOS integrations with comprehensive error handling.

use super::window::AppWindow;
use chrono::{DateTime, Utc};
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::{MainThreadMarker, msg_send, msg_send_id};
use objc2_app_kit::{
    NSVisualEffectBlendingMode, NSVisualEffectMaterial, NSVisualEffectState, NSVisualEffectView,
};
use objc2_foundation::{NSDate, NSDateFormatter, NSDateFormatterStyle, NSRect};

// Re-export show_emoji_popup from actions.rs (already implemented with objc2)
pub use crate::environment::native::platform::macos::actions::show_emoji_popup;

/// Date formatter trait for NSDateFormatter
pub trait NSDateFormatterTrait: Sized {}

impl NSDateFormatterTrait for Retained<NSDateFormatter> {}

/// Format datetime using native macOS NSDateFormatter with objc2
#[inline(always)]
pub fn format_datetime(datetime: &DateTime<Utc>) -> (String, String) {
    let tt = datetime.timestamp() as f64;
    let date = NSDate::dateWithTimeIntervalSince1970(tt);

    let duration = Utc::now().signed_duration_since(*datetime);
    let human = if duration.num_hours() <= 24 {
        // Short time style for recent posts
        let formatter = NSDateFormatter::new();
        formatter.setDateStyle(NSDateFormatterStyle::NoStyle);
        formatter.setTimeStyle(NSDateFormatterStyle::ShortStyle);
        let result = formatter.stringFromDate(&date);
        result.to_string()
    } else if duration.num_days() <= 6 {
        // Day of week for posts within last week
        datetime.format("%A").to_string()
    } else {
        // Date only for older posts
        let formatter = NSDateFormatter::new();
        formatter.setDateStyle(NSDateFormatterStyle::ShortStyle);
        formatter.setTimeStyle(NSDateFormatterStyle::NoStyle);
        let result = formatter.stringFromDate(&date);
        result.to_string()
    };

    // Full date and time string
    let formatter_full = NSDateFormatter::new();
    formatter_full.setDateStyle(NSDateFormatterStyle::MediumStyle);
    formatter_full.setTimeStyle(NSDateFormatterStyle::MediumStyle);
    let dfull = formatter_full.stringFromDate(&date);

    (human, dfull.to_string())
}

/// Get native window handle for macOS-specific operations
#[inline(always)]
pub fn get_native_window_handle(window: &AppWindow) -> Result<*mut AnyObject, String> {
    window
        .get_native_window()
        .map_err(|e| format!("Failed to get native window handle: {}", e))
}

/// Apply window effects like vibrancy to macOS windows using objc2
///
/// # Safety
/// This function uses Objective-C runtime calls which are inherently unsafe.
/// The caller must ensure that `window_handle` is a valid NSWindow pointer.
#[inline(always)]
pub unsafe fn apply_window_effects(window_handle: *mut AnyObject) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Get the window's content view using msg_send_id! (returns Retained)
        let content_view: Retained<AnyObject> = msg_send_id![window_handle, contentView];

        // Get MainThreadMarker - NSVisualEffectView::new() requires it
        let mtm = MainThreadMarker::new().ok_or("NSVisualEffectView requires main thread")?;

        // Create NSVisualEffectView using verified objc2 API
        let effect_view = NSVisualEffectView::new(mtm);

        // Get bounds and set frame
        let bounds: NSRect = msg_send![&*content_view, bounds];
        effect_view.setFrame(bounds);

        // Configure visual effect properties using typed enums
        effect_view.setMaterial(NSVisualEffectMaterial::Sidebar);
        effect_view.setBlendingMode(NSVisualEffectBlendingMode::BehindWindow);
        effect_view.setState(NSVisualEffectState::Active);

        // Add effect view as subview at the back
        let _: () = msg_send![&*content_view, addSubview: &*effect_view positioned: 0i64 relativeTo: std::ptr::null::<*mut AnyObject>()];

        // Make effect view fill the entire content area
        // NSViewWidthSizable | NSViewHeightSizable = 2 | 16 = 18
        let _: () = msg_send![&*effect_view, setAutoresizingMask: 18u64];

        log::debug!("NSVisualEffectView applied successfully with Sidebar material");
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Window effects not supported on this platform");
        Err("Window effects only supported on macOS".to_string())
    }
}

/// Apply window background styling for macOS
#[inline(always)]
pub fn apply_window_background(window: &AppWindow) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        // Get native window handle and apply effects
        let native_window = get_native_window_handle(window)?;
        unsafe { apply_window_effects(native_window) }
    }
    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Window background effects not supported on this platform");
        Err("Window background effects only supported on macOS".to_string())
    }
}
