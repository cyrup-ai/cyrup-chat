//! Window management functionality for macOS Platform
//!
//! This module handles window operations, fullscreen detection, navigation, and scrolling.

use super::core::Platform;
// objc2 core types for memory management and thread safety
use objc2::rc::Retained;
use objc2::{MainThreadMarker, MainThreadOnly, msg_send};
// objc2_foundation geometry and string types
use objc2_foundation::{NSPoint, NSRect, NSSize, NSString};
// objc2_app_kit window management types
use objc2_app_kit::{NSApplication, NSBackingStoreType, NSWindow, NSWindowStyleMask};

impl Platform {
    /// Check if window is in fullscreen mode using native macOS APIs
    #[inline(always)]
    pub fn is_fullscreen(&self) -> Result<bool, String> {
        #[cfg(target_os = "macos")]
        {
            // Pattern from: src/utils/native_platform/macos.rs:276-278
            let mtm = MainThreadMarker::new().ok_or("must be on main thread")?;
            let app = NSApplication::sharedApplication(mtm);

            // Pattern from: src/utils/native_platform/macos.rs:279-281
            let main_window = app.mainWindow();
            match main_window {
                Some(window) => {
                    let style_mask = window.styleMask();
                    let is_fullscreen = style_mask.contains(NSWindowStyleMask::FullScreen);
                    Ok(is_fullscreen)
                }
                None => Ok(false), // No main window, not fullscreen
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(false)
        }
    }

    /// Open a new window with specified parameters
    #[inline(always)]
    pub async fn open_window(&self, title: &str, width: u32, height: u32) -> Result<(), String> {
        log::debug!("Opening new window: {} ({}x{})", title, width, height);

        #[cfg(target_os = "macos")]
        {
            // Pattern from: src/utils/native_platform/macos.rs:276-278
            let mtm = MainThreadMarker::new().ok_or("must be on main thread")?;

            // Pattern from: tmp/objc2/crates/objc2/examples/hello_world_app.rs:58-62
            let frame = NSRect::new(
                NSPoint::new(100.0, 100.0),
                NSSize::new(width as f64, height as f64),
            );

            // Type-safe style mask flags
            let style_mask = NSWindowStyleMask::Titled
                | NSWindowStyleMask::Closable
                | NSWindowStyleMask::Resizable;

            // Pattern from: tmp/objc2/crates/objc2/examples/hello_world_app.rs:67-77
            let window = unsafe {
                NSWindow::initWithContentRect_styleMask_backing_defer(
                    NSWindow::alloc(mtm),
                    frame,
                    style_mask,
                    NSBackingStoreType::Buffered,
                    false,
                )
            };

            // Pattern from: src/notifications/macos_backend.rs:64
            let ns_title = NSString::from_str(title);
            window.setTitle(&ns_title);

            // Pattern from: tmp/objc2/crates/objc2/examples/hello_world_app.rs:88
            window.makeKeyAndOrderFront(None);

            log::debug!("Created new NSWindow: {}", title);
        }

        Ok(())
    }

    /// Navigate to a URL or path using platform-specific methods
    #[inline(always)]
    pub fn navigate(&self, path: &str) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            use std::process::Command;
            let _ = Command::new("open").arg(path).spawn();
        }
        #[cfg(target_os = "windows")]
        {
            use std::process::Command;
            let _ = Command::new("cmd").args(&["/C", "start", path]).spawn();
        }
        #[cfg(target_os = "linux")]
        {
            use std::process::Command;
            let _ = Command::new("xdg-open").arg(path).spawn();
        }
        Ok(())
    }

    /// Scroll to a specific element by ID
    #[inline(always)]
    pub async fn scroll_to_element(&self, element_id: &str) -> Result<(), String> {
        log::debug!("Scrolling to element: {}", element_id);

        #[cfg(target_os = "macos")]
        {
            let script = format!(
                "const element = document.getElementById('{}'); if (element) {{ element.scrollIntoView({{ behavior: 'smooth', block: 'start' }}); return 'scrolled'; }} else {{ return 'not_found'; }}",
                element_id
            );

            match self.execute_javascript(&script).await {
                Ok(result) => {
                    if result == "not_found" {
                        log::warn!("Element not found: {}", element_id);
                        Err(format!("Element '{}' not found", element_id))
                    } else {
                        log::debug!("Successfully scrolled to element: {}", element_id);
                        Ok(())
                    }
                }
                Err(e) => {
                    log::error!("Failed to scroll to element {}: {}", element_id, e);
                    Err(format!("Scroll to element failed: {}", e))
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::debug!("Element scrolling not supported on this platform");
            Ok(())
        }
    }

    /// Scroll to element with specific behavior (smooth/instant)
    #[inline(always)]
    pub async fn scroll_to_element_with_behavior(
        &self,
        element_id: &str,
        behavior: &str,
    ) -> Result<(), String> {
        log::debug!(
            "Scrolling to element {} with behavior {}",
            element_id,
            behavior
        );

        #[cfg(target_os = "macos")]
        {
            let script = format!(
                "const element = document.getElementById('{}'); if (element) {{ element.scrollIntoView({{ behavior: '{}', block: 'start' }}); return 'scrolled'; }} else {{ return 'not_found'; }}",
                element_id, behavior
            );

            match self.execute_javascript(&script).await {
                Ok(result) => {
                    if result == "not_found" {
                        log::warn!("Element not found: {}", element_id);
                        return Err(format!("Element '{}' not found", element_id));
                    } else {
                        log::debug!(
                            "Successfully scrolled to element: {} with behavior: {}",
                            element_id,
                            behavior
                        );
                    }
                }
                Err(e) => {
                    log::error!(
                        "Failed to scroll to element {} with behavior {}: {}",
                        element_id,
                        behavior,
                        e
                    );
                    return Err(format!("Scroll to element failed: {}", e));
                }
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::debug!("Element scrolling not supported on this platform");
        }

        Ok(())
    }

    /// Get the main application window
    #[inline(always)]
    pub(super) fn get_main_window(&self) -> Option<*mut std::ffi::c_void> {
        // Pattern from: src/utils/native_platform/macos.rs:276-279
        let mtm = MainThreadMarker::new()?;
        let app = NSApplication::sharedApplication(mtm);

        // mainWindow() returns Option<Retained<NSWindow>>
        let window = app.mainWindow()?;

        // Convert Retained<NSWindow> to raw c_void pointer for backward compatibility
        // The pointer remains valid as long as the window exists
        let ptr = Retained::as_ptr(&window) as *mut std::ffi::c_void;
        Some(ptr)
    }

    /// Close preferences window if open
    #[inline(always)]
    pub fn close_preferences_window(&self) -> Result<(), String> {
        log::debug!("Closing preferences window");

        #[cfg(target_os = "macos")]
        {
            // Pattern from: src/utils/native_platform/macos.rs:276-278
            let mtm = MainThreadMarker::new().ok_or("must be on main thread")?;
            let app = NSApplication::sharedApplication(mtm);

            // Get all windows as NSArray
            let windows = app.windows();
            let window_count = windows.count();

            for i in 0..window_count {
                // Get window at index (small unsafe block for NSArray access)
                let window: &NSWindow = unsafe {
                    let obj: *const NSWindow = msg_send![&*windows, objectAtIndex: i];
                    &*obj
                };

                // Get window title (safe objc2 API)
                let title = window.title();
                let title_string = title.to_string();

                // Check if this is a preferences window by title
                if title_string.to_lowercase().contains("preferences")
                    || title_string.to_lowercase().contains("settings")
                {
                    window.close();
                    log::debug!("Closed preferences window: {}", title_string);
                    return Ok(());
                }
            }

            log::debug!("No preferences window found to close");
        }

        Ok(())
    }
}
