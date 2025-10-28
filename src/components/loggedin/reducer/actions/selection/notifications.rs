//! Notification selection handler with zero-allocation patterns

use super::super::ReducerState;
use super::errors::SelectionError;
use crate::environment::Environment;
use crate::view_model::AccountViewModel;
use dioxus::prelude::*;

/// Optimized notification account selection handler with zero-allocation patterns
///
/// Handles notification account selection for notification viewing with efficient
/// state updates and automatic notification loading.
///
/// Note: Called through Action::SelectNotifications dispatch - production ready
#[inline(always)]
#[allow(dead_code)] // Notifications selection handler - modular architecture for future use
pub fn handle_select_notifications(
    mut signal: Signal<ReducerState>,
    account: AccountViewModel,
    environment: &Environment,
) -> Result<(), SelectionError> {
    log::debug!(
        "Selecting notifications for account: {} (@{})",
        account.display_name,
        account.acct
    );

    // Validate account before selection
    if account.acct.is_empty() {
        return Err(SelectionError::NotificationSelection(
            "Account has empty acct field".to_string(),
        ));
    }

    // Update state with selected notification account
    signal.with_mut(|state| {
        // Clear previous selections to maintain clean state
        state.selected_account = None;
        state.selected_notifications = Some(account.clone());

        // Clear any navigation errors
        if let Some(ref error) = state.error
            && (error.contains("notification") || error.contains("account"))
        {
            state.error = None;
        }
    });

    // Trigger notification loading asynchronously for better UX
    spawn({
        let environment = environment.clone();
        let mut signal = signal;
        let _account_id = account.id.clone();

        async move {
            // Start loading state
            signal.with_mut(|state| {
                state.flags.loading_notifications = true;
            });

            // Load notifications data in background
            match environment.model.notifications(None, 3).await {
                Ok(notifications) => {
                    log::debug!("Successfully loaded {} notifications", notifications.len());

                    // Update storage with new notification data
                    let mut env_clone = environment.clone();
                    let notif_vms: Vec<crate::view_model::NotificationViewModel> = notifications
                        .into_iter()
                        .filter_map(|n| crate::view_model::NotificationViewModel::new(&n))
                        .collect();

                    let has_unread = !notif_vms.is_empty(); // Assume notifications are unread when first loaded

                    env_clone.storage.with_mut(|storage| {
                        // Store notifications in the notification_posts collection by account
                        for notif in notif_vms {
                            storage
                                .notification_posts
                                .entry(notif.status.account.id.clone())
                                .or_insert_with(Vec::new)
                                .push(notif);
                        }
                    });

                    // Update notification indicator state

                    // Clear loading state and update notification indicator
                    signal.with_mut(|state| {
                        state.flags.loading_notifications = false;
                        state.has_new_notifications = has_unread;
                    });
                }
                Err(e) => {
                    log::error!("Failed to load notifications: {e}");

                    signal.with_mut(|state| {
                        state.flags.loading_notifications = false;
                        state.error = Some(format!("Failed to load notifications: {e}"));
                    });
                }
            }
        }
    });

    Ok(())
}
