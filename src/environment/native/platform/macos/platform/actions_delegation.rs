//! Actions delegation functionality for macOS Platform
//!
//! This module handles delegation of public actions and menu operations to the actions module.

use super::core::Platform;

impl Platform {
    /// Handle public actions by delegating to actions module
    #[inline(always)]
    pub async fn handle_public_action(
        &self,
        action: crate::PublicAction,
        instance_url: &str,
    ) -> Result<(), String> {
        super::super::actions::handle_public_action(action, instance_url).await
    }

    /// Show emoji picker by delegating to actions module
    #[inline(always)]
    pub fn show_emoji_popup(&self) -> Result<(), String> {
        super::super::actions::show_emoji_popup()
    }

    /// Show context menu by delegating to actions module
    #[inline(always)]
    pub fn show_context_menu<T: Clone + std::fmt::Debug + Send + 'static>(
        &self,
        coordinates: (i32, i32),
        title: &str,
        items: Vec<(&str, T)>,
        callback: impl Fn(T) + 'static,
    ) {
        super::super::actions::show_context_menu(coordinates, title, items, callback);
    }
}
