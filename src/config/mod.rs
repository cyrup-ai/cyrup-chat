/// Application configuration module implementing runtime environment detection
///
/// Following ./tmp/dioxus/examples/context_api.rs and global.rs patterns
/// This replaces compile-time conditionals with runtime configuration
use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    /// Debug mode enabled (from CYRUP_DEV_MODE environment variable)
    pub debug_mode: bool,
    /// Log level configuration (from RUST_LOG or defaults)
    pub log_level: String,
    /// Platform-specific features detected at runtime
    pub platform_features: PlatformFeatures,
    /// Error handling mode for production vs development
    pub error_handling: ErrorHandlingMode,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PlatformFeatures {
    /// Native toolbar support available
    pub native_toolbar: bool,
    /// Native file dialogs available  
    pub native_file_dialogs: bool,
    /// System theme detection available
    pub system_theme: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ErrorHandlingMode {
    /// Development mode - verbose errors and debugging
    Development,
    /// Production mode - graceful fallbacks and user-friendly messages
    Production,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self::detect()
    }
}

impl AppConfig {
    /// Detect runtime configuration from environment and system capabilities
    ///
    /// Reference: ./tmp/dioxus/examples/context_api.rs pattern
    pub fn detect() -> Self {
        let debug_mode = std::env::var("CYRUP_DEV_MODE").is_ok();
        let log_level = std::env::var("RUST_LOG").unwrap_or_else(|_| {
            if debug_mode {
                "debug".to_string()
            } else {
                "info".to_string()
            }
        });

        Self {
            debug_mode,
            log_level,
            platform_features: PlatformFeatures::detect(),
            error_handling: if debug_mode {
                ErrorHandlingMode::Development
            } else {
                ErrorHandlingMode::Production
            },
        }
    }

    /// Check if running in production mode (opposite of debug)
    pub fn is_production(&self) -> bool {
        !self.debug_mode
    }

    /// Check if verbose error reporting should be used
    pub fn use_verbose_errors(&self) -> bool {
        matches!(self.error_handling, ErrorHandlingMode::Development)
    }
}

impl PlatformFeatures {
    /// Detect platform capabilities at runtime
    pub fn detect() -> Self {
        Self {
            native_toolbar: cfg!(any(target_os = "macos", target_os = "windows")),
            native_file_dialogs: cfg!(not(target_arch = "wasm32")),
            system_theme: cfg!(any(
                target_os = "macos",
                target_os = "windows",
                target_os = "linux"
            )),
        }
    }
}

/// Global application configuration signal
///
/// Reference: ./tmp/dioxus/examples/global.rs pattern
pub static APP_CONFIG: GlobalSignal<AppConfig> = Signal::global(AppConfig::detect);

/// Hook to access application configuration from any component
///
/// Reference: ./tmp/dioxus/examples/context_api.rs pattern
pub fn use_app_config() -> Signal<AppConfig> {
    try_use_context::<Signal<AppConfig>>().unwrap_or_else(|| {
        log::warn!("AppConfig context not found, using global signal");
        Signal::new(APP_CONFIG())
    })
}

/// Initialize application configuration context at app root
///
/// Call this in your main app component before rendering other components
pub fn provide_app_config() {
    use_context_provider(|| Signal::new(AppConfig::detect()));
}
