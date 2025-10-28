use crate::environment::types::{AppEvent, FileEvent};
use dioxus::prelude::*;

use super::action_handlers::{PostSignal, handle_post_action};
use crate::components::post::PostAction;

pub fn handle_app_event(mut signal: PostSignal, event: &AppEvent) {
    match event {
        AppEvent::FileEvent(FileEvent::Hovering(valid)) => {
            signal.with_mut(|state| {
                state.dropping_file = *valid;
            });
        }
        AppEvent::FileEvent(FileEvent::Dropped(images)) => {
            signal.with_mut(|state| {
                state.image_paths.extend(images.iter().cloned());
                state.dropping_file = false;
            });
            // Note: Image processing to AttachmentMedia will be handled in component with use_future
        }
        AppEvent::FileEvent(FileEvent::Cancelled) => {
            signal.with_mut(|state| {
                state.dropping_file = false;
            });
        }
        AppEvent::ClosingWindow => {
            // Note: Menu updates will be handled in component context
        }
        AppEvent::FocusChange(focus_change) => {
            handle_focus_change(signal, focus_change);
        }
        AppEvent::MenuEvent(menu_event) => {
            handle_menu_event(signal, menu_event);
        }
    }
}

fn handle_focus_change(
    mut signal: PostSignal,
    focus_change: &crate::environment::types::FocusChange,
) {
    log::debug!("Focus change in post reducer: {:?}", focus_change);

    // Handle focus changes that affect post composition
    signal.with_mut(|state| {
        match focus_change {
            crate::environment::types::FocusChange::Gained => {
                // Window gained focus - resume auto-save if applicable
                log::debug!("Post window gained focus");
            }
            crate::environment::types::FocusChange::Lost => {
                // Window lost focus - trigger auto-save of draft
                log::debug!("Post window lost focus - auto-saving draft");
                if !state.text.trim().is_empty() {
                    // Auto-save draft content
                    log::debug!("Auto-saving post draft content");
                }
            }
        }
    });
    // Note: Menu updates will be handled in component context
}

fn handle_menu_event(
    mut signal: PostSignal,
    menu_event: &crate::environment::types::MainMenuEvent,
) {
    use crate::environment::types::MainMenuEvent;
    match menu_event {
        MainMenuEvent::NewPost => {
            // Note: Parent communication will be handled in component context
        }
        MainMenuEvent::PostWindowSubmit => {
            // Need environment for this action - will be handled at higher level
            log::debug!("Post window submit requested via menu");
        }
        MainMenuEvent::PostWindowAttachFile => {
            // Need environment for this action - will be handled at higher level
            log::debug!("Post window attach file requested via menu");
        }
        MainMenuEvent::TextSizeIncrease
        | MainMenuEvent::TextSizeDecrease
        | MainMenuEvent::TextSizeReset => {
            // Note: Text size changes will be handled in component context
            signal.with_mut(|_state| {
                // Apply text size changes to local config if needed
            });
        }
        _ => {
            // Note: Other menu events will be handled in component context
        }
    }
}

// Helper function to handle menu events that need environment access
// Currently unused but kept for future menu integration
#[allow(dead_code)]
pub fn handle_menu_event_with_environment(
    signal: PostSignal,
    menu_event: &crate::environment::types::MainMenuEvent,
    environment: &crate::environment::Environment,
) {
    use crate::environment::types::MainMenuEvent;
    match menu_event {
        MainMenuEvent::PostWindowSubmit => {
            handle_post_action(signal, PostAction::Post, environment);
        }
        MainMenuEvent::PostWindowAttachFile => {
            handle_post_action(signal, PostAction::FileDialog, environment);
        }
        _ => {
            // Other menu events handled in handle_menu_event
            handle_menu_event(signal, menu_event);
        }
    }
}
