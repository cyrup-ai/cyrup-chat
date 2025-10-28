use super::core::{Action, State, TimelineSignal};
use crate::environment::Environment;
use dioxus::prelude::*;

impl State {
    pub fn handle_initial(mut signal: TimelineSignal, environment: &mut Environment) {
        let provider = signal.read().provider.clone();
        let provider_id = provider.identifier();
        let identifier = format!("status_timeline_store_data_{provider_id}").replace(' ', "-");

        signal.with_mut(|state| {
            state.identifier = identifier.clone();
        });

        // Set up auto-reload timer if needed
        let should_auto_reload = signal.read().provider.should_auto_reload();
        if should_auto_reload {
            spawn({
                let signal = signal;
                let environment = environment.clone();
                let _identifier = identifier.clone();
                async move {
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_secs(45)).await;
                        let mut env = environment.clone();
                        super::handle_action(signal, Action::ShouldReloadSoft, &mut env);
                    }
                }
            });
        }

        // Storage subscription using proper Signal reactive patterns from signals.rs example
        // Use use_effect to reactively watch storage changes instead of manual subscription
        let storage_signal = environment.storage;
        let timeline_signal = signal;

        // Clone environment outside of async context to avoid lifetime issues
        let environment_clone = environment.clone();
        spawn({
            let identifier = identifier.clone();
            async move {
                // Set up reactive subscription to storage changes
                use_effect(move || {
                    // Watch for changes in storage that affect this timeline
                    let storage_data = storage_signal.read();
                    if storage_data.has_timeline_updates(&identifier) {
                        log::debug!("Timeline {identifier} has storage updates");
                        // Trigger timeline refresh through DataChanged action
                        spawn({
                            let timeline_signal = timeline_signal;
                            let mut environment = environment_clone.clone();
                            async move {
                                super::handle_action(
                                    timeline_signal,
                                    Action::DataChanged,
                                    &mut environment,
                                );
                            }
                        });
                    }
                });
            }
        });

        // Trigger LoadData action
        spawn({
            let signal = signal;
            let environment = environment.clone();
            async move {
                let mut env = environment;
                super::handle_action(signal, Action::LoadData, &mut env);
            }
        });
    }
}
