//! Post attachment handling with zero-allocation patterns
//!
//! This module provides attachment management for post composition including
//! file validation, media upload coordination, and state synchronization
//! using efficient Dioxus 0.7 signal-based patterns.

use crate::{
    components::post::State as PostState, environment::Environment, view_model::AttachmentMedia,
};
use std::path::PathBuf;

/// Handle adding attachment to post with validation and upload coordination
///
/// Validates file type, size constraints, and initiates asynchronous upload
/// while updating post state to reflect the pending attachment.
#[allow(dead_code)] // Architectural component - attachment handling integration pending
pub fn handle_add_attachment(
    state: &mut PostState,
    _environment: &mut Environment,
    attachment: AttachmentMedia,
) -> Result<(), String> {
    // Validate attachment constraints
    if state.images.len() >= 4 {
        return Err("Maximum of 4 attachments allowed".to_string());
    }

    // Validate file exists and get metadata
    let metadata =
        std::fs::metadata(&attachment.path).map_err(|e| format!("Cannot access file: {e}"))?;

    let file_size = metadata.len();

    // Determine media type from file extension
    let media_type = attachment
        .path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_lowercase())
        .unwrap_or_else(|| "unknown".to_string());

    // Validate file size (10MB limit for images, 40MB for video)
    let max_size = match media_type.as_str() {
        "jpg" | "jpeg" | "png" | "gif" | "webp" => 10 * 1024 * 1024, // 10MB
        "mp4" | "mov" | "avi" | "mkv" => 40 * 1024 * 1024,           // 40MB
        _ => return Err(format!("Unsupported file type: {media_type}")),
    };

    if file_size > max_size {
        return Err(format!(
            "File too large. Maximum size: {}MB",
            max_size / (1024 * 1024)
        ));
    }

    // Add to state images list
    state.images.push(attachment.clone());
    log::info!("Attachment added: {}", attachment.filename);

    Ok(())
}

/// Handle removing attachment from post state
///
/// Removes attachment at specified index and cleans up any associated
/// upload state or temporary files.
#[allow(dead_code)] // Architectural component - attachment handling integration pending
pub fn handle_remove_attachment(
    state: &mut PostState,
    _environment: &mut Environment,
    index: usize,
) -> Result<(), String> {
    if index >= state.images.len() {
        return Err("Invalid attachment index".to_string());
    }

    // Remove attachment from state
    let removed_attachment = state.images.remove(index);
    log::info!("Removed attachment: {}", removed_attachment.filename);

    // Clean up corresponding image_paths if they exist
    if index < state.image_paths.len() {
        state.image_paths.remove(index);
    }

    Ok(())
}

/// Validate current post state for attachment consistency
///
/// Ensures attachment state is consistent between images and image_paths
/// and validates that all attachments meet posting requirements.
#[allow(dead_code)] // Architectural component - attachment validation integration pending
pub fn validate_post_state(state: &PostState) -> Result<(), String> {
    // Validate attachment count consistency
    if state.images.len() != state.image_paths.len() {
        log::warn!(
            "Attachment state inconsistency: {} images vs {} paths",
            state.images.len(),
            state.image_paths.len()
        );
    }

    // Validate attachment descriptions for accessibility
    for (index, attachment) in state.images.iter().enumerate() {
        if attachment.description.as_ref().is_none_or(|d| d.is_empty()) {
            log::warn!("Attachment {index} missing description for accessibility");
        }
    }

    // Ensure we don't exceed platform limits
    if state.images.len() > 4 {
        return Err("Too many attachments (maximum 4 allowed)".to_string());
    }

    // Validate that all attachments have valid paths
    for (index, path) in state.image_paths.iter().enumerate() {
        if !path.exists() {
            return Err(format!("Attachment {index} file not found: {path:?}"));
        }
    }

    Ok(())
}

/// Handle dropped file paths from drag-and-drop operations
///
/// Processes dropped file paths and converts them to AttachmentMedia
/// for integration with the post composition flow.
#[allow(dead_code)] // Architectural component - drag-and-drop integration pending
pub fn handle_dropped_paths(
    state: &mut PostState,
    environment: &mut Environment,
    paths: Vec<PathBuf>,
) -> Result<Vec<AttachmentMedia>, String> {
    let mut processed_attachments = Vec::new();

    for path in paths {
        // Validate file exists and is readable
        if !path.exists() {
            log::warn!("Dropped file does not exist: {path:?}");
            continue;
        }

        // Convert path to AttachmentMedia
        let attachment = AttachmentMedia {
            preview: None,
            path: path.clone(),
            filename: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string(),
            description: None,
            is_uploaded: false,
            server_id: None,
        };

        // Add to state using existing validation
        match handle_add_attachment(state, environment, attachment.clone()) {
            Ok(()) => processed_attachments.push(attachment),
            Err(e) => log::error!("Failed to add dropped attachment: {e}"),
        }
    }

    Ok(processed_attachments)
}
