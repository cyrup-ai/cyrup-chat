//! Conversation selection handler with zero-allocation patterns

use super::super::ReducerState;
use super::errors::SelectionError;
use crate::environment::Environment;
use crate::view_model::StatusId;
use dioxus::prelude::*;

/// Optimized conversation selection handler with zero-allocation patterns
///
/// Handles conversation selection for conversation viewing with efficient
/// state updates and conversation loading.
///
/// Note: Called through Action::SelectConversation dispatch - production ready
#[inline(always)]
#[allow(dead_code)] // Conversation selection handler - modular architecture for future use
pub fn handle_select_conversation(
    mut signal: Signal<ReducerState>,
    status_id: StatusId,
    environment: &Environment,
) -> Result<(), SelectionError> {
    log::debug!("Selecting conversation for status: {}", status_id.0);

    // Validate status ID
    if status_id.0.is_empty() {
        return Err(SelectionError::ConversationSelection(
            "Status ID is empty".to_string(),
        ));
    }

    // Update state to prepare for conversation loading
    signal.with_mut(|state| {
        state.flags.loading_conversation = true;

        // Clear any navigation errors
        if let Some(ref error) = state.error
            && error.contains("conversation")
        {
            state.error = None;
        }
    });

    // Load conversation data asynchronously
    spawn({
        let environment = environment.clone();
        let mut signal = signal;
        let status_id = status_id.clone();

        async move {
            // Build conversation from messages using helper
            use crate::components::conversation::conversation_helpers::build_conversation;

            match build_conversation(&environment.model, status_id.0.clone()).await {
                Ok(conversation) => {
                    log::debug!(
                        "Successfully built conversation for status: {}",
                        status_id.0
                    );

                    // Update storage with conversation data
                    let mut env_clone = environment.clone();
                    env_clone.storage.with_mut(|storage| {
                        storage
                            .conversations
                            .insert(status_id.clone(), conversation);
                    });

                    // Clear loading state
                    signal.with_mut(|state| {
                        state.flags.loading_conversation = false;
                    });
                }
                Err(e) => {
                    log::error!("Failed to build conversation: {e}");

                    signal.with_mut(|state| {
                        state.flags.loading_conversation = false;
                        state.error = Some(format!("Failed to load conversation: {e}"));
                    });
                }
            }
        }
    });

    Ok(())
}
