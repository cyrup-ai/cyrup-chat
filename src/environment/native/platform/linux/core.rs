use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use dioxus_desktop::{LogicalSize, WindowBuilder};

use crate::environment::{
    storage::UiTab,
    types::{ActionFromEvent, AppEvent, MainMenuConfig},
};

/// Linux-specific window handle with GTK integration
pub struct AppWindow {
    pub window_id: u64,
    pub title: String,
    pub is_fullscreen: Arc<Mutex<bool>>,
    pub notifications: Arc<Mutex<Vec<LinuxNotification>>>,
}

impl Default for AppWindow {
    fn default() -> Self {
        Self {
            window_id: 0,
            title: "CYRUP".to_string(),
            is_fullscreen: Arc::new(Mutex::new(false)),
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LinuxNotification {
    pub id: u32,
    pub title: String,
    pub message: String,
    pub timestamp: std::time::SystemTime,
}

pub fn default_window() -> WindowBuilder {
    let builder = WindowBuilder::new();
    let s = LogicalSize::new(1200., 775.);

    let builder = builder
        .with_title("CYRUP")
        .with_theme(Some(dioxus_desktop::tao::window::Theme::Dark))
        .with_inner_size(s);
    builder
}

#[derive(Clone)]
pub struct Platform {
    pub windows: Arc<Mutex<HashMap<u64, AppWindow>>>,
    pub notification_counter: Arc<Mutex<u32>>,
    pub menu_handlers: Arc<Mutex<Vec<Arc<dyn Fn(AppEvent) + Send + Sync>>>>,
}

impl Default for Platform {
    fn default() -> Self {
        Self {
            windows: Arc::new(Mutex::new(HashMap::new())),
            notification_counter: Arc::new(Mutex::new(0)),
            menu_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

pub fn apply_window_background<'a>(_window: &AppWindow) -> Result<(), String> {
    use gtk4::prelude::*;
    use gtk4::{CssProvider, gdk};

    log::debug!("Applying window background styling for Linux using GTK4");

    // Create CSS provider for window background styling
    let css_provider = CssProvider::new();

    // Define window background CSS with transparency and styling
    let window_css = "
        window {
            background-color: rgba(0, 0, 0, 0.85);
            border-radius: 8px;
        }
        
        window.transparent {
            background-color: rgba(0, 0, 0, 0.7);
        }
        
        window.blur-background {
            background-color: rgba(30, 30, 30, 0.8);
            backdrop-filter: blur(10px);
        }
    ";

    css_provider.load_from_string(window_css);

    // Apply CSS to the default display
    if let Some(display) = gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &css_provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );

        log::info!("Window background CSS applied to GTK4 display");
    } else {
        return Err("Could not get default GTK4 display".to_string());
    }

    // Apply additional window styling if we can access GTK windows
    // Note: This applies to all GTK windows in the application
    let toplevels = gtk4::Window::toplevels();
    for i in 0..toplevels.n_items() {
        if let Some(window) = toplevels.item(i) {
            if let Ok(gtk_window) = window.downcast::<gtk4::Window>() {
                // Apply transparency and CSS classes
                gtk_window.set_opacity(0.95);
                gtk_window.add_css_class("blur-background");

                log::debug!("Applied background styling to GTK4 window");
            }
        }
    }

    Ok(())
}
