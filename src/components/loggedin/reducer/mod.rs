//! Optimized loggedin state management with modular action handlers
//!
//! This module provides the main interface for loggedin state management
//! using efficient, zero-allocation patterns and comprehensive error handling.

pub mod actions;

// Modern public API exports with clean interfaces
pub use actions::{Action, ActionError, ReducerState, handle_action};

use crate::environment::Environment;
use dioxus::prelude::*;

/// Modern action handler with comprehensive error handling and zero allocation patterns
/// Uses Result<T, E> for proper error propagation and structured error handling
#[allow(dead_code)] // Modern action handler - architectural scaffolding for future Dioxus 0.7 migration
pub fn handle_action_modern(
    signal: Signal<ReducerState>,
    action: Action,
    environment: &mut Environment,
) -> Result<(), ActionError> {
    handle_action(signal, action, environment)
}

/// Efficient signal-based handle_action with optimized error handling
///
/// This is the primary interface for action handling with full error propagation
/// and performance optimizations.
#[inline(always)]
#[allow(dead_code)]
pub fn handle_action_with_result(
    signal: Signal<ReducerState>,
    action: Action,
    environment: &mut Environment,
) -> Result<(), ActionError> {
    handle_action(signal, action, environment)
}

/// Type alias for the main reducer state signal
#[allow(dead_code)]
pub type LoggedinSignal = Signal<ReducerState>;

/// Create a new reducer state with optimized defaults
#[inline(always)]
#[allow(dead_code)]
pub fn create_initial_state() -> ReducerState {
    ReducerState::default()
}

/// Create a new reducer signal with initial state
#[inline(always)]
#[allow(dead_code)]
pub fn create_reducer_signal() -> Signal<ReducerState> {
    use_signal(create_initial_state)
}
