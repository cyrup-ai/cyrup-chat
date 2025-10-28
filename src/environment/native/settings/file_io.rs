//! File I/O operations for settings data persistence
//!
//! This module handles all filesystem operations including reading, writing,
//! and path management for settings data storage.

use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{from_slice, to_string_pretty};
use std::path::PathBuf;

/// Error type for data directory access failures
#[derive(Debug)]
pub enum DataDirectoryError {
    /// Failed to determine project directories
    ProjectDirsNotFound,
    /// Failed to create data directory
    DirectoryCreationFailed(std::io::Error),
}

impl std::fmt::Display for DataDirectoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProjectDirsNotFound => {
                write!(f, "Unable to determine system project directories")
            }
            Self::DirectoryCreationFailed(e) => write!(f, "Failed to create data directory: {e}"),
        }
    }
}

impl std::error::Error for DataDirectoryError {}

/// Try to get the data directory with proper error handling
pub async fn try_data_directory() -> Result<PathBuf, DataDirectoryError> {
    use directories_next::ProjectDirs;

    let proj_dirs = ProjectDirs::from("com", "stylemac", "cyrup")
        .ok_or(DataDirectoryError::ProjectDirsNotFound)?;

    let dirs = proj_dirs.config_dir().to_path_buf();

    if !dirs.exists() {
        tokio::fs::create_dir_all(&dirs)
            .await
            .map_err(DataDirectoryError::DirectoryCreationFailed)?;
        log::info!("Created data directory: {}", dirs.display());
    }

    Ok(dirs)
}

/// Get data directory - no fallbacks, proper directory or fail
/// Application must have access to a valid data directory to function
pub async fn data_directory() -> PathBuf {
    try_data_directory().await.unwrap_or_else(|e| {
        panic!("Failed to access data directory: {e}. Application cannot continue without proper data storage.");
    })
}

/// Generic read function for deserializing JSON data from files
pub async fn read<T: DeserializeOwned>(name: &str) -> Result<Option<T>, String> {
    let data_path = data_directory().await.join(name);
    if !data_path.exists() {
        return Ok(None);
    };
    let data = tokio::fs::read(&data_path)
        .await
        .map_err(|e| format!("Could not read {}: {e:?}", data_path.display()))?;
    let obj: T =
        from_slice(&data).map_err(|e| format!("Could not parse {}: {e:?}", data_path.display()))?;
    Ok(Some(obj))
}

/// Generic write function for serializing data to JSON files
pub async fn write<T: Serialize>(name: &str, value: &T) -> Result<(), String> {
    let data_path = data_directory().await.join(name);
    let data = to_string_pretty(&value).map_err(|e| format!("Could not parse value:{e:?}"))?;
    tokio::fs::write(&data_path, data)
        .await
        .map_err(|e| format!("Could not write to {}: {e:?}", data_path.display()))?;
    Ok(())
}
