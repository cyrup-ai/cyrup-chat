//! Windows actions and navigation implementation
//!
//! This module handles user actions, navigation, scrolling, and UI state management
//! for the Windows platform.

use super::core::Platform;
use std::process::Command;

impl Platform {
    /// Navigate to URL using Windows default browser
    pub fn navigate(&self, path: &str) -> Result<(), String> {
        log::debug!("Navigating to URL on Windows: {}", path);

        let result = Command::new("cmd")
            .args(&["/C", "start", "", path]) // Empty string prevents issues with URLs containing &
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;

        log::info!("URL opened successfully: {}", path);
        Ok(())
    }

    // Scrolling and DOM manipulation
    // Note: These functions provide the platform-specific interface but actual WebView
    // JavaScript execution must be handled at the component level using use_window() hook
    pub async fn scroll_to_element(&self, element_id: &str) -> Result<(), String> {
        log::debug!("Scrolling to element on Windows: {}", element_id);
        self.scroll_to_element_with_behavior(element_id, "smooth")
            .await
    }

    pub async fn scroll_to_element_with_behavior(
        &self,
        element_id: &str,
        behavior: &str,
    ) -> Result<(), String> {
        log::debug!(
            "Scrolling to element {} with behavior {} on Windows",
            element_id,
            behavior
        );

        // Generate JavaScript for scrolling
        let script = super::webview_helpers::generate_scroll_script(element_id, behavior);

        // Execute the script in WebView
        match super::webview_helpers::execute_webview_script(&script).await {
            Ok(super::webview_helpers::WebViewResult::Success(_)) => {
                log::info!("Successfully scrolled to element: {}", element_id);
                Ok(())
            }
            Ok(super::webview_helpers::WebViewResult::NotFound) => {
                Err(format!("Element with ID '{}' not found", element_id))
            }
            Ok(super::webview_helpers::WebViewResult::Error(msg)) => {
                log::error!("JavaScript error during scroll: {}", msg);
                Err(format!("Failed to scroll to element: {}", msg))
            }
            Err(e) => {
                log::error!("WebView execution error: {}", e);
                Err(format!("WebView execution failed: {}", e))
            }
        }
    }

    // UI state management
    pub fn should_auto_reload(&self) -> Result<bool, String> {
        log::debug!("Checking if auto-reload should occur on Windows");

        // Check network connectivity
        let has_network = self.check_network_connectivity()?;
        if !has_network {
            log::debug!("Auto-reload disabled: No network connectivity");
            return Ok(false);
        }

        // Check if application is in focus
        let is_focused = self.is_application_focused()?;
        if !is_focused {
            log::debug!("Auto-reload disabled: Application not focused");
            return Ok(false);
        }

        log::debug!("Auto-reload enabled: All conditions met");
        Ok(true)
    }

    /// Check Windows network connectivity using WinInet API
    fn check_network_connectivity(&self) -> Result<bool, String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::NetworkManagement::WinInet::*;

