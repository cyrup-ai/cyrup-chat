#![allow(non_snake_case)]
#[allow(dead_code)] // Post composition actions - architectural scaffolding pending integration
use std::path::PathBuf;

use crate::environment::model::{Status, UploadMedia};
use crate::environment::types::AppEvent;
use crate::view_model::AttachmentMedia;

#[allow(clippy::large_enum_variant)]
#[derive(Clone, Debug)]
#[allow(dead_code)] // Post action variants - architectural scaffolding pending integration
pub enum PostAction {
    Open(Vec<PathBuf>),
    Close,
    FileDialog,
    FileDialogDone(Option<AttachmentMedia>),
    UploadMediaDone((AttachmentMedia, Result<UploadMedia, String>)),
    RemoveImage(usize),
    ShowImageDisk(usize),
    UpdateImageDescription(usize, String),
    UpdateImageDescriptionResult(Result<(), String>),
    UpdateVisibility(String),
    UpdateText(String),
    Post,
    PostResult(Result<Status, String>),
    ClearError,
    AppEvent(AppEvent),
    DroppedPaths(Vec<PathBuf>),
    DroppedMedia(Vec<AttachmentMedia>),
    DroppedMediaUploaded(Vec<(Result<UploadMedia, String>, AttachmentMedia)>),
}
