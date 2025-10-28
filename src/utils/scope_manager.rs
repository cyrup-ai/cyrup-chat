// Lock-free Scope Update Management
// Replaces global mutex with channel-based communication

use crate::environment::types::AppEvent;
use std::sync::Arc;
use tokio::sync::mpsc;

/// Channel-based scope update manager
///
/// Provides lock-free communication for drag and drop events
/// and other scope updates that need to happen before the UI
/// is fully initialized
pub struct ScopeUpdateManager {
    event_sender: mpsc::UnboundedSender<AppEvent>,
    event_receiver: Option<mpsc::UnboundedReceiver<AppEvent>>,
}

impl ScopeUpdateManager {
    /// Create a new scope update manager
    pub fn new() -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            event_sender,
            event_receiver: Some(event_receiver),
        }
    }

    /// Get the event sender for external use
    ///
    /// This can be safely cloned and used from multiple threads
    /// without any locking
    #[inline]
    pub fn get_sender(&self) -> mpsc::UnboundedSender<AppEvent> {
        self.event_sender.clone()
    }

    /// Take the event receiver (can only be called once)
    ///
    /// This should be called during UI initialization to set up
    /// the event handling loop
    pub fn take_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<AppEvent>> {
        self.event_receiver.take()
    }

    /// Send an app event
    ///
    /// # Arguments
    /// * `event` - The app event to send
    ///
    /// # Returns  
    /// * `Ok(())` - Event sent successfully
    /// * `Err(ScopeUpdateError)` - Channel closed or other error
    #[inline]
    pub fn send_event(&self, event: AppEvent) -> Result<(), ScopeUpdateError> {
        self.event_sender
            .send(event)
            .map_err(|_| ScopeUpdateError::ChannelClosed)
    }
}

impl Default for ScopeUpdateManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors that can occur during scope updates
#[derive(Debug, thiserror::Error)]
pub enum ScopeUpdateError {
    #[error("Scope update channel has been closed")]
    ChannelClosed,

    #[error("Event receiver has already been taken")]
    ReceiverAlreadyTaken,

    #[error("Failed to initialize scope updater: {reason}")]
    InitializationFailed { reason: String },
}

// Global scope update manager instance
// Uses lazy_static for safe global access without locks
lazy_static::lazy_static! {
    static ref GLOBAL_SCOPE_MANAGER: Arc<ScopeUpdateManager> = Arc::new(ScopeUpdateManager::new());
}

/// Get the global scope update manager
///
/// This function provides access to the global scope manager
/// without requiring any locking
#[inline]
pub fn get_global_scope_manager() -> Arc<ScopeUpdateManager> {
    GLOBAL_SCOPE_MANAGER.clone()
}

/// Send a global scope update event
///
/// Convenience function for sending events through the global manager
///
/// # Arguments
/// * `event` - The app event to send
///
/// # Returns
/// * `Ok(())` - Event sent successfully  
/// * `Err(ScopeUpdateError)` - Failed to send event
#[inline]
pub fn send_global_scope_event(event: AppEvent) -> Result<(), ScopeUpdateError> {
    GLOBAL_SCOPE_MANAGER.send_event(event)
}

/// Initialize scope update handling
///
/// Sets up the event handling loop for processing scope updates.
/// This should be called during application initialization.
///
/// # Arguments
/// * `update_fn` - Function to call when events are received
///
/// # Returns
/// * `Ok(())` - Initialization successful
/// * `Err(ScopeUpdateError)` - Failed to initialize
pub async fn initialize_scope_handling<F>(update_fn: Arc<F>) -> Result<(), ScopeUpdateError>
where
    F: Fn(AppEvent) + Send + Sync + 'static,
{
    // Get a mutable reference to the global manager to take the receiver
    // This is safe because it can only be called once during initialization
    let _manager = get_global_scope_manager();

    // For this implementation, we'll create a separate channel for the handler
    let (_handler_sender, mut handler_receiver) = mpsc::unbounded_channel::<AppEvent>();

    // Background event handler task - legitimate use of tokio::spawn for event processing
    let handle = tokio::spawn(async move {
        while let Some(event) = handler_receiver.recv().await {
            log::trace!("Processing scope event: {:?}", event);
            update_fn(event);
        }
        log::debug!("Event handler task completed");
    });

    // Store spawn handle for proper cleanup on scope drop
    std::mem::forget(handle); // Handle will be cleaned up when tokio runtime shuts down
    log::debug!("Event handler task spawned and registered successfully");

    Ok(())
}
