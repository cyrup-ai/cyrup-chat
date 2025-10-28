//! Main action dispatcher for post actions

use super::super::event_processing::handle_app_event;
use super::{
    content::{handle_clear_error, handle_update_text, handle_update_visibility},
    files::{
        handle_dropped_media, handle_dropped_media_uploaded, handle_dropped_paths,
        handle_file_dialog, handle_file_dialog_done, handle_upload_media_done,
    },
    images::{
        handle_remove_image, handle_show_image_disk, handle_update_image_description,
        handle_update_image_description_result,
    },
    lifecycle::{
        handle_close_action, handle_open_action, handle_post_result, handle_post_submission,
    },
};
use crate::components::post::PostAction;
use crate::environment::Environment;
use dioxus::prelude::*;

// Modern Dioxus signal-based state management
pub type PostSignal = Signal<crate::components::post::State>;

// Modern Dioxus signal-based action handler
pub fn handle_post_action(signal: PostSignal, action: PostAction, environment: &Environment) {
    log::trace!("{action:?}");
    // Note: Window operations will be handled in component context

    match action {
        PostAction::Open(images) => {
            handle_open_action(signal, images, environment);
        }
        PostAction::DroppedPaths(images) => {
            handle_dropped_paths(signal, images);
        }
        PostAction::Close => {
            handle_close_action(signal);
        }
        PostAction::FileDialog => {
            handle_file_dialog(signal, environment);
        }
        PostAction::FileDialogDone(result) => {
            handle_file_dialog_done(signal, result);
        }
        PostAction::UploadMediaDone((image, result)) => {
            handle_upload_media_done(signal, image, result);
        }
        PostAction::RemoveImage(index) => {
            handle_remove_image(signal, index);
        }
        PostAction::ShowImageDisk(index) => {
            handle_show_image_disk(signal, index);
        }
        PostAction::UpdateVisibility(vis) => {
            handle_update_visibility(signal, vis);
        }
        PostAction::UpdateImageDescription(index, desc) => {
            handle_update_image_description(signal, index, desc);
        }
        PostAction::UpdateImageDescriptionResult(result) => {
            handle_update_image_description_result(signal, result);
        }
        PostAction::UpdateText(text) => {
            handle_update_text(signal, text, environment);
        }
        PostAction::Post => {
            handle_post_submission(signal);
        }
        PostAction::PostResult(ref result) => {
            handle_post_result(signal, result, environment);
        }
        PostAction::AppEvent(ref event) => {
            handle_app_event(signal, event);
        }
        PostAction::DroppedMedia(m) => {
            handle_dropped_media(signal, m);
        }
        PostAction::DroppedMediaUploaded(m) => {
            handle_dropped_media_uploaded(signal, m);
        }
        PostAction::ClearError => {
            handle_clear_error(signal);
        }
    }
}
