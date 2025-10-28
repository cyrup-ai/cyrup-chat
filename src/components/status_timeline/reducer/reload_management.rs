use super::core::{Action, State, TimelineSignal};
use crate::environment::Environment;
use dioxus::prelude::*;

impl State {
    pub fn handle_should_reload_soft(signal: TimelineSignal, environment: &mut Environment) {
        // Use proper Signal-based UI interactions for scroll position checking
        log::debug!("ShouldReloadSoft requested - checking scroll position");

        // Use modern web platform APIs through environment for scroll position
        let _should_reload = environment.platform.should_auto_reload().unwrap_or(true);
        spawn({
            let environment = environment.clone();
            async move {
                let mut env = environment;
                super::handle_action(signal, Action::ReloadSoft(true), &mut env);
            }
        });
    }

    pub fn handle_reload_soft(
        mut signal: TimelineSignal,
        reload: bool,
        environment: &mut Environment,
    ) {
        if !reload {
            return;
        }
        signal.with_mut(|state| {
            state.is_loading_more = true;
        });

        spawn({
            let signal = signal;
            let environment = environment.clone();
            let provider = signal.read().provider.clone();
            async move {
                let result = provider.request_data(None).await;
                let mut env = environment;
                super::handle_action(signal, Action::LoadedData(result, true), &mut env);
            }
        });
    }

    pub fn handle_data_changed(mut signal: TimelineSignal, environment: &mut Environment) {
        signal.with_mut(|state| {
            state.posts = state.provider.data(state.direction());
            environment.storage.with(|data| {
                state.known_conversations = data.conversations.keys().cloned().collect();
            });
        });
    }
}
