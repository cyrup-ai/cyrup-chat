//! Core Platform struct and basic functionality for macOS
//!
//! This module provides the core Platform struct with its fields and basic methods.

use cacao::appkit::toolbar::Toolbar;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use super::super::{
    super::super::super::types::{ActionFromEvent, MainMenuConfig},
    super::super::toolbar::{LoggedInToolbar, LoggedOutBar},
    menu::create_main_menu,
};
use crate::environment::types::AppEvent;

#[allow(dead_code)] // Toolbar type variants - pending macOS toolbar integration
#[derive(Default)]
pub enum ToolbarType {
    #[default]
    NoneYet,
    LoggedOut(Toolbar<LoggedOutBar>),
    LoggedIn(Toolbar<LoggedInToolbar>),
}

pub type ToolbarHandlerUpdateCell = Arc<Mutex<Option<Arc<dyn Fn(AppEvent) + Send + Sync>>>>;

#[derive(Clone)]
pub struct Platform {
    /// The current Menu Configuration
    pub content: Arc<Mutex<MainMenuConfig>>,
    /// The current toolbar (main thread only - macOS UI types are not Send/Sync)
    pub toolbar: Rc<RefCell<ToolbarType>>,
    /// The handler for the current toolbar
    pub toolbar_handler: ToolbarHandlerUpdateCell,
}

impl Default for Platform {
    fn default() -> Self {
        let platform = Self {
            content: Arc::new(Mutex::new(MainMenuConfig::default())),
            toolbar: Rc::new(RefCell::new(ToolbarType::default())),
            toolbar_handler: Arc::new(Mutex::new(None)),
        };

        // ✅ Initialize menu immediately for macOS NSApp
        #[cfg(target_os = "macos")]
        {
            let config = MainMenuConfig::default();
            let menu = create_main_menu(config);
            menu.init_for_nsapp();
            log::debug!("Initial menu attached to NSApp");
        }

        platform
    }
}

impl Platform {
    /// Update menu configuration with safe error handling
    #[inline(always)]
    pub fn update_menu(&self, mutator: impl Fn(&mut MainMenuConfig)) {
        if let Ok(mut config) = self.content.lock() {
            mutator(&mut config);
            let config_clone = *config;
            let menu = create_main_menu(config_clone);

            // ✅ Actually attach menu to NSApp (macOS-specific)
            #[cfg(target_os = "macos")]
            {
                menu.init_for_nsapp();
                log::debug!("Menu updated and attached to NSApp");
            }

            #[cfg(not(target_os = "macos"))]
            {
                log::debug!("Menu configuration updated (non-macOS platform)");
            }
        } else {
            log::error!("Failed to acquire menu configuration lock");
        }
    }

    /// Handle menu events with generic action conversion
    #[inline(always)]
    pub fn handle_menu_events<A: ActionFromEvent + 'static>(
        &self,
        updater: Arc<dyn Fn(A) + Send + Sync>,
    ) {
        use dioxus_desktop::{tao::event::WindowEvent, use_wry_event_handler};

        use_wry_event_handler(move |event, _target| match event {
            dioxus_desktop::tao::event::Event::WindowEvent {
                event: WindowEvent::Focused(a),
                ..
            } => {
                let Some(converted) = A::make_focus_event(*a) else {
                    return;
                };
                updater(converted);
            }
            dioxus_desktop::tao::event::Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                let Some(converted) = A::make_close_window_event() else {
                    return;
                };
                updater(converted);
            }
            _ => (),
        });
    }

    /// Execute JavaScript using Dioxus document::eval API
    #[inline(always)]
    pub async fn execute_javascript(&self, script: &str) -> Result<serde_json::Value, String> {
        use dioxus::document;

        log::debug!("Executing JavaScript: {}", script);

        let eval = document::eval(script);
        match eval.await {
            Ok(result) => {
                log::debug!("JavaScript executed successfully: {:?}", result);
                Ok(result)
            }
            Err(e) => {
                log::error!("JavaScript execution failed: {}", e);
                Err(e.to_string())
            }
        }
    }
}
