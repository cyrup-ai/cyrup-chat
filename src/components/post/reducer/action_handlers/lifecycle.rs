//! Post lifecycle management - initialization, submission, and completion

use super::super::validation::validate_text;
use super::dispatcher::{PostSignal, handle_post_action};
use crate::components::loggedin::reducer::actions::post::PostError;
use crate::components::post::PostAction;
use crate::components::post::{PostKind, Visibility};
use crate::environment::Environment;
use dioxus::prelude::*;

pub fn handle_open_action(
    mut signal: PostSignal,
    images: Vec<std::path::PathBuf>,
    environment: &Environment,
) {
    // Note: Menu updates will be handled in component context
    let (image_paths, should_handle_images) = signal.with_mut(|state| {
        state.text = match state.kind {
            PostKind::Post => "".to_string(),
            PostKind::Reply(ref s) | PostKind::ReplyPrivate(ref s) => {
                // have to ignore our own username for a mention
                let compare = format!("@{}", state.account.username);
                let f: Vec<_> = s
                    .mentions
                    .iter()
                    .filter(|e| *e != &compare)
                    .cloned()
                    .collect();
                let mut others = f.join(" ");
                if !f.is_empty() {
                    others.push(' ');
                }
                format!("@{} {others}", s.account.acct)
            }
        };
        if matches!(state.kind, PostKind::ReplyPrivate(_)) {
            state.visibility = Some(Visibility::Direct);
        }
        let instance = environment.model.instance();
        state.validity = validate_text(Some(instance.clone()), &state.text);

        let imgs = if images.is_empty() {
            state.image_paths.clone()
        } else {
            images.clone()
        };

        (imgs.clone(), !imgs.is_empty())
    });

    if should_handle_images {
        // Directly handle dropped paths instead of Effect::action
        handle_post_action(signal, PostAction::DroppedPaths(image_paths), environment);
    }
}

pub fn handle_close_action(mut signal: PostSignal) {
    // Note: Menu updates and window operations will be handled in component context
    signal.with_mut(|_state| {
        // Any state cleanup can be done here
    });
}

pub fn handle_post_submission(mut signal: PostSignal) {
    // Validate post state before submitting
    let validation_error = signal.with(|state| {
        // Check if text is too short or too long
        let text_len = state.text.trim().len();
        if text_len == 0 && state.image_paths.is_empty() {
            Some("Cannot post empty content".to_string())
        } else if text_len > 500 {
            Some("Post text is too long".to_string())
        } else {
            // Check that all attached files exist
            for path in &state.image_paths {
                if !path.exists() {
                    return Some(format!("Attached file no longer exists: {path:?}"));
                }
            }
            None
        }
    });

    if let Some(error) = validation_error {
        log::error!("Post creation failed: {error}");
        signal.with_mut(|state| {
            state.error_message = Some(
                PostError::creation_failed(format!("Cannot create post: {error}")).to_string(),
            );
        });
        return;
    }

    signal.with_mut(|state| {
        state.posting = true;
    });
    // Note: Post submission will be handled in component with use_future
}

pub fn handle_post_result(
    mut signal: PostSignal,
    result: &Result<crate::environment::model::Status, String>,
    environment: &Environment,
) {
    signal.with_mut(|state| {
        state.posting = false;
        match result {
            Ok(_) => {
                // Success - component will handle close
            }
            Err(e) => {
                state.error_message = Some(PostError::completion_failed(e.clone()).to_string());
            }
        }
    });
    // If successful, trigger close action
    if result.is_ok() {
        handle_post_action(signal, PostAction::Close, environment);
    }
}
