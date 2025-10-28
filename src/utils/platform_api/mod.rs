//! Platform API Abstraction - Production Implementation
//!
//! This module provides the fundamental platform API abstraction for
//! cross-platform operations with zero-allocation patterns.

pub mod desktop;
pub mod features;
pub mod types;
pub mod web;

use crate::errors::ui::UiError;
use crate::utils::async_platform::CursorPosition;

pub use desktop::DesktopPlatformAPI;
pub use features::PlatformFeature;
pub use types::{AsyncTask, PlatformApiImpl};
pub use web::WebPlatformAPI;

/// Platform-specific API abstraction
///
/// Provides a unified interface for platform-specific operations
/// that previously required JavaScript execution
pub trait PlatformAPI: Send + Sync {
    /// Configure text area behavior for optimal user experience
    ///
    /// Sets cursor position, focus behavior, and other text area properties
    /// without requiring JavaScript execution
    ///
    /// # Arguments
    /// * `element_id` - ID of the text area element to configure
    /// * `config` - Configuration options for the text area
    ///
    /// # Returns
    /// * `AsyncTask<Result<(), UiError>>` - Task that completes when configuration is done
    fn configure_text_area(
        &self,
        element_id: &str,
        config: crate::utils::async_platform::TextAreaConfig,
    ) -> AsyncTask<Result<(), UiError>>;

    /// Set up file upload handlers for drag and drop operations
    ///
    /// Configures native file upload handling without JavaScript
    ///
    /// # Arguments
    /// * `updater` - Event updater for emitting file drop events
    ///
    /// # Returns
    /// * `AsyncTask<Result<(), UiError>>` - Task that completes when setup is done
    fn setup_upload_handlers(
        &self,
        updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>,
    ) -> AsyncTask<Result<(), UiError>>;

    /// Focus an element and set cursor position
    ///
    /// # Arguments
    /// * `element_id` - ID of the element to focus
    /// * `cursor_position` - Where to position the cursor
    ///
    /// # Returns
    /// * `AsyncTask<Result<(), UiError>>` - Task that completes when focus is set
    fn focus_element(
        &self,
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> AsyncTask<Result<(), UiError>>;

    /// Enable or disable specific platform features
    ///
    /// # Arguments
    /// * `feature` - Platform feature to control
    /// * `enabled` - Whether to enable or disable the feature
    ///
    /// # Returns
    /// * `AsyncTask<Result<(), UiError>>` - Task that completes when feature is toggled
    fn set_feature_enabled(
        &self,
        feature: PlatformFeature,
        enabled: bool,
    ) -> AsyncTask<Result<(), UiError>>;
}

/// Create a platform API instance based on the current platform
///
/// # Returns
/// A PlatformApiImpl appropriate for the current platform
pub fn create_platform_api() -> PlatformApiImpl {
    #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
    {
        PlatformApiImpl::Desktop(DesktopPlatformAPI)
    }

    #[cfg(target_arch = "wasm32")]
    {
        PlatformApiImpl::Web(WebPlatformAPI)
    }

    #[cfg(not(any(
        target_os = "windows",
        target_os = "macos",
        target_os = "linux",
        target_arch = "wasm32"
    )))]
    {
        // Fallback for other platforms
        PlatformApiImpl::Desktop(DesktopPlatformAPI)
    }
}