            unsafe {
                let connected = InternetGetConnectedState(None, 0);
                let is_connected = connected.as_bool();
                log::debug!("Network connectivity check: {}", is_connected);
                Ok(is_connected)
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Fallback for non-Windows platforms
            log::debug!("Network connectivity check not available on this platform");
            Ok(true)
        }
    }

    /// Check if the application window is currently focused
    fn is_application_focused(&self) -> Result<bool, String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::*;

            if let Ok(windows) = self.windows.lock() {
                for window in windows.values() {
                    if let Some(hwnd) = window.hwnd {
                        unsafe {
                            let foreground = GetForegroundWindow();
                            if foreground.0 == hwnd {
                                log::debug!("Application window is focused");
                                return Ok(true);
                            }
                        }
                    }
                }
            }

            log::debug!("Application window is not focused");
            Ok(false)
        }

        #[cfg(not(target_os = "windows"))]
        {
            // Fallback for non-Windows platforms
            log::debug!("Focus detection not available on this platform");
            Ok(true)
        }
    }

    pub fn update_sidebar_visibility(&self, visible: bool) {
        log::debug!("Updating sidebar visibility on Windows: {}", visible);

        // Generate JavaScript for sidebar visibility
        let script = super::webview_helpers::generate_sidebar_visibility_script(visible);

        // Execute the script in WebView (async execution in background)
        let script_clone = script.clone();
        tokio::spawn(async move {
            match super::webview_helpers::execute_webview_script(&script_clone).await {
                Ok(super::webview_helpers::WebViewResult::Success(_)) => {
                    log::info!("Successfully updated sidebar visibility to: {}", visible);
                }
                Ok(super::webview_helpers::WebViewResult::NotFound) => {
                    log::warn!("Sidebar element not found in DOM");
                }
                Ok(super::webview_helpers::WebViewResult::Error(msg)) => {
                    log::error!("JavaScript error during sidebar visibility update: {}", msg);
                }
                Err(e) => {
                    log::error!("WebView execution error for sidebar visibility: {}", e);
                }
            }
        });
    }

    pub fn set_text_size(&self, size: f32) {
        log::debug!("Setting text size on Windows: {}", size);

        // Validate size range (0.5x to 3.0x scaling)
        let clamped_size = size.clamp(0.5, 3.0);
        if size != clamped_size {
            log::warn!("Text size {} clamped to {}", size, clamped_size);
        }

        // Generate JavaScript for text size scaling
        let script = super::webview_helpers::generate_text_size_script(clamped_size);

        // Execute the script in WebView (async execution in background)
        let script_clone = script.clone();
        tokio::spawn(async move {
            match super::webview_helpers::execute_webview_script(&script_clone).await {
                Ok(super::webview_helpers::WebViewResult::Success(_)) => {
                    log::info!("Successfully set text size to: {}", clamped_size);
                }
                Ok(super::webview_helpers::WebViewResult::Error(msg)) => {
                    log::error!("JavaScript error during text size update: {}", msg);
                }
                Err(e) => {
                    log::error!("WebView execution error for text size: {}", e);
                }
                _ => {}
            }
        });
    }

    pub fn update_text_size(&self, size: f32) {
        log::debug!("Updating text size on Windows: {}", size);

        // Delegate to set_text_size for consistent behavior
        self.set_text_size(size);

        // Additional Windows-specific DPI awareness setup
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::HiDpi::*;

            // Set process DPI awareness for better text scaling
            unsafe {
                let result =
                    SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
                if result.as_bool() {
                    log::debug!("Successfully set DPI awareness context");
                } else {
                    log::warn!("Failed to set DPI awareness context - may already be set");
                }
            }
        }
    }

    // Public action handling
    pub async fn handle_public_action(
        &self,
        action: crate::PublicAction,
        instance_url: &str,
    ) -> Result<(), String> {
        log::debug!("Handling public action on Windows: {:?}", action);

        match action {
            crate::PublicAction::OpenLink(url) => {
                self.navigate(&url)?;
            }
            crate::PublicAction::Copy(text) => {
                #[cfg(not(target_arch = "wasm32"))]
                {
                    use copypasta::{ClipboardContext, ClipboardProvider};
                    if let Ok(mut ctx) = ClipboardContext::new() {
                        let _ = ctx.set_contents(text);
                    }
                }
            }
            crate::PublicAction::OpenProfile(profile) => {
                log::debug!("Opening profile on Windows: {:?}", profile.username);
            }
            crate::PublicAction::OpenTag(tag) => {
                log::debug!("Opening tag on Windows: {}", tag);

                // Construct tag URL using current instance URL
                let tag_url = format!("{}/tags/{}", instance_url, tag);
                self.navigate(&tag_url)?;
            }
            _ => {
                log::debug!("Unhandled public action on Windows: {:?}", action);
            }
        }

        Ok(())
    }

    /// Show emoji popup using native Windows emoji panel (Win + ;)
    pub fn show_emoji_popup(&self) -> Result<(), String> {
        log::debug!("Showing emoji popup on Windows");

        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::*;
            use windows::Win32::UI::Input::KeyboardAndMouse::*;

            unsafe {
                // Simulate Windows + ; (semicolon) to open native emoji panel
                let inputs = [
                    // Press Windows key
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_LWIN,
                                wScan: 0,
                                dwFlags: KEYBD_EVENT_FLAGS(0),
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                    // Press semicolon key
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_OEM_1, // Semicolon key
                                wScan: 0,
                                dwFlags: KEYBD_EVENT_FLAGS(0),
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                    // Release semicolon key
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_OEM_1,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                    // Release Windows key
                    INPUT {
                        r#type: INPUT_KEYBOARD,
                        Anonymous: INPUT_0 {
                            ki: KEYBDINPUT {
                                wVk: VK_LWIN,
                                wScan: 0,
                                dwFlags: KEYEVENTF_KEYUP,
                                time: 0,
                                dwExtraInfo: 0,
                            },
                        },
                    },
                ];

                let sent = SendInput(&inputs, std::mem::size_of::<INPUT>() as i32);

                if sent == inputs.len() as u32 {
                    log::info!("Successfully triggered Windows emoji panel (Win + ;)");
                    Ok(())
                } else {
                    log::error!(
                        "Failed to send all input events for emoji panel. Sent: {}/{}",
                        sent,
                        inputs.len()
                    );
                    Err("Failed to trigger emoji panel input simulation".to_string())
                }
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            log::warn!("Emoji popup only available on Windows platform");
            Err("Emoji popup only available on Windows platform".to_string())
        }
    }

    /// Show a context menu with the given items and coordinates using Win32 APIs
    pub fn show_context_menu<T: Clone + std::fmt::Debug + Send + 'static>(
        &self,
        coordinates: (i32, i32),
        title: &str,
        items: Vec<(&str, T)>,
        callback: impl Fn(T) + 'static,
    ) -> Result<(), String> {
        log::debug!(
            "Context menu requested at {:?}: {} with {} items",
            coordinates,
            title,
            items.len()
        );

        #[cfg(target_os = "windows")]
        {
            use std::collections::HashMap;
            use std::sync::Arc;
            use windows::Win32::Foundation::*;
            use windows::Win32::UI::WindowsAndMessaging::*;

            if items.is_empty() {
                return Err("Cannot show empty context menu".to_string());
            }

            unsafe {
                // Create popup menu
                let menu = CreatePopupMenu();
                if menu.is_invalid() {
                    return Err("Failed to create popup menu".to_string());
                }

                // Store callback mappings for menu item selection
                let mut callback_map: HashMap<u32, T> = HashMap::new();

                // Add menu items
                for (index, (label, value)) in items.iter().enumerate() {
                    let menu_id = 1000 + index as u32;

                    // Convert label to wide string for Win32 API
                    let wide_label: Vec<u16> =
                        label.encode_utf16().chain(std::iter::once(0)).collect();

                    let result = AppendMenuW(
                        menu,
                        MF_STRING,
                        menu_id as usize,
                        windows::core::PCWSTR(wide_label.as_ptr()),
                    );

                    if !result.as_bool() {
                        DestroyMenu(menu);
                        return Err(format!("Failed to add menu item: {}", label));
                    }

                    callback_map.insert(menu_id, value.clone());
                    log::debug!("Added menu item {}: {} (ID: {})", index, label, menu_id);
                }

                // Get window handle for menu positioning
                let hwnd = if let Ok(windows) = self.windows.lock() {
                    if let Some(window) = windows.values().next() {
                        window.hwnd.map(|h| HWND(h as isize))
                    } else {
                        None
                    }
                } else {
                    None
                };

                let hwnd = hwnd.unwrap_or(HWND(0)); // Use desktop window if no app window

                // Show context menu at coordinates
                let selected = TrackPopupMenu(
                    menu,
                    TPM_RETURNCMD | TPM_NONOTIFY | TPM_LEFTBUTTON,
                    coordinates.0,
                    coordinates.1,
                    0,
                    hwnd,
                    None,
                );

                // Handle selection and execute callback
                if selected != 0 {
                    if let Some(value) = callback_map.get(&(selected as u32)) {
                        log::info!("Context menu item selected: ID {}", selected);
                        callback(value.clone());
                    } else {
                        log::warn!("Unknown menu item selected: ID {}", selected);
                    }
                } else {
                    log::debug!("Context menu dismissed without selection");
                }

                // Clean up menu resources
                DestroyMenu(menu);
                Ok(())
            }
        }

        #[cfg(not(target_os = "windows"))]
        {
            log::warn!("Context menus only available on Windows platform");
            Err("Context menus only available on Windows platform".to_string())
        }
    }
}
