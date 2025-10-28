// Native platform operations with async-task integration
//
// This module provides async-task based platform operations with direct native async APIs
// for zero-allocation, high-performance platform-specific operations.

pub mod error;

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "windows")]  
pub mod windows;

#[cfg(target_os = "linux")]
pub mod linux;

// Re-export platform-specific implementations
#[cfg(target_os = "macos")]
pub use macos::*;

#[cfg(target_os = "windows")]
pub use windows::*;

#[cfg(target_os = "linux")]
pub use linux::*;

use crate::errors::ui::UiError;
use async_task::{Runnable, Task};
use std::path::PathBuf;
use std::future::Future;

/// Configuration for text area operations
#[derive(Debug, Clone)]
pub struct TextAreaConfig {
    pub placeholder: Option<String>,
    pub initial_text: Option<String>,
    pub max_length: Option<usize>,
    pub multiline: bool,
    pub readonly: bool,
}

/// Cursor position for text operations
#[derive(Debug, Clone, Copy)]
pub struct CursorPosition {
    pub line: usize,
    pub column: usize,
}

/// Text selection range
#[derive(Debug, Clone, Copy)]
pub struct TextRange {
    pub start: CursorPosition,
    pub end: CursorPosition,
}

/// Async platform operations interface with async-task integration
#[async_trait::async_trait]
pub trait AsyncPlatformOperations {
    /// Configure text area element with async-task execution
    async fn configure_text_area(
        &self,
        element_id: &str,
        config: &TextAreaConfig,
    ) -> Result<(), UiError>;
    
    /// Focus element and position cursor with async-task execution
    async fn focus_element(
        &self,
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> Result<(), UiError>;
    
    /// Setup drag and drop support with async-task execution
    async fn setup_drag_drop(&self, updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>) -> Result<(), UiError>;
    
    /// Get text content from element with async-task execution
    async fn get_text_content(&self, element_id: &str) -> Result<String, UiError>;
    
    /// Set text content in element with async-task execution
    async fn set_text_content(&self, element_id: &str, text: &str) -> Result<(), UiError>;
    
    /// Get cursor position from element with async-task execution
    async fn get_cursor_position(&self, element_id: &str) -> Result<CursorPosition, UiError>;
    
    /// Set cursor position in element with async-task execution
    async fn set_cursor_position(&self, element_id: &str, position: CursorPosition) -> Result<(), UiError>;
    
    /// Get selected text range with async-task execution
    async fn get_selection(&self, element_id: &str) -> Result<Option<TextRange>, UiError>;
    
    /// Set selected text range with async-task execution
    async fn set_selection(&self, element_id: &str, range: TextRange) -> Result<(), UiError>;
    
    /// Spawn platform operation as async task
    fn spawn_platform_operation<F, T>(&self, operation: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static;
    
    /// Execute platform operation with async-task management
    async fn execute_platform_operation<F, T>(&self, operation: F) -> Result<T, UiError>
    where
        F: Future<Output = Result<T, UiError>> + Send + 'static,
        T: Send + 'static;
}

/// Async task executor for platform operations
pub struct AsyncTaskExecutor {
    executor: async_executor::Executor<'static>,
}

impl AsyncTaskExecutor {
    pub fn new() -> Self {
        Self {
            executor: async_executor::Executor::new(),
        }
    }
    
    /// Spawn a platform operation task with async-task
    pub fn spawn_platform_task<F, T>(&self, future: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.executor.spawn(future)
    }
    
    /// Run platform operations with proper async task management
    pub async fn run_platform_operation<F, T>(&self, operation: F) -> Result<T, UiError>
    where
        F: Future<Output = Result<T, UiError>> + Send + 'static,
        T: Send + 'static,
    {
        let task = self.spawn_platform_task(operation);
        task.await
    }
}

impl Default for AsyncTaskExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Get the native platform operations implementation for the current platform
pub fn native_platform() -> impl AsyncPlatformOperations {
    NativePlatform::new()
}

/// Get the async task executor for platform operations
pub fn async_task_executor() -> AsyncTaskExecutor {
    AsyncTaskExecutor::new()
}

impl TextAreaConfig {
    pub fn new() -> Self {
        Self {
            placeholder: None,
            initial_text: None,
            max_length: None,
            multiline: false,
            readonly: false,
        }
    }
    
    pub fn with_placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }
    
    pub fn with_initial_text(mut self, text: impl Into<String>) -> Self {
        self.initial_text = Some(text.into());
        self
    }
    
    pub fn with_max_length(mut self, max_length: usize) -> Self {
        self.max_length = Some(max_length);
        self
    }
    
    pub fn multiline(mut self) -> Self {
        self.multiline = true;
        self
    }
    
    pub fn readonly(mut self) -> Self {
        self.readonly = true;
        self
    }
}

impl CursorPosition {
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }
    
    pub fn start() -> Self {
        Self::new(0, 0)
    }
}

impl TextRange {
    pub fn new(start: CursorPosition, end: CursorPosition) -> Self {
        Self { start, end }
    }
    
    pub fn single_point(position: CursorPosition) -> Self {
        Self::new(position, position)
    }
    
    pub fn is_collapsed(&self) -> bool {
        self.start.line == self.end.line && self.start.column == self.end.column
    }
    
    pub fn length_in_lines(&self) -> usize {
        if self.end.line >= self.start.line {
            self.end.line - self.start.line + 1
        } else {
            0
        }
    }
}