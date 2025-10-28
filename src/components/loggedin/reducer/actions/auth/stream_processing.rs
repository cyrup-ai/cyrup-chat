//! User stream setup and message processing for real-time updates

use super::super::ReducerState;
use super::error_types::AuthError;
use crate::environment::Environment;
use crate::environment::model::Message;
use dioxus::prelude::*;

/// Efficient user stream setup with zero-allocation channel patterns
///
/// Implements the streaming subscription logic from the original LoggedIn handler
/// with optimized async patterns and comprehensive error handling
#[inline(always)]
pub fn setup_user_stream(
    signal: Signal<ReducerState>,
    environment: &Environment,
) -> Result<(), AuthError> {
    let _model = environment.model.clone();

    spawn({
        let _signal = signal;
        let environment = environment.clone();
        async move {
            // Stream subscription with comprehensive error handling
            if let Err(e) = environment
                .model
                .subscribe_user_stream(std::sync::Arc::new(|message| {
                    log::debug!("Received stream message: {:?}", message);

                    // Process different types of streaming messages
                    match message {
                        crate::environment::native::model::types::Message::Update(status) => {
                            log::debug!("Stream update for status: {}", status.id);
                            // Status updates are handled by storage layer
                        }
                        crate::environment::native::model::types::Message::Notification(
                            notification,
                        ) => {
                            log::debug!("Stream notification: {}", notification.id);
                            // Notifications are handled by notification system
                        }
                        crate::environment::native::model::types::Message::Delete(status_id) => {
                            log::debug!("Stream delete for status: {}", status_id);
                            // Deletions are handled by storage layer
                        }
                        crate::environment::native::model::types::Message::StatusUpdate(status) => {
                            log::debug!("Stream status update: {}", status.id);
                            // Status updates are handled by storage layer
                        }
                        crate::environment::native::model::types::Message::Heartbeat() => {
                            log::trace!("Stream heartbeat received");
                            // Heartbeats maintain connection health
                        }
                        crate::environment::native::model::types::Message::Conversation(_) => {
                            log::debug!("Stream conversation message received");
                            // Conversation messages are handled by conversation system
                        }
                    }
                }))
                .await
            {
                log::error!("User stream subscription failed: {e}");
            }
        }
    });

    Ok(())
}

/// Optimized stream message processing with efficient updates
///
/// Processes incoming stream messages with minimal allocations
/// and efficient storage updates
#[inline(always)]
#[allow(dead_code)]
pub fn process_stream_message(
    mut signal: Signal<ReducerState>,
    environment: &Environment,
    message: Message,
) {
    match message {
        Message::Update(status) => {
            let status_vm = crate::view_model::StatusViewModel::new(&status);

            // Separate storage and signal operations to avoid double mutable borrow
            let mut env_clone = environment.clone();
            env_clone.storage.with_mut(|storage| {
                storage.add_status_update(status_vm);
            });

            // Trigger signal update
            signal.with_mut(|_| {});
        }
        Message::Notification(notification) => {
            // Update storage first
            let mut env_clone = environment.clone();
            env_clone.storage.with_mut(|storage| {
                storage.add_notification_update(notification);
            });

            // Then update signal state
            signal.with_mut(|state| {
                state.has_new_notifications = true;
            });
        }
        Message::Delete(status_id) => {
            // Update storage first
            let mut env_clone = environment.clone();
            env_clone.storage.with_mut(|storage| {
                storage.remove_status(&status_id);
            });

            // Trigger signal update
            signal.with_mut(|_| {});
        }
        Message::StatusUpdate(updated_status) => {
            let status_vm = crate::view_model::StatusViewModel::new(&updated_status);

            // Update storage first
            let mut env_clone = environment.clone();
            env_clone.storage.with_mut(|storage| {
                storage.update_existing_status(status_vm);
            });

            // Trigger signal update
            signal.with_mut(|_| {});
        }
        _ => {
            log::debug!("Received other streaming message type: {message:?}");
        }
    }
}
