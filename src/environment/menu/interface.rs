//! Public interface for context menu display

use super::display::show_context_menu;
use super::events::{resolve_current_action, setup_menu_handler};
use super::structures::ContextMenu;
use super::types::{CTX_STATE, CTX_STATE_A};
use crate::environment::platform::AppWindow;
use dioxus::prelude::*;
use std::sync::Arc;

pub trait ViewStoreContextMenu<'a> {
    type Action;
    fn context_menu<T>(&self, event: &'a MouseData, menu: ContextMenu<Self::Action>);
}

// ViewStore pattern removed - using modern Dioxus Signal patterns instead
// Context menus now handled directly in components with Signal-based state

pub fn context_menu<A: Clone + std::fmt::Debug + Send + 'static, T>(
    //sender: ActionSender<A>,
    sender: Arc<dyn Fn(A) + Send + Sync>,
    window: AppWindow,
    event: &MouseData,
    menu: ContextMenu<A>,
) -> impl FnOnce() {
    // Generate unique ID for this context menu using thread-safe counter
    use std::sync::atomic::{AtomicUsize, Ordering};
    static MENU_ID_COUNTER: AtomicUsize = AtomicUsize::new(0);
    let id = MENU_ID_COUNTER.fetch_add(1, Ordering::SeqCst);
    let action_key = std::any::type_name::<A>().to_string();
    let action_key = format!("{id}-{action_key}");

    // Setup the menu handler
    setup_menu_handler::<A>(
        id,
        Some(Arc::new(move |ev| {
            if let Some(action) = resolve_current_action(id, ev) {
                // sender.send(action);
                sender(action);
            }
        })),
    );

    // Show the menu with proper error handling
    if let Err(error) = show_context_menu(&window, event, menu, action_key.clone()) {
        log::error!("Context menu display failed: {error}");
    }

    // Return cleanup function for caller to use with use_effect
    // This allows proper resource cleanup when component unmounts
    let cleanup_action_key = action_key;
    move || cleanup_menu_resources(id, cleanup_action_key)
}

/// Cleanup function to properly remove menu resources when component unmounts
/// This prevents memory leaks and stale event handlers in the global menu state
fn cleanup_menu_resources(_id: usize, action_key: String) {
    // Remove from action handler map
    if let Ok(mut actions) = CTX_STATE_A.write() {
        actions.remove(&action_key);
    } else {
        log::error!("Failed to acquire write lock for menu action cleanup");
    }

    // Remove from event state map
    if let Ok(mut state) = CTX_STATE.write() {
        state.remove(&action_key);
    } else {
        log::error!("Failed to acquire write lock for menu state cleanup");
    }

    log::debug!("Cleaned up menu resources for action_key: {}", action_key);
}
