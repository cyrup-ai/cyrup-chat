//! Core Windows platform definitions and structures
//!
//! This module contains the fundamental data structures and basic setup
//! functions for Windows platform integration.

use crate::environment::{
    storage::UiTab,
    types::{ActionFromEvent, AppEvent, MainMenuConfig},
};
use dioxus_desktop::{LogicalSize, WindowBuilder};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Windows-specific window handle with Win32 integration
pub struct AppWindow {
    pub hwnd: Option<isize>, // HWND handle for Win32 API calls
    pub window_id: u64,
    pub title: String,
    pub is_fullscreen: Arc<Mutex<bool>>,
    pub notifications: Arc<Mutex<Vec<WindowsNotification>>>,
}

impl Default for AppWindow {
    fn default() -> Self {
        Self {
            hwnd: None,
            window_id: 0,
            title: "CYRUP".to_string(),
            is_fullscreen: Arc::new(Mutex::new(false)),
            notifications: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowsNotification {
    pub id: u32,
    pub title: String,
    pub message: String,
    pub timestamp: std::time::SystemTime,
    pub toast_id: Option<String>, // Windows Toast notification ID
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
