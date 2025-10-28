//! Public API for async platform operations
//!
//! This module contains the main public methods that provide cross-platform
//! async operations for UI elements and feature configuration.

use super::core::{AsyncPlatform, CursorPosition, TextAreaConfig};
use crate::errors::ui::UiError;

impl AsyncPlatform {
    /// Configure text area asynchronously without blocking
    pub async fn configure_text_area(
        element_id: &str,
        config: &TextAreaConfig,
    ) -> Result<(), UiError> {
        log::debug!("Configuring text area '{element_id}' with async platform operations");

        #[cfg(target_os = "macos")]
        {
            Self::macos_configure_text_area_async(element_id, config).await
        }

        #[cfg(target_os = "windows")]
        {
            Self::windows_configure_text_area_async(element_id, config).await
        }

        #[cfg(target_os = "linux")]
        {
            Self::linux_configure_text_area_async(element_id, config).await
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self::web_configure_text_area_async(element_id, config).await
        }

        #[cfg(not(any(
            target_os = "macos",
            target_os = "windows",
            target_os = "linux",
            target_arch = "wasm32"
        )))]
        {
            log::debug!("Platform not supported for text area configuration");
            Ok(())
        }
    }

    /// Focus element asynchronously without blocking
    pub async fn focus_element(
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> Result<(), UiError> {
        log::debug!("Focusing element '{element_id}' with async platform operations");

        #[cfg(target_os = "macos")]
        {
            Self::macos_focus_element_async(element_id, cursor_position).await
        }

        #[cfg(target_os = "windows")]
        {
            Self::windows_focus_element_async(element_id, cursor_position).await
        }

        #[cfg(target_os = "linux")]
        {
            Self::linux_focus_element_async(element_id, cursor_position).await
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self::web_focus_element_async(element_id, cursor_position).await
        }

        #[cfg(not(any(
            target_os = "macos",
            target_os = "windows",
            target_os = "linux",
            target_arch = "wasm32"
        )))]
        {
            log::debug!("Platform not supported for element focusing");
            Ok(())
        }
    }

    /// Setup drag-and-drop handlers asynchronously
    pub async fn setup_drag_drop_handlers(
        _updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>,
    ) -> Result<(), UiError> {
        log::debug!("Setting up drag-drop handlers with async platform operations");

        #[cfg(target_os = "macos")]
        {
            Self::macos_setup_drag_drop_async().await
        }

        #[cfg(target_os = "windows")]
        {
            Self::windows_setup_drag_drop_async().await
        }

        #[cfg(target_os = "linux")]
        {
            Self::linux_setup_drag_drop_async(updater).await
        }

        #[cfg(target_arch = "wasm32")]
        {
            Self::web_setup_drag_drop_async().await
        }

        #[cfg(not(any(
            target_os = "macos",
            target_os = "windows",
            target_os = "linux",
            target_arch = "wasm32"
        )))]
        {
            log::debug!("Platform not supported for drag-drop setup");
            Ok(())
        }
    }

    /// Toggle file upload feature asynchronously
    pub async fn toggle_file_upload_feature(enabled: bool) -> Result<(), UiError> {
        log::debug!("Toggling file upload feature: {enabled}");

        // Direct async call without spawning tasks - Modern Dioxus 0.7 async pattern
        Self::update_feature_config("file_upload", enabled).await
    }

    /// Update feature configuration asynchronously
    async fn update_feature_config(feature: &str, enabled: bool) -> Result<(), UiError> {
        // Use async file I/O instead of blocking operations
        use serde_json::{Value, json};
        use tokio::fs;

        let config_path = dirs::config_dir()
            .ok_or_else(|| UiError::platform_error("No config directory available"))?
            .join("cyrup")
            .join("features.json");

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                UiError::platform_error(format!("Failed to create config directory: {e}"))
            })?;
        }

        // Read existing config or create new one
        let mut config: Value = match fs::read_to_string(&config_path).await {
            Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| json!({})),
            Err(_) => json!({}),
        };

        // Update feature setting
        config[feature] = json!(enabled);

        // Write config asynchronously
        let content = serde_json::to_string_pretty(&config)
            .map_err(|e| UiError::platform_error(format!("Failed to serialize config: {e}")))?;

        fs::write(&config_path, content)
            .await
            .map_err(|e| UiError::platform_error(format!("Failed to write config: {e}")))?;

        log::info!(
            "Feature '{}' {}d successfully",
            feature,
            if enabled { "enable" } else { "disable" }
        );
        Ok(())
    }
}
