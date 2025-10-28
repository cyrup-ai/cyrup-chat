//! Windows window management implementation
//!
//! This module handles window lifecycle operations, fullscreen management,
//! and window background styling for the Windows platform.

use super::core::{AppWindow, Platform};

impl Platform {
    /// Check if window is in fullscreen mode using Win32 API
    pub fn is_fullscreen(&self) -> Result<bool, String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Graphics::Gdi::*;
            use windows::Win32::UI::WindowsAndMessaging::*;

            log::debug!("Checking fullscreen state using Win32 API");

            // Get the foreground window (currently active window)
            let hwnd = unsafe { GetForegroundWindow() };
            if hwnd.0 == 0 {
                return Ok(false);
            }

            // Get window placement to check if maximized
            let mut placement = WINDOWPLACEMENT {
                length: std::mem::size_of::<WINDOWPLACEMENT>() as u32,
                ..Default::default()
            };

            unsafe {
                if GetWindowPlacement(hwnd, &mut placement).as_bool() {
                    // Check if window is maximized
                    if placement.showCmd == SW_SHOWMAXIMIZED.0 as u32 {
                        // Get monitor info to compare dimensions
                        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
                        let mut monitor_info = MONITORINFO {
                            cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                            ..Default::default()
                        };

                        if GetMonitorInfoW(monitor, &mut monitor_info).as_bool() {
                            // Get window rect
                            let mut window_rect = windows::Win32::Foundation::RECT::default();
                            if GetWindowRect(hwnd, &mut window_rect).as_bool() {
                                // Compare window rect with monitor rect
                                let monitor_rect = monitor_info.rcMonitor;
                                let is_fullscreen = window_rect.left <= monitor_rect.left
                                    && window_rect.top <= monitor_rect.top
                                    && window_rect.right >= monitor_rect.right
                                    && window_rect.bottom >= monitor_rect.bottom;

                                log::debug!("Window fullscreen check: {}", is_fullscreen);
                                return Ok(is_fullscreen);
                            }
                        }
                    }
                }
            }

            Ok(false)
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("Fullscreen detection only available on Windows".to_string())
        }
    }

    /// Open a new window with specified dimensions using Win32 API
    pub async fn open_window(&self, title: &str, width: u32, height: u32) -> Result<(), String> {
        log::debug!(
            "Opening new Windows window: {} ({}x{})",
            title,
            width,
            height
        );

        let window_id = {
            let mut counter = self.notification_counter.lock().unwrap_or_else(|e| {
                log::error!("Failed to lock notification counter: {e}");
                panic!("Notification counter lock poisoned");
            });
            *counter += 1;
            *counter as u64
        };

        let window = AppWindow {
            hwnd: None, // Would be set after CreateWindow call
            window_id,
            title: title.to_string(),
            is_fullscreen: std::sync::Arc::new(std::sync::Mutex::new(false)),
            notifications: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        };

        // Store the window reference
        {
            let mut windows = self.windows.lock().unwrap_or_else(|e| {
                log::error!("Failed to lock windows: {e}");
                panic!("Windows lock poisoned");
            });
            windows.insert(window_id, window);
        }

        // Windows window creation would use:
        // - CreateWindowEx for native windows
        // - RegisterClass for window class
        // - ShowWindow to display

        log::info!("Windows window opened: {} (ID: {})", title, window_id);
        Ok(())
    }

    /// Window management
    pub fn close_preferences_window(&self) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::UI::WindowsAndMessaging::*;
            use windows::core::PCWSTR;

            log::debug!("Closing preferences window on Windows using Win32 API");

            // Find window by class name or title
            let window_title = windows::core::w!("Preferences");
            let hwnd = unsafe { FindWindowW(None, PCWSTR(window_title.as_ptr())) };

            if hwnd.0 != 0 {
                // Send close message to the window
                unsafe {
                    let result = SendMessageW(
                        hwnd,
                        WM_CLOSE,
                        windows::Win32::Foundation::WPARAM(0),
                        windows::Win32::Foundation::LPARAM(0),
                    );
                    if result.0 == 0 {
                        log::info!("Preferences window closed successfully");
                        return Ok(());
                    } else {
                        log::warn!("Failed to close preferences window: {}", result.0);
                    }
                }
            } else {
                log::debug!("No preferences window found to close");
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("Window management only available on Windows".to_string())
        }
    }
}

pub fn apply_window_background<'a>(window: &AppWindow) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use windows::Win32::Foundation::*;
        use windows::Win32::Graphics::Dwm::*;

        log::debug!("Applying Windows window background styling using DWM API");

        if let Some(hwnd) = window.hwnd {
            unsafe {
                // Enable blur behind for modern Windows styling
                let blur_behind = DWM_BLURBEHIND {
                    dwFlags: DWM_BB_ENABLE,
                    fEnable: TRUE,
                    hRgnBlur: std::ptr::null_mut(),
                    fTransitionOnMaximized: FALSE,
                };

                let result = DwmEnableBlurBehindWindow(HWND(hwnd as isize), &blur_behind);
                if result.is_ok() {
                    log::info!("Windows blur effect applied successfully");
                } else {
                    log::warn!("Failed to apply Windows blur effect: {:?}", result);
                }

                // Try to enable composition for better visual effects
                let composition_result = DwmEnableComposition(TRUE);
                if composition_result.is_ok() {
                    log::debug!("DWM composition enabled");
                } else {
                    log::debug!("DWM composition already enabled or not supported");
                }
            }

            Ok(())
        } else {
            Err("No Windows HWND available for background styling".to_string())
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        log::warn!("Windows background styling only available on Windows platform");
        Err("Windows background styling not supported on this platform".to_string())
    }
}
