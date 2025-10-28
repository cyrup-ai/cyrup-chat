//! Platform API types and shared structures
//!
//! This module contains common types used across platform implementations.

use crate::errors::ui::UiError;
use crate::utils::async_platform::CursorPosition;
use std::future::Future;
use std::pin::Pin;

use super::{DesktopPlatformAPI, PlatformAPI, PlatformFeature, WebPlatformAPI};

/// Async task type for platform operations
pub type AsyncTask<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Enum wrapping different platform implementations
#[derive(Debug)]
pub enum PlatformApiImpl {
    Desktop(DesktopPlatformAPI),
    Web(WebPlatformAPI),
}

impl PlatformAPI for PlatformApiImpl {
    fn configure_text_area(
        &self,
        element_id: &str,
        config: crate::utils::async_platform::TextAreaConfig,
    ) -> AsyncTask<Result<(), UiError>> {
        match self {
            PlatformApiImpl::Desktop(api) => api.configure_text_area(element_id, config),
            PlatformApiImpl::Web(api) => api.configure_text_area(element_id, config),
        }
    }

    fn setup_upload_handlers(
        &self,
        updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>,
    ) -> AsyncTask<Result<(), UiError>> {
        match self {
            PlatformApiImpl::Desktop(api) => api.setup_upload_handlers(updater),
            PlatformApiImpl::Web(api) => api.setup_upload_handlers(updater),
        }
    }

    fn focus_element(
        &self,
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> AsyncTask<Result<(), UiError>> {
        match self {
            PlatformApiImpl::Desktop(api) => api.focus_element(element_id, cursor_position),
            PlatformApiImpl::Web(api) => api.focus_element(element_id, cursor_position),
        }
    }

    fn set_feature_enabled(
        &self,
        feature: PlatformFeature,
        enabled: bool,
    ) -> AsyncTask<Result<(), UiError>> {
        match self {
            PlatformApiImpl::Desktop(api) => api.set_feature_enabled(feature, enabled),
            PlatformApiImpl::Web(api) => api.set_feature_enabled(feature, enabled),
        }
    }
}
