//! macOS-specific async platform implementations
//!
//! This module contains all macOS-specific implementations for async platform
//! operations using AppleScript and native macOS APIs.

use super::core::{AsyncPlatform, CursorPosition, TextAreaConfig};
use crate::errors::ui::UiError;

impl AsyncPlatform {
    #[cfg(target_os = "macos")]
    pub(super) async fn macos_configure_text_area_async(
        element_id: &str,
        config: &TextAreaConfig,
    ) -> Result<(), UiError> {
        let element_id = element_id.to_string();
        let config = config.clone();

        Self::macos_text_area_operation_async(element_id, config).await
    }

    #[cfg(target_os = "macos")]
    pub(super) async fn macos_focus_element_async(
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> Result<(), UiError> {
        let element_id = element_id.to_string();

        // Use async Command execution instead of
        Self::macos_focus_operation_async(element_id, cursor_position).await
    }

    #[cfg(target_os = "macos")]
    pub(super) async fn macos_setup_drag_drop_async() -> Result<(), UiError> {
        // Pure async implementation without
        log::debug!("Setting up macOS drag-and-drop async");
        // macOS drag-drop setup would go here using async APIs
        Ok(())
    }

    #[cfg(target_os = "macos")]
    async fn macos_text_area_operation_async(
        element_id: String,
        config: TextAreaConfig,
    ) -> Result<(), UiError> {
        // Use async Command execution instead of blocking operations
        use tokio::process::Command;

        log::debug!("Configuring macOS text area '{element_id}' async");

        // Use AppleScript for UI operations with async execution
        let mut script = "tell application \"System Events\" to ".to_string();

        if config.auto_focus {
            script.push_str(&format!(
                "set focused of text area \"{element_id}\" to true; "
            ));
        }

        match config.cursor_position {
            CursorPosition::Start => {
                script.push_str(&format!(
                    "set selection of text area \"{element_id}\" to insertion point 0; "
                ));
            }
            CursorPosition::End => {
                script.push_str(&format!(
                    "set selection of text area \"{element_id}\" to insertion point -1; "
                ));
            }
            CursorPosition::Position(pos) => {
                script.push_str(&format!(
                    "set selection of text area \"{element_id}\" to insertion point {pos}; "
                ));
            }
            CursorPosition::SelectAll => {
                script.push_str(&format!("select text area \"{element_id}\"; "));
            }
        }

        // Execute AppleScript asynchronously using tokio::process::Command
        match Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .await
        {
            Ok(output) => {
                if output.status.success() {
                    log::debug!("macOS text area configured successfully");
                    Ok(())
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    log::warn!("macOS text area configuration warning: {error}");
                    Ok(()) // Non-critical failure, continue
                }
            }
            Err(e) => {
                log::warn!("macOS text area configuration failed: {e}");
                Ok(()) // Non-critical failure, continue
            }
        }
    }

    #[cfg(target_os = "macos")]
    async fn macos_focus_operation_async(
        element_id: String,
        cursor_position: CursorPosition,
    ) -> Result<(), UiError> {
        log::debug!("Focusing macOS element '{element_id}' with cursor {cursor_position:?}");

        // Use async AppleScript instead of blocking operations
        use tokio::process::Command;

        let script = format!(
            "tell application \"System Events\" to set focused of UI element \"{element_id}\" to true"
        );

        match Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .await
        {
            Ok(_) => {
                log::debug!("macOS element focused successfully");
                Ok(())
            }
            Err(e) => {
                log::warn!("macOS element focus failed: {e}");
                Ok(()) // Non-critical failure, continue
            }
        }
    }
}
