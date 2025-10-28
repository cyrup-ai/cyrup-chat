//! Action handling for macOS platform with zero-allocation patterns
//!
//! This module provides comprehensive action handling for the macOS platform
//! including clipboard operations, URL navigation, and platform integrations.

/// Handle public actions with platform-specific implementations
#[inline(always)]
pub async fn handle_public_action(
    action: crate::PublicAction,
    instance_url: &str,
) -> Result<(), String> {
    log::debug!("Handling public action: {action:?}");

    match action {
        crate::PublicAction::OpenLink(url) => {
            navigate_to_url(&url)?;
        }
        crate::PublicAction::Copy(text) => {
            copy_to_clipboard(&text)?;
        }
        crate::PublicAction::OpenProfile(profile) => {
            log::debug!("Opening profile: {:?}", profile.username);

            // Construct profile URL using current instance URL
            let profile_url = format!("{}/@{}", instance_url, profile.username);
            navigate_to_url(&profile_url)?;

            log::debug!("Opened profile URL: {}", profile_url);
        }
        crate::PublicAction::OpenTag(tag) => {
            log::debug!("Opening tag: {tag}");

            // Construct tag URL using current instance URL
            let tag_url = format!("{}/tags/{}", instance_url, tag);
            navigate_to_url(&tag_url)?;
        }
        _ => {
            log::debug!("Unhandled public action: {action:?}");
        }
    }

    Ok(())
}

/// Navigate to URL using platform-specific methods
#[inline(always)]
fn navigate_to_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;
    }
    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("cmd")
            .args(&["/C", "start", url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;
    }
    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;
    }

    log::debug!("Navigated to URL: {url}");
    Ok(())
}

/// Copy text to clipboard with comprehensive error handling
#[inline(always)]
fn copy_to_clipboard(text: &str) -> Result<(), String> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        use copypasta::{ClipboardContext, ClipboardProvider};

        match ClipboardContext::new() {
            Ok(mut ctx) => {
                ctx.set_contents(text.to_string())
                    .map_err(|e| format!("Failed to set clipboard contents: {e}"))?;
                log::debug!("Copied {} characters to clipboard", text.len());
                Ok(())
            }
            Err(e) => {
                log::error!("Failed to create clipboard context: {e}");
                Err(format!("Clipboard operation failed: {e}"))
            }
        }
    }
    #[cfg(target_arch = "wasm32")]
    {
        log::warn!("Clipboard operations not supported on WASM");
        Err("Clipboard not supported on WASM".to_string())
    }
}

/// Show the emoji picker/character palette on macOS
#[inline(always)]
pub fn show_emoji_popup() -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        use objc2::MainThreadMarker;
        use objc2_app_kit::NSApplication;

        // Get main thread marker (required for UI operations)
        let mtm = MainThreadMarker::new().expect("must be on main thread");

        // Get shared application instance
        let app = NSApplication::sharedApplication(mtm);

        // Show character palette (orderFrontCharacterPalette is already unsafe)
        app.orderFrontCharacterPalette(None);

        log::debug!("Character palette opened using objc2 NSApplication API");
        Ok(())
    }
    #[cfg(not(target_os = "macos"))]
    {
        Err("Emoji popup not supported on this platform".to_string())
    }
}

/// Show a context menu with the given items and coordinates
#[inline(always)]
pub fn show_context_menu<T: Clone + std::fmt::Debug + Send + 'static>(
    coordinates: (i32, i32),
    title: &str,
    items: Vec<(&str, T)>,
    callback: impl Fn(T) + 'static,
) {
    #[cfg(target_os = "macos")]
    {
        use objc2::rc::Retained;
        use objc2::runtime::NSObject;
        use objc2::{MainThreadMarker, class, msg_send, msg_send_id};
        use objc2_app_kit::NSApplication;
        use objc2_foundation::{NSPoint, NSString};

        if items.is_empty() {
            log::warn!("Context menu requested but no items provided");
            return;
        }

        log::debug!(
            "Creating NSMenu context menu at {:?}: {} with {} items",
            coordinates,
            title,
            items.len()
        );

        unsafe {
            // Create NSMenu with title
            let title_ns = NSString::from_str(title);
            let menu: Retained<NSObject> = msg_send_id![
                msg_send_id![class!(NSMenu), alloc],
                initWithTitle: &*title_ns
            ];

            // Create empty string for key equivalent
            let empty_ns = NSString::from_str("");

            // Add menu items with tags for identification
            for (i, (label, _action)) in items.iter().enumerate() {
                let label_ns = NSString::from_str(label);
                let item: Retained<NSObject> = msg_send_id![
                    msg_send_id![class!(NSMenuItem), alloc],
                    initWithTitle: &*label_ns,
                    action: std::ptr::null::<u8>(),
                    keyEquivalent: &*empty_ns
                ];

                // Set the tag to identify this item (void return, use msg_send!)
                let () = msg_send![&*item, setTag: i as isize];

                // Add item to menu (void return, use msg_send!)
                let () = msg_send![&*menu, addItem: &*item];
            }

            // Get the main window for menu positioning
            let mtm = MainThreadMarker::new().expect("must be on main thread");
            let app = NSApplication::sharedApplication(mtm);
            let main_window: Option<Retained<NSObject>> = msg_send![&*app, mainWindow];

            if let Some(window) = main_window {
                // Convert coordinates to NSPoint (from objc2_foundation)
                let point = NSPoint {
                    x: coordinates.0 as f64,
                    y: coordinates.1 as f64,
                };

                // Get content view
                let content_view: Retained<NSObject> = msg_send_id![&*window, contentView];

                // Show context menu at coordinates
                // popUpMenuPositioningItem not in objc2-app-kit, use msg_send!
                let () = msg_send![
                    &*menu,
                    popUpMenuPositioningItem: std::ptr::null::<u8>(),
                    atLocation: point,
                    inView: &*content_view
                ];

                log::debug!("NSMenu context menu displayed successfully");

                // Check which item was selected
                let selected_item: Option<Retained<NSObject>> = msg_send![&*menu, selectedItem];
                if let Some(item) = selected_item {
                    let selected_index: isize = msg_send![&*item, tag];
                    if selected_index >= 0 && (selected_index as usize) < items.len() {
                        let (_label, action) = &items[selected_index as usize];
                        callback(action.clone());
                        log::debug!(
                            "Executed callback for selected menu item: {}",
                            selected_index
                        );
                    } else {
                        log::warn!("Invalid selected item tag: {}", selected_index);
                    }
                } else {
                    log::debug!("No menu item was selected");
                }
            } else {
                log::error!("No main window available for context menu");
            }
        }
    }
    #[cfg(not(target_os = "macos"))]
    {
        log::warn!("Context menus not supported on this platform");
        // Fallback: just call callback with first item
        if let Some((first_label, first_action)) = items.into_iter().next() {
            log::debug!("Fallback: executing first menu item: {}", first_label);
            callback(first_action);
        }
    }
}
