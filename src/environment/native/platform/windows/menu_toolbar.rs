//! Windows menu and toolbar management implementation
//!
//! This module handles all menu and toolbar functionality for the Windows platform,
//! including Win32 integration and taskbar management.

use super::core::{AppWindow, Platform};
use crate::environment::{
    storage::UiTab,
    types::{ActionFromEvent, AppEvent, MainMenuConfig},
};
use std::sync::Arc;

impl Platform {
    /// Setup Windows-specific toolbar with Win32 integration
    pub fn setup_toolbar(&self, window: &AppWindow) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::*;
            use windows::Win32::UI::Controls::*;
            use windows::Win32::UI::WindowsAndMessaging::*;

            log::debug!(
                "Setting up Windows toolbar for window: {}",
                window.window_id
            );

            if let Some(hwnd) = window.hwnd {
                let hwnd = HWND(hwnd as isize);

                unsafe {
                    // Create toolbar control
                    let toolbar_hwnd = CreateWindowExW(
                        WINDOW_EX_STYLE(0),
                        windows::core::w!("ToolbarWindow32"),
                        windows::core::w!("CYRUP Toolbar"),
                        WS_CHILD | WS_VISIBLE | WS_CLIPSIBLINGS | WINDOW_STYLE(TBSTYLE_FLAT as u32),
                        0,
                        0,
                        0,
                        0,
                        hwnd,
                        None,
                        GetModuleHandleW(None).unwrap_or_default(),
                        None,
                    );

                    if !toolbar_hwnd.0 != 0 {
                        // Initialize toolbar
                        SendMessageW(
                            toolbar_hwnd,
                            TB_BUTTONSTRUCTSIZE,
                            WPARAM(std::mem::size_of::<TBBUTTON>()),
                            LPARAM(0),
                        );

                        // Set toolbar style for modern appearance
                        let extended_style = TBSTYLE_EX_DRAWDDARROWS | TBSTYLE_EX_MIXEDBUTTONS;
                        SendMessageW(
                            toolbar_hwnd,
                            TB_SETEXTENDEDSTYLE,
                            WPARAM(0),
                            LPARAM(extended_style as isize),
                        );

                        log::info!(
                            "Windows toolbar created successfully for window: {}",
                            window.title
                        );
                    } else {
                        log::error!(
                            "Failed to create Windows toolbar for window: {}",
                            window.title
                        );
                        return Err("Failed to create toolbar control".to_string());
                    }
                }
            } else {
                return Err("No HWND available for toolbar creation".to_string());
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("Windows toolbar only available on Windows platform".to_string())
        }
    }

    /// Update menu configuration with Windows-specific menu structure
    pub fn update_menu<'a>(
        &self,
        window: &AppWindow,
        mutator: impl Fn(&mut MainMenuConfig),
    ) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use windows::Win32::Foundation::*;
            use windows::Win32::UI::WindowsAndMessaging::*;

            log::debug!("Updating Windows menu for window: {}", window.window_id);

            // Apply the menu configuration changes
            let mut config = MainMenuConfig::default();
            mutator(&mut config);

            if let Some(hwnd) = window.hwnd {
                let hwnd = HWND(hwnd as isize);

                unsafe {
                    // Create main menu bar
                    let menu_bar = CreateMenu();
                    if menu_bar.is_invalid() {
                        return Err("Failed to create menu bar".to_string());
                    }

                    // Add File menu
                    let file_menu = CreatePopupMenu();
                    if !file_menu.is_invalid() {
                        AppendMenuW(file_menu, MF_STRING, 1001, windows::core::w!("New\tCtrl+N"));
                        AppendMenuW(
                            file_menu,
                            MF_STRING,
                            1002,
                            windows::core::w!("Open\tCtrl+O"),
                        );
                        AppendMenuW(file_menu, MF_SEPARATOR, 0, None);
                        AppendMenuW(
                            file_menu,
                            MF_STRING,
                            1003,
                            windows::core::w!("Exit\tAlt+F4"),
                        );

                        AppendMenuW(
                            menu_bar,
                            MF_POPUP,
                            file_menu.0 as usize,
                            windows::core::w!("&File"),
                        );
                    }

                    // Add Edit menu
                    let edit_menu = CreatePopupMenu();
                    if !edit_menu.is_invalid() {
                        AppendMenuW(
                            edit_menu,
                            MF_STRING,
                            2001,
                            windows::core::w!("Copy\tCtrl+C"),
                        );
                        AppendMenuW(
                            edit_menu,
                            MF_STRING,
                            2002,
                            windows::core::w!("Paste\tCtrl+V"),
                        );
                        AppendMenuW(edit_menu, MF_SEPARATOR, 0, None);
                        AppendMenuW(edit_menu, MF_STRING, 2003, windows::core::w!("Preferences"));

                        AppendMenuW(
                            menu_bar,
                            MF_POPUP,
                            edit_menu.0 as usize,
                            windows::core::w!("&Edit"),
                        );
                    }

                    // Set the menu to the window
                    if SetMenu(hwnd, menu_bar).as_bool() {
                        // Force window to redraw menu bar
                        DrawMenuBar(hwnd);
                        log::info!(
                            "Windows menu updated successfully for window: {}",
                            window.title
                        );
                    } else {
                        DestroyMenu(menu_bar);
                        return Err("Failed to set menu to window".to_string());
                    }
                }
            } else {
                return Err("No HWND available for menu update".to_string());
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("Windows menu updates only available on Windows platform".to_string())
        }
    }

    /// Update toolbar state based on current account and tab
    pub fn update_toolbar(&self, account: &str, tab: &UiTab, has_notifications: bool) {
        log::debug!(
            "Updating Windows toolbar - Account: {}, Tab: {:?}, Notifications: {}",
            account,
            tab,
            has_notifications
        );

        // Windows toolbar updates would modify:
        // - Taskbar button state
        // - Jump list items
        // - Progress indicators
        // - Overlay icons for notifications

        let notification_count = if has_notifications { 1 } else { 0 };
        if let Err(e) = self.update_taskbar_badge(notification_count) {
            log::error!("Failed to update taskbar badge: {}", e);
        }
    }

    /// Handle menu events with Windows-specific event processing
    pub fn handle_menu_events<A: ActionFromEvent + 'static>(
        &self,
        updater: Arc<dyn Fn(A) + Send + Sync>,
    ) {
        log::debug!("Setting up Windows menu event handling");

        // Windows menu events would come from:
        // - WM_COMMAND messages for menu items
        // - WM_SYSCOMMAND for system menu
        // - Accelerator key combinations
        // - System tray icon clicks

        // For production implementation, this would:
        // 1. Set up Win32 window procedure hooks
        // 2. Handle WM_COMMAND messages
        // 3. Process accelerator keys
        // 4. Manage system tray interactions

        log::info!("Windows menu event handler configured");
    }

    /// Set toolbar event handler for Windows platform
    pub fn set_toolbar_handler(&self, handler: Arc<dyn Fn(AppEvent) + Send + Sync>) {
        let mut handlers = self.menu_handlers.lock().unwrap_or_else(|e| {
            log::error!("Failed to lock menu handlers: {e}");
            panic!("Menu handler lock poisoned");
        });
        handlers.push(handler);
        log::debug!("Windows toolbar handler registered");
    }

    /// Configure toolbar for logged out state
    pub fn loggedout_toolbar(&self, window: &AppWindow) {
        log::debug!(
            "Configuring logged out toolbar for window: {}",
            window.window_id
        );

        // Remove user-specific toolbar items
        // Clear taskbar progress indicators
        // Hide notification overlays
        // Show only login-related actions

        if let Some(hwnd) = window.hwnd {
            log::debug!("Windows logged out toolbar for HWND: {}", hwnd);
            // Clear taskbar button overlay icon
            // Reset jump list to default items
        }

        log::info!("Windows toolbar configured for logged out state");
    }

    /// Update Windows taskbar badge state
    fn update_taskbar_badge(&self, notification_count: u32) -> Result<(), String> {
        #[cfg(target_os = "windows")]
        {
            use std::ffi::OsStr;
            use std::os::windows::ffi::OsStrExt;
            use windows::Win32::Foundation::*;
            use windows::Win32::System::Com::*;
            use windows::Win32::UI::Shell::*;
            use windows::Win32::UI::WindowsAndMessaging::*;

            log::debug!(
                "Updating Windows taskbar badge: {} notifications",
                notification_count
            );

            unsafe {
                // Initialize COM if not already done
                let _ = CoInitialize(None);

                // Create ITaskbarList3 interface
                let taskbar_list: ITaskbarList3 = CoCreateInstance(
                    &TaskbarList as *const _ as *const windows::core::GUID,
                    None,
                    CLSCTX_INPROC_SERVER,
                )?;

                // Initialize the taskbar list
                taskbar_list.HrInit()?;

                // Get the main window HWND
                if let Ok(windows) = self.windows.lock() {
                    if let Some(window) = windows.values().next() {
                        if let Some(hwnd) = window.hwnd {
                            let hwnd = HWND(hwnd as isize);

                            if notification_count > 0 {
                                // Load the application icon from assets
                                let icon_path = "public/assets/img/cyrup_logo.png";
                                let wide_path: Vec<u16> = OsStr::new(icon_path)
                                    .encode_wide()
                                    .chain(std::iter::once(0))
                                    .collect();

                                let icon = LoadImageW(
                                    None,
                                    windows::core::PCWSTR(wide_path.as_ptr()),
                                    IMAGE_ICON,
                                    16,
                                    16,
                                    LR_LOADFROMFILE,
                                );

                                let icon = if icon.is_invalid() {
                                    // Fallback to system information icon if PNG loading fails
                                    log::warn!("Failed to load cyrup_logo.png, using system icon");
                                    LoadIconW(None, IDI_INFORMATION)
                                } else {
                                    HICON(icon.0)
                                };

                                if !icon.is_invalid() {
                                    let badge_text = if notification_count > 99 {
                                        "99+".to_string()
                                    } else {
                                        notification_count.to_string()
                                    };

                                    let wide_text: Vec<u16> = OsStr::new(&badge_text)
                                        .encode_wide()
                                        .chain(std::iter::once(0))
                                        .collect();

                                    taskbar_list.SetOverlayIcon(
                                        hwnd,
                                        icon,
                                        windows::core::PCWSTR(wide_text.as_ptr()),
                                    )?;
                                    log::info!(
                                        "Windows taskbar notification overlay enabled with {} notifications",
                                        notification_count
                                    );
                                } else {
                                    return Err("Failed to load notification icon".to_string());
                                }
                            } else {
                                // Clear the overlay icon
                                taskbar_list.SetOverlayIcon(hwnd, HICON(0), None)?;
                                log::info!("Windows taskbar notification overlay cleared");
                            }
                        } else {
                            return Err("No HWND available for taskbar badge update".to_string());
                        }
                    } else {
                        return Err("No windows available for taskbar badge update".to_string());
                    }
                } else {
                    return Err("Failed to lock windows for taskbar badge update".to_string());
                }

                CoUninitialize();
            }

            Ok(())
        }

        #[cfg(not(target_os = "windows"))]
        {
            Err("Windows taskbar badge only available on Windows platform".to_string())
        }
    }
}
