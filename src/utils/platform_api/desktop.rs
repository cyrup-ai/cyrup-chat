//! Desktop platform implementation
//!
//! This module provides desktop-specific implementations using native APIs
//! instead of JavaScript execution for macOS, Windows, and Linux.

use super::features::*;
use super::{PlatformAPI, PlatformFeature, types::AsyncTask};
use crate::errors::ui::UiError;
use crate::utils::async_file_dialog::{AsyncFileDialog, FileDialogConfig, FileDialogResult};
use crate::utils::async_platform::{AsyncPlatform, CursorPosition, TextAreaConfig};

/// Desktop implementation of PlatformAPI
///
/// Provides desktop-specific implementations using native APIs
/// instead of JavaScript execution
#[derive(Debug)]
pub struct DesktopPlatformAPI;

impl PlatformAPI for DesktopPlatformAPI {
    fn configure_text_area(
        &self,
        element_id: &str,
        config: TextAreaConfig,
    ) -> AsyncTask<Result<(), UiError>> {
        let element_id = element_id.to_string();
        Box::pin(async move {
            log::debug!("Configuring native text area '{element_id}' with config: {config:?}");

            // Apply delay if specified for UI stability
            if let Some(delay) = config.delay_ms {
                tokio::time::sleep(tokio::time::Duration::from_millis(delay as u64)).await;
            }

            // Platform-specific text area configuration
            #[cfg(target_os = "macos")]
            {
                macos_configure_text_area(&element_id, &config).await?;
            }

            #[cfg(target_os = "windows")]
            {
                windows_configure_text_area(&element_id, &config).await?;
            }

            #[cfg(target_os = "linux")]
            {
                linux_configure_text_area(&element_id, &config).await?;
            }

            log::debug!("Text area '{element_id}' configured successfully");
            Ok(())
        })
    }

    fn setup_upload_handlers(
        &self,
        updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>,
    ) -> AsyncTask<Result<(), UiError>> {
        Box::pin(async move {
            log::debug!("Setting up native file upload handlers");

            // Initialize native file dialog capabilities using async implementation
            let config = FileDialogConfig::new()
                .with_title("Select files to upload")
                .with_filter(
                    "Images",
                    vec![
                        "png".to_string(),
                        "jpg".to_string(),
                        "jpeg".to_string(),
                        "gif".to_string(),
                        "webp".to_string(),
                    ],
                )
                .with_filter(
                    "Documents",
                    vec![
                        "txt".to_string(),
                        "md".to_string(),
                        "pdf".to_string(),
                        "doc".to_string(),
                        "docx".to_string(),
                    ],
                )
                .with_filter("All Files", vec!["*".to_string()])
                .multiple(true);

            match AsyncFileDialog::pick_files(config).await? {
                FileDialogResult::Selected(files) => {
                    log::info!(
                        "File dialog initialized, {} file types supported",
                        files.len()
                    );
                    setup_drag_drop_handlers(updater.clone()).await?;
                }
                FileDialogResult::Cancelled => {
                    log::debug!("File dialog cancelled by user");
                    // Still setup drag-drop handlers even if dialog was cancelled
                    setup_drag_drop_handlers(updater.clone()).await?;
                }
                FileDialogResult::Error(msg) => {
                    log::error!("File dialog failed: {msg}");
                    // Continue with drag-drop setup even if dialog had errors
                    setup_drag_drop_handlers(updater.clone()).await?;
                }
            }

            log::debug!("Native file upload handlers configured successfully");
            Ok(())
        })
    }

    fn focus_element(
        &self,
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> AsyncTask<Result<(), UiError>> {
        let element_id = element_id.to_string();
        Box::pin(async move {
            log::debug!(
                "Focusing native element '{element_id}' with cursor at {cursor_position:?}"
            );

            // Platform-specific element focusing
            #[cfg(target_os = "macos")]
            {
                macos_focus_element(&element_id, cursor_position).await?;
            }

            #[cfg(target_os = "windows")]
            {
                windows_focus_element(&element_id, cursor_position).await?;
            }

            #[cfg(target_os = "linux")]
            {
                linux_focus_element(&element_id, cursor_position).await?;
            }

            log::debug!("Element '{element_id}' focused successfully");
            Ok(())
        })
    }

    fn set_feature_enabled(
        &self,
        feature: PlatformFeature,
        enabled: bool,
    ) -> AsyncTask<Result<(), UiError>> {
        Box::pin(async move {
            log::info!(
                "{} native feature: {}",
                if enabled { "Enabling" } else { "Disabling" },
                feature
            );

            // Configure platform-specific features with real implementations
            match feature {
                PlatformFeature::FileUpload => {
                    toggle_file_upload_feature(enabled).await?;
                }
                PlatformFeature::SpellCheck => {
                    toggle_spell_check_feature(enabled).await?;
                }
                PlatformFeature::AutoCorrect => {
                    toggle_auto_correct_feature(enabled).await?;
                }
                PlatformFeature::ContextMenus => {
                    toggle_context_menus_feature(enabled).await?;
                }
                PlatformFeature::KeyboardShortcuts => {
                    configure_keyboard_shortcuts(enabled).await?;
                }
            }

            log::info!(
                "Feature '{}' {} successfully",
                feature,
                if enabled { "enabled" } else { "disabled" }
            );
            Ok(())
        })
    }
}

// Platform-specific implementation functions

// macOS implementations using Cocoa and native APIs
#[cfg(target_os = "macos")]
async fn macos_configure_text_area(
    element_id: &str,
    config: &TextAreaConfig,
) -> Result<(), UiError> {
    // Use async platform abstraction instead of blocking Cocoa calls
    AsyncPlatform::configure_text_area(element_id, config).await
}

#[cfg(target_os = "macos")]
async fn macos_focus_element(
    element_id: &str,
    cursor_position: CursorPosition,
) -> Result<(), UiError> {
    // Use async platform abstraction instead of blocking operations
    AsyncPlatform::focus_element(element_id, cursor_position).await
}

// Windows implementations using async operations
#[cfg(target_os = "windows")]
async fn windows_configure_text_area(
    element_id: &str,
    config: &TextAreaConfig,
) -> Result<(), UiError> {
    // Use async platform abstraction instead of blocking Win32 calls
    AsyncPlatform::configure_text_area(element_id, config).await
}

#[cfg(target_os = "windows")]
async fn windows_focus_element(
    element_id: &str,
    cursor_position: CursorPosition,
) -> Result<(), UiError> {
    // Use async platform abstraction instead of blocking Win32 calls
    AsyncPlatform::focus_element(element_id, cursor_position).await
}

// Linux implementations using async operations
#[cfg(target_os = "linux")]
async fn linux_configure_text_area(
    element_id: &str,
    config: &TextAreaConfig,
) -> Result<(), UiError> {
    // Use async platform abstraction instead of blocking GTK calls
    AsyncPlatform::configure_text_area(element_id, config).await
}

#[cfg(target_os = "linux")]
async fn linux_focus_element(
    element_id: &str,
    cursor_position: CursorPosition,
) -> Result<(), UiError> {
    // Use async platform abstraction instead of blocking GTK calls
    AsyncPlatform::focus_element(element_id, cursor_position).await
}

// Cross-platform drag-and-drop setup
async fn setup_drag_drop_handlers(
    updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>,
) -> Result<(), UiError> {
    // Use async platform abstraction instead of blocking operations
    AsyncPlatform::setup_drag_drop_handlers(updater).await
}
