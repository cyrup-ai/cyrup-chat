//! Image management and metadata operations

use super::dispatcher::PostSignal;
use crate::components::loggedin::reducer::actions::post::PostError;
use dioxus::prelude::*;

pub fn handle_remove_image(mut signal: PostSignal, index: usize) {
    // Remove image at the specified index
    signal.with_mut(|state| {
        if index < state.image_paths.len() {
            state.image_paths.remove(index);
        }
        if index < state.images.len() {
            state.images.remove(index);
        }
    });
    signal.with_mut(|state| {
        if index < state.images.len() {
            state.images.remove(index);
        }
    });
}

pub fn handle_show_image_disk(signal: PostSignal, index: usize) {
    let image_path = signal.with(|state| state.images.get(index).map(|img| img.path.clone()));
    if let Some(path) = image_path {
        crate::environment::platform::open_file(&path);
    }
}

pub fn handle_update_image_description(mut signal: PostSignal, index: usize, desc: String) {
    let server_id = signal.with_mut(|state| {
        if let Some(entry) = state.images.get_mut(index) {
            entry.description = Some(desc.clone());
            entry.server_id.clone()
        } else {
            None
        }
    });
    // Update media description on server using the media API
    if let Some(id) = server_id {
        log::debug!("Updating media description for server ID: {id}");

        // Spawn async task to update media description
        let description_clone = desc.clone();
        let id_clone = id.clone();
        spawn(async move {
            // Use the environment's model to update media description
            match crate::app::context::try_use_environment() {
                Ok(env) => {
                    let model = env.with(|e| e.model.clone());
                    let result = model.update_media(id_clone, Some(description_clone));

                    match result.await {
                        Ok(_) => log::debug!("Media description updated successfully"),
                        Err(e) => log::error!("Failed to update media description: {e}"),
                    }
                }
                Err(e) => {
                    log::error!("Cannot update media description - environment unavailable: {e}")
                }
            }
        });
    }
}

pub fn handle_update_image_description_result(mut signal: PostSignal, result: Result<(), String>) {
    signal.with_mut(|state| {
        if let Err(e) = result {
            state.error_message = Some(
                PostError::FileAttachmentFailed(format!("Could not change description: {e:?}"))
                    .to_string(),
            );
        }
    });
}
