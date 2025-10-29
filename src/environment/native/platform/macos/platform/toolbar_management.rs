//! Toolbar management functionality for macOS Platform
//!
//! This module handles toolbar setup, updates, event handling, and window integration.

use super::core::{Platform, ToolbarType};
use cacao::{
    appkit::{toolbar::Toolbar, window::WindowToolbarStyle},
    foundation::NSUInteger,
};
use objc2::{msg_send, runtime::AnyObject};

use super::super::{
    super::super::toolbar::{LoggedInToolbar, LoggedOutBar},
    window::AppWindow,
};
use crate::environment::{storage::UiTab, types::AppEvent};

impl Platform {
    /// Setup toolbar for the window with proper error handling
    #[inline(always)]
    pub fn setup_toolbar(&self, window: &AppWindow) -> Result<(), String> {
        let toolbar = Toolbar::new("com.cyrup.CYRUPToolbar.LoggedOut", LoggedOutBar::new());

        // Validate window context is available before native operations
        let _context = window
            .get_context()
            .map_err(|e| format!("Window context unavailable for toolbar setup: {}", e))?;

        // Use the new production window management system
        let native_window = window
            .get_native_window()
            .map_err(|e| format!("Failed to get native window for toolbar setup: {}", e))?;

        unsafe {
            let toolbar_ptr = &*toolbar.objc as *const _ as *mut AnyObject;
            let () = msg_send![native_window, setToolbar: toolbar_ptr];
            let style: NSUInteger = WindowToolbarStyle::Unified.into();
            let () = msg_send![native_window, setToolbarStyle: style];
        }

        *self.toolbar.borrow_mut() = ToolbarType::LoggedOut(toolbar);
        log::debug!("Toolbar setup completed successfully");
        Ok(())
    }

    /// Set toolbar event handler
    #[inline(always)]
    pub fn set_toolbar_handler(&self, handler: std::sync::Arc<dyn Fn(AppEvent) + Send + Sync>) {
        if let Ok(mut handler_guard) = self.toolbar_handler.lock() {
            *handler_guard = Some(handler);
        } else {
            log::error!("Failed to acquire toolbar handler lock");
        }
    }

    /// Update toolbar configuration for logged in state
    #[inline(always)]
    pub fn update_toolbar(
        &self,
        account: &str,
        tab: &UiTab,
        has_notifications: bool,
    ) -> Result<(), String> {
        log::trace!("update_toolbar {tab:?}");

        let handler = match self.toolbar_handler.lock() {
            Ok(guard) => match guard.clone() {
                Some(h) => h,
                None => {
                    let error = "No toolbar handler set - call set_toolbar_handler first";
                    log::error!("{}", error);
                    return Err(error.to_string());
                }
            },
            Err(_) => {
                let error = "Failed to acquire toolbar handler lock";
                log::error!("{}", error);
                return Err(error.to_string());
            }
        };

        let toolbar = Toolbar::new(
            "com.cyrup.CYRUPToolbar.LoggedIn",
            LoggedInToolbar::new(
                account.to_string(),
                super::super::super::tab_index(tab),
                has_notifications,
                handler,
            ),
        );

        log::debug!("Updating toolbar configuration for account: {}", account);

        // Apply toolbar to the current window through proper window management
        if let Some(window) = self.get_main_window() {
            if let Err(e) = self.apply_toolbar_to_window(&toolbar, window) {
                log::error!("Failed to apply toolbar to window: {}", e);
                return Err(format!("Toolbar application failed: {}", e));
            }
        } else {
            log::warn!("No main window available for toolbar application");
        }

        *self.toolbar.borrow_mut() = ToolbarType::LoggedIn(toolbar);
        log::debug!("Toolbar updated successfully for account: {}", account);
        Ok(())
    }

    /// Setup logged out toolbar state
    #[inline(always)]
    pub fn loggedout_toolbar(&self, window: &AppWindow) -> Result<(), String> {
        let toolbar = Toolbar::new("com.cyrup.CYRUPToolbar", LoggedOutBar::new());

        // Validate window context is available before native operations
        let _context = window
            .get_context()
            .map_err(|e| format!("Window context unavailable for logout toolbar: {}", e))?;

        // Use the new production window management system
        let native_window = window
            .get_native_window()
            .map_err(|e| format!("Failed to get native window for logout toolbar: {}", e))?;

        unsafe {
            let toolbar_ptr = &*toolbar.objc as *const _ as *mut AnyObject;
            let () = msg_send![native_window, setToolbar: toolbar_ptr];
        }

        *self.toolbar.borrow_mut() = ToolbarType::LoggedOut(toolbar);
        log::debug!("Logged out toolbar set successfully");
        Ok(())
    }

    /// Apply toolbar to the specified window
    #[inline(always)]
    pub(super) fn apply_toolbar_to_window<T>(
        &self,
        toolbar: &Toolbar<T>,
        window: *mut std::ffi::c_void,
    ) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            if window.is_null() {
                return Err("Invalid window pointer provided".to_string());
            }

            unsafe {
                // Cast to AnyObject pointer for objc2 compatibility
                let window_ptr = window as *mut AnyObject;
                let toolbar_ptr = &*toolbar.objc as *const _ as *mut AnyObject;

                // Set the toolbar on the window
                let () = msg_send![window_ptr, setToolbar: toolbar_ptr];

                // Make toolbar visible
                let () = msg_send![window_ptr, setToolbarStyle: 0u64]; // NSWindowToolbarStyleAutomatic

                log::debug!("Applied native toolbar to NSWindow successfully");
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            log::debug!("Toolbar application not supported on this platform");
        }

        Ok(())
    }
}
