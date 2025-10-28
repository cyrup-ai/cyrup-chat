//! macOS window management with zero-allocation patterns
//!
//! This module provides production-ready window management for macOS platform
//! integrating with Dioxus 0.7 desktop backend for native functionality.

use dioxus_desktop::DesktopContext;
use dioxus_desktop::tao::platform::macos::WindowBuilderExtMacOS;
use dioxus_desktop::{LogicalSize, WindowBuilder};
use objc2::{msg_send, runtime::AnyObject};
use std::{
    string::ToString,
    sync::{Arc, Weak},
};

/// Production-ready window management for macOS platform
/// Integrates with Dioxus 0.7 desktop backend for native functionality
#[derive(Clone)]
pub struct AppWindow {
    /// Weak reference to Dioxus desktop context to avoid cycles
    pub desktop_context: Weak<DesktopContext>,
    /// Window identifier for tracking multiple windows
    pub window_id: String,
    /// Native window handle for direct macOS integration
    pub native_handle: Option<Arc<dyn std::any::Any + Send + Sync>>,
}

/// Error types for window operations
#[derive(Debug, thiserror::Error)]
pub enum WindowError {
    #[error("Desktop context not available")]
    ContextUnavailable,
    #[error("Window not found: {0}")]
    WindowNotFound(String),
    #[error("Native operation failed: {0}")]
    NativeOperationFailed(String),
}

impl WindowError {
    /// Create a context unavailable error with enhanced context
    #[inline(always)]
    pub fn context_unavailable() -> Self {
        Self::ContextUnavailable
    }

    /// Create a window not found error with enhanced context
    #[inline(always)]
    pub fn window_not_found(window_id: impl Into<String>) -> Self {
        Self::WindowNotFound(window_id.into())
    }

    /// Create a native operation failed error with enhanced context
    #[inline(always)]
    pub fn native_operation_failed(msg: impl Into<String>) -> Self {
        Self::NativeOperationFailed(msg.into())
    }
}

pub type WindowResult<T> = Result<T, WindowError>;

impl AppWindow {
    /// Create new AppWindow with proper Dioxus 0.7 integration
    #[inline(always)]
    pub fn new(desktop_context: Weak<DesktopContext>, window_id: String) -> Self {
        Self {
            desktop_context,
            window_id,
            native_handle: None,
        }
    }

    /// Get desktop context with proper error handling
    #[inline(always)]
    pub fn get_context(&self) -> WindowResult<Arc<DesktopContext>> {
        self.desktop_context
            .upgrade()
            .ok_or(WindowError::ContextUnavailable)
    }

    /// Get native window handle for macOS-specific operations using Dioxus 0.7 API
    #[cfg(target_os = "macos")]
    #[inline(always)]
    pub fn get_native_window(&self) -> WindowResult<*mut AnyObject> {
        use dioxus_desktop::tao::platform::macos::WindowExtMacOS;

        // Use modern Dioxus 0.7 window API
        let window = dioxus::desktop::window();

        // Get native NSWindow handle from window
        let native_window = window.window.ns_window() as *mut AnyObject;

        if native_window.is_null() {
            return Err(WindowError::NativeOperationFailed(
                "Failed to get NSWindow handle".to_string(),
            ));
        }

        Ok(native_window)
    }

    /// Check if window is currently fullscreen
    #[inline(always)]
    pub fn is_fullscreen(&self) -> WindowResult<bool> {
        #[cfg(target_os = "macos")]
        {
            let native_window = self.get_native_window()?;
            unsafe {
                let style_mask: usize = msg_send![native_window, styleMask];
                // NSWindowStyleMaskFullScreen = 1 << 14
                Ok((style_mask & (1 << 14)) != 0)
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            Ok(false)
        }
    }

    /// Set window fullscreen state using Dioxus 0.7 API
    #[inline(always)]
    pub fn set_fullscreen(&self, fullscreen: bool) -> WindowResult<()> {
        let window = dioxus::desktop::window();

        if fullscreen {
            window.set_fullscreen(true);
        } else {
            window.set_fullscreen(false);
        }

        Ok(())
    }

    /// Focus the window using Dioxus 0.7 API
    #[inline(always)]
    pub fn focus(&self) -> WindowResult<()> {
        let window = dioxus::desktop::window();
        window.set_focus();
        Ok(())
    }

    /// Minimize the window using Dioxus 0.7 API
    #[inline(always)]
    pub fn minimize(&self) -> WindowResult<()> {
        let window = dioxus::desktop::window();
        window.set_minimized(true);
        Ok(())
    }

    /// Show/hide the window using Dioxus 0.7 API
    #[inline(always)]
    pub fn set_visible(&self, visible: bool) -> WindowResult<()> {
        let window = dioxus::desktop::window();
        window.set_visible(visible);
        Ok(())
    }

    /// Set application menu with comprehensive error handling
    #[inline(always)]
    pub fn set_menu(&self, menu: Option<muda::Menu>) -> WindowResult<()> {
        if let Some(menu) = menu {
            #[cfg(target_os = "macos")]
            {
                // Initialize menu for NSApp
                menu.init_for_nsapp();
                log::debug!("Menu successfully initialized for NSApp using muda API");
            }
            #[cfg(not(target_os = "macos"))]
            {
                log::warn!("Menu setting not supported on this platform");
            }
        } else {
            log::debug!("No menu provided - using default system menu");
        }
        Ok(())
    }

    /// Get window title using Dioxus 0.7 API
    #[inline(always)]
    pub fn get_title(&self) -> WindowResult<String> {
        let window = dioxus::desktop::window();
        Ok(window.title())
    }

    /// Set window title using Dioxus 0.7 API
    #[inline(always)]
    pub fn set_title(&self, title: &str) -> WindowResult<()> {
        let window = dioxus::desktop::window();
        window.set_title(title);
        Ok(())
    }

    /// Get window size using Dioxus 0.7 API
    #[inline(always)]
    pub fn get_size(&self) -> WindowResult<(u32, u32)> {
        let window = dioxus::desktop::window();
        let size = window.inner_size();
        Ok((size.width, size.height))
    }

    /// Set window size using Dioxus 0.7 API
    #[inline(always)]
    pub fn set_size(&self, width: u32, height: u32) -> WindowResult<()> {
        let window = dioxus::desktop::window();
        let size = dioxus_desktop::tao::dpi::LogicalSize::new(width, height);
        window.set_inner_size(size);
        Ok(())
    }

    /// Execute JavaScript code in the webview using modern Dioxus 0.7 eval API
    #[inline(always)]
    pub async fn evaluate_script(&self, script: &str) -> WindowResult<String> {
        use dioxus::document;

        let eval = document::eval(script);
        match eval.await {
            Ok(result) => Ok(result.to_string()),
            Err(e) => Err(WindowError::NativeOperationFailed(format!(
                "Script evaluation failed: {}",
                e
            ))),
        }
    }
}

/// Create default window configuration for macOS
#[inline(always)]
pub fn default_window() -> WindowBuilder {
    let builder = WindowBuilder::new();
    let s = LogicalSize::new(1200., 775.);

    builder
        .with_title("CYRUP")
        .with_theme(Some(dioxus_desktop::tao::window::Theme::Dark))
        .with_inner_size(s)
        .with_decorations(true) // ✅ Native decorations with traffic light buttons
        .with_transparent(true)
        .with_automatic_window_tabbing(false)
        .with_title_hidden(true)
        .with_titlebar_transparent(true)
        .with_fullsize_content_view(true)
}
