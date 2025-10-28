//! Event handling action handlers
//!
//! Handles app events, message events, and system notifications.

use crate::environment::Environment;
use crate::environment::model::Message;
use crate::environment::types::AppEvent;
use dioxus::prelude::*;

use super::super::{ActionError, ReducerState};

/// Handle application events
pub fn handle_app_event(
    mut signal: Signal<ReducerState>,
    event: AppEvent,
    _environment: &mut Environment,
) -> Result<(), ActionError> {
    match event {
        AppEvent::MenuEvent(menu_event) => {
            log::debug!("Menu event received: {menu_event:?}");
            // Handle menu interactions
        }
        AppEvent::FocusChange(_change) => {
            log::debug!("Focus change event received");
            // Handle focus changes
        }
        AppEvent::FileEvent(_file_event) => {
            log::debug!("File event received");
            // Handle file operations
        }
        AppEvent::ClosingWindow => {
            log::debug!("Window closing event received");
            // Handle window cleanup
        }
    }

    // Trigger signal update
    signal.with_mut(|_| {});
    Ok(())
}

/// Handle message events
pub fn handle_message_event(
    mut signal: Signal<ReducerState>,
    message: Message,
    _environment: &mut Environment,
) -> Result<(), ActionError> {
    log::debug!("Message event received: {message:?}");

    signal.with_mut(|state| {
        // Update notification state if needed
        // Note: Message enum variants will be handled when available
        state.has_new_notifications = true;
    });

    Ok(())
}
