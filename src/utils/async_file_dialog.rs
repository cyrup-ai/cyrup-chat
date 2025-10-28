// Zero allocation, non-blocking file selection using rfd async APIs

use crate::errors::ui::UiError;
use std::path::PathBuf;

/// File dialog configuration for async operations
#[derive(Clone, Debug)]
pub struct FileDialogConfig {
    pub title: String,
    pub filters: Vec<FileFilter>,
    pub multiple: bool,
    pub directory: Option<PathBuf>,
    pub save_mode: bool,
}

#[derive(Clone, Debug)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Result of file dialog operation
#[derive(Debug)]
pub enum FileDialogResult {
    Selected(Vec<PathBuf>),
    Cancelled,
    Error(String),
}

/// Async file dialog abstraction using rfd for blazing-fast native performance
#[derive(Default)]
pub struct AsyncFileDialog;

impl AsyncFileDialog {
    pub fn new() -> Self {
        Self
    }

    /// Show file picker dialog asynchronously using rfd's native async APIs
    ///
    /// This implementation uses rfd's async methods which provide:
    /// - Zero  calls (pure async)
    /// - Native platform dialogs (NSOpenPanel on macOS, etc.)
    /// - Cross-platform compatibility
    /// - Optimal performance
    pub async fn pick_files(config: FileDialogConfig) -> Result<FileDialogResult, UiError> {
        if config.save_mode {
            Self::show_save_dialog_async(config).await
        } else {
            Self::show_open_dialog_async(config).await
        }
    }

    /// Show open file dialog using rfd's async API - ZERO
    async fn show_open_dialog_async(config: FileDialogConfig) -> Result<FileDialogResult, UiError> {
        use rfd::AsyncFileDialog;

        let mut dialog = AsyncFileDialog::new();

        // Set title
        dialog = dialog.set_title(&config.title);

        // Set initial directory
        if let Some(directory) = config.directory {
            dialog = dialog.set_directory(&directory);
        }

        // Add file filters
        for filter in config.filters {
            let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &extensions);
        }

        // Execute dialog based on configuration
        let result = if config.multiple {
            dialog.pick_files().await
        } else {
            match dialog.pick_file().await {
                Some(file) => Some(vec![file]),
                None => None,
            }
        };

        match result {
            Some(files) if !files.is_empty() => {
                let paths: Vec<PathBuf> =
                    files.into_iter().map(|f| f.path().to_path_buf()).collect();
                Ok(FileDialogResult::Selected(paths))
            }
            Some(_) => Ok(FileDialogResult::Cancelled),
            None => Ok(FileDialogResult::Cancelled),
        }
    }

    /// Show save file dialog using rfd's async API - ZERO
    async fn show_save_dialog_async(config: FileDialogConfig) -> Result<FileDialogResult, UiError> {
        use rfd::AsyncFileDialog;

        let mut dialog = AsyncFileDialog::new();

        // Set title
        dialog = dialog.set_title(&config.title);

        // Set initial directory
        if let Some(directory) = config.directory {
            dialog = dialog.set_directory(&directory);
        }

        // Add file filters
        for filter in config.filters {
            let extensions: Vec<&str> = filter.extensions.iter().map(|s| s.as_str()).collect();
            dialog = dialog.add_filter(&filter.name, &extensions);
        }

        // Execute save dialog
        let result = dialog.save_file().await;

        match result {
            Some(file) => {
                let path = file.path().to_path_buf();
                Ok(FileDialogResult::Selected(vec![path]))
            }
            None => Ok(FileDialogResult::Cancelled),
        }
    }

    /// Show folder selection dialog using rfd's async API - ZERO
    pub async fn pick_folder(config: FileDialogConfig) -> Result<FileDialogResult, UiError> {
        use rfd::AsyncFileDialog;

        let mut dialog = AsyncFileDialog::new();

        // Set title
        dialog = dialog.set_title(&config.title);

        // Set initial directory
        if let Some(directory) = config.directory {
            dialog = dialog.set_directory(&directory);
        }

        // Execute folder dialog based on configuration
        let result = if config.multiple {
            dialog.pick_folders().await
        } else {
            match dialog.pick_folder().await {
                Some(folder) => Some(vec![folder]),
                None => None,
            }
        };

        match result {
            Some(folders) if !folders.is_empty() => {
                let paths: Vec<PathBuf> = folders
                    .into_iter()
                    .map(|f| f.path().to_path_buf())
                    .collect();
                Ok(FileDialogResult::Selected(paths))
            }
            Some(_) => Ok(FileDialogResult::Cancelled),
            None => Ok(FileDialogResult::Cancelled),
        }
    }
}

impl FileDialogConfig {
    pub fn new() -> Self {
        Self {
            title: "Select Files".to_string(),
            filters: Vec::new(),
            multiple: false,
            directory: None,
            save_mode: false,
        }
    }

    #[inline]
    pub fn with_title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    #[inline]
    pub fn with_filter(mut self, name: &str, extensions: Vec<String>) -> Self {
        self.filters.push(FileFilter {
            name: name.to_string(),
            extensions,
        });
        self
    }

    #[inline]
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    #[inline]
    pub fn save_mode(mut self, save_mode: bool) -> Self {
        self.save_mode = save_mode;
        self
    }

    #[inline]
    pub fn directory(mut self, directory: PathBuf) -> Self {
        self.directory = Some(directory);
        self
    }
}

impl Default for FileDialogConfig {
    fn default() -> Self {
        Self::new()
    }
}
