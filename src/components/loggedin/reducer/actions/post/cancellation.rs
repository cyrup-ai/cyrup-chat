//! Post cancellation handler with zero-allocation patterns
//!
//! This module handles post creation cancellation with efficient state cleanup
//! and proper resource management.

use super::{super::ReducerState, PostError};
use crate::environment::Environment;
use dioxus::prelude::*;

/// Optimized post cancellation handler with zero-allocation patterns
///
/// Handles post creation cancellation with efficient state cleanup.
#[inline(always)]
pub fn handle_post_cancel(
    mut signal: Signal<ReducerState>,
    _environment: &Environment,
) -> Result<(), PostError> {
    log::debug!("Cancelling post creation");

    // Check if there's actually a post in progress
    let has_reply = signal.with(|state| state.is_replying.is_some());

    if !has_reply {
        log::debug!("No post in progress to cancel");
        return Ok(()); // Not an error, just nothing to cancel
    }

    // Clear replying state
    signal.with_mut(|state| {
        // Get information about what we're cancelling for logging
        if let Some((ref kind, ref paths)) = state.is_replying {
            log::debug!("Cancelling {kind:?} with {} attachments", paths.len());
        }

        // Clear the replying state
        state.is_replying = None;

        // Clear any post-related errors
        if let Some(ref error) = state.error
            && (error.contains("post") || error.contains("reply"))
        {
            state.error = None;
        }
    });

    log::debug!("Post creation cancelled successfully");
    Ok(())
}
