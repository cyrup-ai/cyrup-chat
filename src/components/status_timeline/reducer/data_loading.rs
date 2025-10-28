use super::core::{Action, State, TimelineSignal};
use crate::environment::Environment;
use crate::environment::types::TimelineDirection;
use dioxus::prelude::*;

impl State {
    pub fn handle_load_data(mut signal: TimelineSignal, environment: &mut Environment) {
        signal.with_mut(|state| {
            state.is_loading = true;
        });

        spawn({
            let signal = signal;
            let environment = environment.clone();
            let provider = signal.read().provider.clone();
            async move {
                let result = provider.request_data(None).await;
                let mut env = environment;
                super::handle_action(signal, Action::LoadedData(result, false), &mut env);
            }
        });
    }

    pub fn handle_load_more_data(
        mut signal: TimelineSignal,
        id: Option<crate::view_model::StatusId>,
        environment: &mut Environment,
    ) {
        let Some(id) = id else {
            signal.with_mut(|state| {
                state.can_load_more = false;
            });
            return;
        };
        signal.with_mut(|state| {
            state.is_loading_more = true;
        });

        spawn({
            let signal = signal;
            let environment = environment.clone();
            let provider = signal.read().provider.clone();
            async move {
                let result = provider.request_data(Some(id)).await;
                let mut env = environment;
                super::handle_action(signal, Action::LoadedMoreData(result), &mut env);
            }
        });
    }

    pub fn handle_loaded_data(
        mut signal: TimelineSignal,
        data: Result<Vec<crate::environment::model::Status>, String>,
        was_reload: bool,
        environment: &mut Environment,
    ) {
        signal.with_mut(|state| {
            state.is_loading = false;
            state.is_loading_more = false; // enabled via was_reload
            let Ok(updates) = data else {
                state.can_load_more = false;
                return;
            };

            let length = updates.len();

            let m = 3;
            if length > m
                && let Some(m) = updates.get(length - m)
            {
                log::debug!("set marker {} for {}", &&m.account.id, &m.id);
                // Handle async operation with spawn to avoid blocking reducer
                spawn({
                    let environment = environment.clone();
                    let account_id = m.account.id.clone();
                    let status_id = m.id.clone();
                    async move {
                        if environment
                            .settings
                            .set_timeline_marker(&account_id, &status_id)
                            .await
                            .is_none()
                        {
                            log::warn!(
                                "Timeline marker operation returned None - may not have been set"
                            );
                        }
                    }
                });
            }

            let possible_scroll = state.provider.scroll_to_item(&updates);

            let direction = state.direction();
            state.can_load_more = state
                .provider
                .process_new_data(&updates, direction, was_reload);
            state.posts = state.provider.data(direction);

            // Update menu with proper platform access - no window parameter needed
            environment.platform.update_menu(|config| {
                config.enable_scroll = true;
            });
            log::debug!("Timeline data loaded: {} items", updates.len());

            if was_reload {
                return;
            }

            // if we're supposed to scroll to the newest, have to do some more work
            // this is based on whether we have a scroll id *and* whether the direction
            // is down
            if direction == TimelineDirection::NewestTop {
                return;
            }

            // Store scroll ID for async operation outside the signal closure
            if let Some(scroll_id) = possible_scroll {
                let dom_id = scroll_id.dom_id();
                let env = environment.clone();

                // Use spawn for async DOM manipulation (proper Dioxus 0.7 pattern)
                spawn(async move {
                    log::debug!("Scroll to item: {dom_id}");
                    if let Err(e) = env.platform.scroll_to_element(&dom_id).await {
                        log::warn!("Failed to scroll to element {dom_id}: {e}");
                    }
                });
            }
        });
    }

    pub fn handle_loaded_more_data(
        mut signal: TimelineSignal,
        result: Result<Vec<crate::environment::model::Status>, String>,
        environment: &mut Environment,
    ) {
        signal.with_mut(|state| {
            state.is_loading_more = false;
            let Ok(batch) = result else {
                return;
            };
            if let Some(ref account) = state.account {
                environment.storage.with_mut(|storage| {
                    if batch.is_empty() {
                        storage.accounts_no_older_data.insert(account.id.clone());
                    } else {
                        storage.accounts_no_older_data.remove(&account.id);
                    }
                });
            }

            let direction = state.direction();
            state.can_load_more = state.provider.process_new_data(&batch, direction, false);
            state.posts = state.provider.data(direction);
            environment
                .storage
                .with_mut(|s| s.update_account_historical_data(&batch, &direction));
        });
    }
}
