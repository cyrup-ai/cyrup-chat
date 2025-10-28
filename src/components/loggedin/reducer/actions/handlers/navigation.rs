//! Navigation action handlers
//!
//! Handles UI navigation, tab switching, and view transitions.

use crate::components::sidebar::MoreSelection;
use crate::environment::Environment;
use crate::view_model::{AccountViewModel, StatusId};
use dioxus::prelude::*;

use super::super::{ActionError, ReducerState};

/// Handle navigation to account view
pub fn handle_select_account(
    mut signal: Signal<ReducerState>,
    account: AccountViewModel,
    _environment: &mut Environment,
) -> Result<(), ActionError> {
    signal.with_mut(|state| {
        state.selected_account = Some(account);
        state.flags.loading_account = true;
    });
    Ok(())
}

/// Handle navigation to notifications view
pub fn handle_select_notifications(
    mut signal: Signal<ReducerState>,
    account: AccountViewModel,
    _environment: &mut Environment,
) -> Result<(), ActionError> {
    signal.with_mut(|state| {
        state.selected_notifications = Some(account);
        state.flags.loading_notifications = true;
    });
    Ok(())
}

/// Handle navigation to conversation view
pub fn handle_select_conversation(
    mut signal: Signal<ReducerState>,
    status_id: StatusId,
    _environment: &mut Environment,
) -> Result<(), ActionError> {
    signal.with_mut(|state| {
        state.flags.loading_conversation = true;
    });
    log::debug!("Navigating to conversation: {status_id:?}");
    Ok(())
}

/// Handle navigation to more menu
pub fn handle_select_more(
    mut signal: Signal<ReducerState>,
    selection: MoreSelection,
    _environment: &mut Environment,
) -> Result<(), ActionError> {
    signal.with_mut(|state| {
        state.more_selection = selection;
    });
    Ok(())
}
