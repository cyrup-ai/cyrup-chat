pub mod core;
pub mod data_loading;
pub mod event_handling;
pub mod initialization;
pub mod reload_management;

pub use core::{Action, State, TimelineSignal};

use crate::environment::Environment;
use dioxus::prelude::*;

pub fn handle_action(mut signal: TimelineSignal, action: Action, environment: &mut Environment) {
    log::trace!("{action:?}");

    signal.with_mut(|state| {
        state.ui_settings = environment.settings.config().unwrap_or_default();
    });

    match action {
        Action::Initial => {
            State::handle_initial(signal, environment);
        }
        Action::LoadData => {
            State::handle_load_data(signal, environment);
        }
        Action::LoadMoreData(id) => {
            State::handle_load_more_data(signal, id, environment);
        }
        Action::LoadedData(data, was_reload) => {
            State::handle_loaded_data(signal, data, was_reload, environment);
        }
        Action::LoadedMoreData(result) => {
            State::handle_loaded_more_data(signal, result, environment);
        }
        Action::ShouldReloadSoft => {
            State::handle_should_reload_soft(signal, environment);
        }
        Action::ReloadSoft(reload) => {
            State::handle_reload_soft(signal, reload, environment);
        }
        Action::DataChanged => {
            State::handle_data_changed(signal, environment);
        }
        Action::Public(action) => {
            State::handle_public_action(signal, *action, environment);
        }
        Action::AccountVisibility(account, visibility) => {
            State::handle_account_visibility(signal, account, visibility, environment);
        }
        Action::AppEvent(app_event) => {
            State::handle_app_event(signal, app_event, environment);
        }
    }
}
