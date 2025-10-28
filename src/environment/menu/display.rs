//! Platform-specific menu display implementations

use super::structures::ContextMenu;
use super::types::CTX_STATE;
use dioxus::prelude::*;
use muda::{ContextMenu as MudaContextMenu, dpi::Position};
use std::any::Any;
use std::collections::HashMap;

/// Error types for menu operations
#[derive(Debug, thiserror::Error)]
pub enum MenuError {
    #[error("Platform not supported: {0}")]
    PlatformNotSupported(&'static str),
    #[error("Window handle unavailable")]
    WindowHandleUnavailable,
    #[error("Menu display failed: {0}")]
    MenuDisplayFailed(String),
    #[error("Native window operation failed: {0}")]
    NativeWindowOperationFailed(String),
    #[error("Menu handler not registered for action: {0}")]
    HandlerNotRegistered(String),
}

pub fn show_context_menu<A>(
    _window: &dyn Any,
    event: &MouseData,
    menu: ContextMenu<A>,
    action_key: String,
) -> Result<(), MenuError> {
    use muda::Submenu;

    let mut context_menu = Submenu::new(menu.title, menu.enabled);
    let mut actions = HashMap::new();
    for child in menu.children {
        child.build(&mut context_menu, &mut actions);
    }

    if let Ok(mut t) = CTX_STATE.write() {
        let Some(entry) = t.get_mut(&action_key) else {
            log::warn!(
                "setup_menu_handler was not called for action {action_key}. No handler registered"
            );
            return Err(MenuError::HandlerNotRegistered(action_key));
        };
        *entry = actions;
    }

    #[cfg(target_os = "macos")]
    show_macos_menu(event, context_menu)?;

    #[cfg(target_os = "windows")]
    show_windows_menu(event, context_menu)?;

    #[cfg(target_os = "linux")]
    show_linux_menu(event, context_menu)?;

    Ok(())
}

#[cfg(target_os = "macos")]
fn show_macos_menu(event: &MouseData, context_menu: muda::Submenu) -> Result<(), MenuError> {
    let menu: &dyn MudaContextMenu = &context_menu;
    let position = Position::Physical(muda::dpi::PhysicalPosition::new(
        event.screen_coordinates().x as i32,
        event.screen_coordinates().y as i32,
    ));

    // Get native NSView handle using Dioxus desktop API
    let window_handle = dioxus::desktop::window();

    #[cfg(target_os = "macos")]
    {
        use dioxus_desktop::tao::platform::macos::WindowExtMacOS;
        let native_view = window_handle.window.ns_view() as *const std::ffi::c_void;

        if native_view.is_null() {
            return Err(MenuError::WindowHandleUnavailable);
        }

        // Show context menu using muda API
        let success = unsafe { menu.show_context_menu_for_nsview(native_view, Some(position)) };

        if success {
            log::debug!(
                "macOS context menu displayed successfully at position {:?}",
                position
            );
            Ok(())
        } else {
            Err(MenuError::MenuDisplayFailed(
                "Failed to show macOS context menu".to_string(),
            ))
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        Err(MenuError::PlatformNotSupported(
            "macOS context menu on non-macOS platform",
        ))
    }
}

#[cfg(target_os = "windows")]
fn show_windows_menu(event: &MouseData, context_menu: muda::Submenu) -> Result<(), MenuError> {
    let menu: &dyn MudaContextMenu = &context_menu;
    let position = Position::Physical(muda::dpi::PhysicalPosition::new(
        event.screen_coordinates().x as i32,
        event.screen_coordinates().y as i32,
    ));

    // Get HWND using Dioxus desktop API
    let window_handle = dioxus::desktop::window();

    #[cfg(target_os = "windows")]
    {
        use dioxus_desktop::tao::platform::windows::WindowExtWindows;
        let hwnd = window_handle.window.hwnd() as isize;

        if hwnd == 0 {
            return Err(MenuError::WindowHandleUnavailable);
        }

        // Show context menu using muda API
        let success = unsafe { menu.show_context_menu_for_hwnd(hwnd, Some(position)) };

        if success {
            log::debug!(
                "Windows context menu displayed successfully at position {:?}",
                position
            );
            Ok(())
        } else {
            Err(MenuError::MenuDisplayFailed(
                "Failed to show Windows context menu".to_string(),
            ))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err(MenuError::PlatformNotSupported(
            "Windows context menu on non-Windows platform",
        ))
    }
}

#[cfg(target_os = "linux")]
fn show_linux_menu(event: &MouseData, context_menu: muda::Submenu) -> Result<(), MenuError> {
    let menu: &dyn MudaContextMenu = &context_menu;
    let position = Position::Physical(muda::dpi::PhysicalPosition::new(
        event.screen_coordinates().x as i32,
        event.screen_coordinates().y as i32,
    ));

    // Get GTK window using Dioxus desktop API
    let window_handle = dioxus::desktop::window();

    #[cfg(target_os = "linux")]
    {
        use dioxus_desktop::tao::platform::unix::WindowExtUnix;
        let gtk_window = window_handle.window.gtk_window();

        // Show context menu using muda API
        let success = menu.show_context_menu_for_gtk_window(gtk_window.as_ref(), Some(position));

        if success {
            log::debug!(
                "Linux context menu displayed successfully at position {:?}",
                position
            );
            Ok(())
        } else {
            Err(MenuError::MenuDisplayFailed(
                "Failed to show Linux context menu".to_string(),
            ))
        }
    }

    #[cfg(not(target_os = "linux"))]
    {
        Err(MenuError::PlatformNotSupported(
            "Linux context menu on non-Linux platform",
        ))
    }
}
