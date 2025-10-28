//! Core Environment struct and basic functionality

use crate::database::Database;
use crate::environment::OpenWindowState;
use crate::environment::storage::Data;
use dioxus::prelude::*;
use std::rc::Rc;
use std::sync::Arc;

pub use super::model::Model;
pub use super::platform;
pub use super::settings::Settings;

#[derive(Clone)]
pub struct Environment {
    /// Database connection for conversation/message operations
    pub database: Arc<Database>,

    /// Model layer - agent manager and high-level operations
    pub model: Model,

    /// Settings - user preferences and UI configuration
    pub settings: Settings,

    /// Platform-specific operations (windows, menus, etc.)
    pub platform: platform::Platform,

    /// Reactive storage for UI state
    pub storage: Signal<Data>,
}

impl std::fmt::Debug for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Environment").finish()
    }
}

impl PartialEq for Environment {
    fn eq(&self, _other: &Self) -> bool {
        // For component equality, we just return true since Environment
        // is essentially a singleton service container
        true
    }
}

impl Environment {
    /// Create default window configuration for macOS
    #[cfg(target_os = "macos")]
    pub fn macos_window() -> dioxus_desktop::WindowBuilder {
        platform::default_window()
    }

    /// Create new Environment with initialized database
    ///
    /// # Arguments
    /// * `database` - Arc-wrapped database connection
    /// * `model` - Model instance created with database reference
    /// * `settings` - Settings for user preferences
    pub fn new(database: Arc<Database>, model: Model, settings: Settings) -> Self {
        Self {
            database,
            model,
            settings,
            platform: platform::Platform::default(),
            storage: Signal::new(Data::default()),
        }
    }

    pub fn update_model(&mut self, model: Model) {
        self.model = model;
    }

    pub fn open_url(&self, url: &str) {
        let _ = webbrowser::open(url);
    }

    pub fn open_window<S: OpenWindowState + 'static>(
        &self,
        _window: &super::platform::AppWindow,
        state: S,
        width: f64,
        height: f64,
        title: impl AsRef<str>,
        parent_handler: Rc<dyn Fn(S::Action)>,
    ) {
        super::window::open_window_impl(self, state, width, height, title, parent_handler);
    }
}
