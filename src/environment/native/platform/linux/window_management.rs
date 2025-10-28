use super::core::{AppWindow, Platform};
use std::process::Command;

impl Platform {
    /// Check if window is in fullscreen mode
    pub fn is_fullscreen(&self) -> Result<bool, String> {
        use gtk4::prelude::*;

        log::debug!("Checking fullscreen state using GTK4 API");

        // Get all toplevel GTK windows and check if any are fullscreen
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(window) = toplevels.item(i) {
                if let Ok(gtk_window) = window.downcast::<gtk4::Window>() {
                    if gtk_window.is_fullscreen() {
                        log::debug!("Found fullscreen GTK4 window");
                        return Ok(true);
                    }
                }
            }
        }

        log::debug!("No fullscreen GTK4 windows found");
        Ok(false)
    }

    /// Open a new window with specified dimensions
    pub fn open_window(&self, title: &str, width: u32, height: u32) -> Result<u64, String> {
        log::debug!("Opening new Linux window: {} ({}x{})", title, width, height);

        let window_id = {
            let mut counter = self
                .notification_counter
                .lock()
                .map_err(|e| format!("Failed to lock notification counter: {}", e))?;
            *counter += 1;
            *counter as u64
        };

        let window = AppWindow {
            window_id,
            title: title.to_string(),
            is_fullscreen: std::sync::Arc::new(std::sync::Mutex::new(false)),
            notifications: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
        };

        // Store the window reference
        {
            let mut windows = self
                .windows
                .lock()
                .map_err(|e| format!("Failed to lock windows: {}", e))?;
            windows.insert(window_id, window);
        }

        log::info!("Linux window opened: {} (ID: {})", title, window_id);
        Ok(window_id)
    }

    /// Navigate to URL using Linux default browser
    pub fn navigate(&self, path: &str) -> Result<(), String> {
        log::debug!("Navigating to URL on Linux: {}", path);

        let _result = Command::new("xdg-open")
            .arg(path)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {e}"))?;

        log::info!("URL opened successfully: {}", path);
        Ok(())
    }

    // Scrolling and DOM manipulation
    pub async fn scroll_to_element(&self, element_id: &str) -> Result<(), String> {
        log::debug!("Scrolling to element: {}", element_id);

        let error_msg = format!(
            "DOM scrolling requires component context - use document::eval() in Dioxus components for element: {}",
            element_id
        );
        log::warn!("{}", error_msg);
        Err(error_msg)
    }

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

        let error_msg = format!(
            "DOM scrolling requires component context - use document::eval() in Dioxus components for element: {} with behavior: {}",
            element_id, behavior
        );
        log::warn!("{}", error_msg);
        Err(error_msg)
    }

    // Window management
    pub fn close_preferences_window(&self) -> Result<(), String> {
        use gtk4::prelude::*;

        log::debug!("Closing preferences window using GTK4 API");

        // Find and close preferences window by title or CSS class
        let toplevels = gtk4::Window::toplevels();
        for i in 0..toplevels.n_items() {
            if let Some(window) = toplevels.item(i) {
                if let Ok(gtk_window) = window.downcast::<gtk4::Window>() {
                    // Check if this is a preferences window by title
                    if let Some(title) = gtk_window.title() {
                        if title.to_lowercase().contains("preferences")
                            || title.to_lowercase().contains("settings")
                        {
                            gtk_window.close();
                            log::info!("Closed preferences window: {}", title);
                            return Ok(());
                        }
                    }

                    // Check by CSS class
                    if gtk_window.has_css_class("preferences-window")
                        || gtk_window.has_css_class("settings-window")
                    {
                        gtk_window.close();
                        log::info!("Closed preferences window by CSS class");
                        return Ok(());
                    }
                }
            }
        }

        log::debug!("No preferences window found to close");
        Ok(())
    }
}
