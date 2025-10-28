//! Unit tests for loggedin reducer module
//!
//! These tests verify the core functionality of the reducer state management
//! including initial state creation and default implementations.

use cyrup::components::loggedin::reducer::{create_initial_state, ReducerState};

/// Test that initial state is created correctly with expected defaults
#[test]
fn test_initial_state_creation() {
    let state = create_initial_state();
    assert!(!state.logged_in);
    assert!(!state.flags.logging_in);
    assert!(state.error.is_none());
}

/// Test that ReducerState default implementation sets expected values  
#[test]
fn test_reducer_state_defaults() {
    let state = ReducerState::default();
    assert_eq!(state.logged_in, false);
    assert_eq!(state.flags.logging_in, false);
    assert_eq!(state.has_new_notifications, false);
    assert!(state.current_user.is_none());
    assert!(state.selected_account.is_none());
}