//! File and media handling operations

use super::super::validation::handle_image_upload;
use super::dispatcher::{PostSignal, handle_post_action};
use crate::components::loggedin::reducer::actions::post::PostError;
use crate::components::post::PostAction;
use crate::environment::Environment;
use crate::view_model::AttachmentMedia;
use dioxus::prelude::*;
use megalodon::entities::UploadMedia;

pub fn handle_dropped_paths(mut signal: PostSignal, images: Vec<std::path::PathBuf>) {
    // Add dropped paths directly to state
    signal.with_mut(|state| {
        state.image_paths.extend(images);
        state.dropping_file = false;
    });
    // Note: Image processing to AttachmentMedia will be handled in component with use_future
}

pub fn handle_file_dialog(signal: PostSignal, environment: &Environment) {
    // Handle async file dialog with spawn
    spawn({
        let environment = environment.clone();
        async move {
            let result = crate::environment::platform::open_file_dialog("~").await;
            handle_post_action(signal, PostAction::FileDialogDone(result), &environment);
        }
    });
}

pub fn handle_file_dialog_done(mut signal: PostSignal, result: Option<AttachmentMedia>) {
    if let Some(image) = result {
        signal.with_mut(|state| {
            state.images.push(image.clone());
        });
        // Note: Media upload will be handled in component with use_future
    }
}

pub fn handle_upload_media_done(
    mut signal: PostSignal,
    image: AttachmentMedia,
    result: Result<UploadMedia, String>,
) {
    signal.with_mut(|state| {
        if let Some(e) = handle_image_upload(image, result, &mut state.images) {
            state.error_message = Some(PostError::FileAttachmentFailed(e).to_string());
        }
    });
}

pub fn handle_dropped_media(mut signal: PostSignal, m: Vec<AttachmentMedia>) {
    signal.with_mut(|state| {
        state.dropping_file = false;
        state.images.extend(m.clone());
    });
    // Note: Media upload will be handled in component with use_future
}

pub fn handle_dropped_media_uploaded(
    mut signal: PostSignal,
    m: Vec<(Result<UploadMedia, String>, AttachmentMedia)>,
) {
    signal.with_mut(|state| {
        for (result, image) in m {
            handle_image_upload(image, result, &mut state.images);
        }
    });
}
