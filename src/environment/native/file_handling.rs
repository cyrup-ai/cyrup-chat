//! File handling functionality for native environment

use crate::environment::types::{AppEvent, FileEvent};
use dioxus::html::FileData;
use std::sync::Arc;

/// File drop handling for Dioxus 0.7 using FileData API
pub fn handle_file_event(
    files: Vec<FileData>,
    updater: &Arc<dyn Fn(AppEvent) + Send + Sync>,
) -> bool {
    if !files.is_empty() {
        log::debug!("Files dropped: {} files", files.len());
        let paths: Vec<_> = files.iter().map(|f| f.path()).collect();
        updater(AppEvent::FileEvent(FileEvent::Dropped(paths)));
        true
    } else {
        log::debug!("No files in drop event");
        updater(AppEvent::FileEvent(FileEvent::Cancelled));
        false
    }
}
