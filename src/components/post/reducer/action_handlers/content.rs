//! Content and state management operations

use super::super::validation::validate_text;
use super::dispatcher::PostSignal;
use crate::components::loggedin::reducer::actions::post::PostError;
use crate::components::post::Visibility;
use crate::environment::Environment;
use dioxus::prelude::*;
use std::str::FromStr;

pub fn handle_update_visibility(mut signal: PostSignal, vis: String) {
    signal.with_mut(|state| match Visibility::from_str(&vis) {
        Ok(v) => {
            state.visibility = Some(v);
            state.error_message = None;
        }
        Err(_) => {
            state.error_message = Some(
                PostError::validation_failed(format!("Invalid Visibility: {vis:?}")).to_string(),
            );
        }
    });
}

pub fn handle_update_text(mut signal: PostSignal, text: String, environment: &Environment) {
    let instance = environment.model.instance();
    let has_missing_files = signal.with_mut(|state| {
        state.validity = validate_text(Some(instance.clone()), &text);
        state.text = text;

        // Validate post state - check that all image paths exist
        for path in &state.image_paths {
            if !path.exists() {
                log::warn!("Attached file no longer exists: {path:?}");
                state.error_message = Some(
                    PostError::EnvironmentError(format!(
                        "Attached file no longer exists: {path:?}"
                    ))
                    .to_string(),
                );
                return true; // Return true to indicate error
            }
        }
        false // No missing files
    });

    if has_missing_files { // Early return from action handler
    }
}

pub fn handle_clear_error(mut signal: PostSignal) {
    signal.with_mut(|state| {
        state.error_message = None;
    });
}
