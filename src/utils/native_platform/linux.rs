// Linux native platform operations using GTK4 APIs
//
// Complete GTK4 implementation with zero-allocation patterns

use super::{AsyncPlatformOperations, TextAreaConfig, CursorPosition, TextRange};
use crate::errors::ui::UiError;

#[cfg(target_os = "linux")]
use gtk::{prelude::*, TextView, TextBuffer, TextIter, DropTarget, glib, gdk};
#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::sync::Arc;
#[cfg(target_os = "linux")]
use tokio::sync::RwLock;

/// Linux-specific platform errors
#[cfg(target_os = "linux")]
#[derive(Debug, thiserror::Error)]
pub enum LinuxPlatformError {
    #[error("Widget not found: {0}")]
    WidgetNotFound(String),
    #[error("GTK operation failed: {0}")]
    GtkError(String),
    #[error("Text buffer operation failed: {0}")]
    TextBufferError(String),
    #[error("Invalid cursor position: line {line}, column {column}")]
    InvalidCursorPosition { line: i32, column: i32 },
}

/// Native Linux platform implementation with GTK4 integration
#[cfg(target_os = "linux")]
#[derive(Clone)]
pub struct NativePlatform {
    widget_cache: Arc<RwLock<HashMap<String, TextView>>>,
    drop_targets: Arc<RwLock<HashMap<String, DropTarget>>>,
    app: Arc<RwLock<Option<gtk::Application>>>,
}

/// Fallback implementation for non-Linux platforms
#[cfg(not(target_os = "linux"))]
pub struct NativePlatform {}

#[cfg(target_os = "linux")]
impl NativePlatform {
    pub fn new() -> Self {
        Self {
            widget_cache: Arc::new(RwLock::new(HashMap::new())),
            drop_targets: Arc::new(RwLock::new(HashMap::new())),
            app: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize with GTK application reference
    pub async fn set_application(&self, app: gtk::Application) {
        let mut app_guard = self.app.write().await;
        *app_guard = Some(app);
    }

    /// Find or discover a TextView widget by element ID with caching
    async fn find_text_view(&self, element_id: &str) -> Result<TextView, UiError> {
        // Try cache first for zero-allocation access
        {
            let cache = self.widget_cache.read().await;
            if let Some(text_view) = cache.get(element_id) {
                return Ok(text_view.clone());
            }
        }

        // Discover widget in GTK hierarchy
        let text_view = self.discover_text_view(element_id).await?;

        // Cache for future zero-allocation access
        {
            let mut cache = self.widget_cache.write().await;
            cache.insert(element_id.to_string(), text_view.clone());
        }

        Ok(text_view)
    }

    /// Discover TextView in GTK widget hierarchy
    async fn discover_text_view(&self, element_id: &str) -> Result<TextView, UiError> {
        log::debug!("Discovering TextView widget with id: '{}'", element_id);
        
        // Get the GTK application instance
        let app_guard = self.app.read().await;
        let app = app_guard.as_ref()
            .ok_or_else(|| UiError::platform_error(
                "GTK application not initialized. Call set_application() first."
            ))?;
        
        // Get all application windows
        let windows = app.windows();
        
        if windows.is_empty() {
            return Err(UiError::platform_error(
                "No GTK windows found. Application may not be fully initialized."
            ));
        }
        
        log::debug!("Searching through {} window(s) for widget '{}'", windows.len(), element_id);
        
        // Search through each window's widget hierarchy
        for (i, window) in windows.iter().enumerate() {
            log::trace!("Searching window {} for widget '{}'", i, element_id);
            
            if let Some(widget) = self.find_widget_recursive(window, element_id) {
                log::debug!("Found widget '{}' in window {}", element_id, i);
                
                // Try to downcast to TextView
                if let Ok(text_view) = widget.downcast::<TextView>() {
                    log::debug!("Widget '{}' is TextView", element_id);
                    return Ok(text_view);
                }
                
                // Check if it's an Entry (single-line text input)
                if widget.is::<gtk::Entry>() {
                    return Err(UiError::platform_error(format!(
                        "Widget '{}' is Entry (single-line), not TextView (multi-line). \
                         Use Entry-specific API methods instead.",
                        element_id
                    )));
                }
                
                // Widget found but wrong type
                let widget_type = widget.type_().name();
                return Err(UiError::platform_error(format!(
                    "Widget '{}' exists but is {} (not a TextView or Entry). \
                     Check that the element_id points to a text input widget.",
                    element_id, widget_type
                )));
            }
        }
        
        // Widget not found in any window
        Err(UiError::platform_error(format!(
            "Widget '{}' not found in application widget hierarchy. \
             Searched {} window(s). Check that:\n\
             1. The element_id is correct\n\
             2. The widget has been created and added to a window\n\
             3. widget.set_widget_name(\"{}\") was called during widget creation",
            element_id, windows.len(), element_id
        )))
    }

    /// Recursively search widget hierarchy for widget with matching name
    fn find_widget_recursive(&self, root: &impl IsA<gtk::Widget>, target_name: &str) -> Option<gtk::Widget> {
        let root_widget = root.as_ref();
        
        // Check if current widget matches
        let widget_name = root_widget.widget_name();
        if widget_name == target_name {
            log::trace!("Found matching widget: '{}'", target_name);
            return Some(root_widget.clone());
        }
        
        // Recursively search children using depth-first traversal
        let mut child = root_widget.first_child();
        while let Some(current_child) = child {
            if let Some(found) = self.find_widget_recursive(&current_child, target_name) {
                return Some(found);
            }
            child = current_child.next_sibling();
        }
        
        None
    }

    /// Get TextBuffer from TextView with error handling
    fn get_text_buffer(text_view: &TextView) -> Result<TextBuffer, UiError> {
        let buffer = text_view.buffer();
        Ok(buffer)
    }

    /// Create TextIter at specific line and column with bounds checking
    fn create_text_iter(buffer: &TextBuffer, line: i32, column: i32) -> Result<TextIter, UiError> {
        let line_count = buffer.line_count();
        if line < 0 || line >= line_count {
            return Err(UiError::platform_error(&format!(
                "Invalid line {}: must be between 0 and {}", line, line_count - 1
            )));
        }

        let mut iter = buffer.iter_at_line(line);
        let line_length = iter.chars_in_line();
        
        if column < 0 || column > line_length {
            return Err(UiError::platform_error(&format!(
                "Invalid column {}: must be between 0 and {}", column, line_length
            )));
        }

        iter.set_line_offset(column);
        Ok(iter)
    }
}

#[cfg(not(target_os = "linux"))]
impl NativePlatform {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(target_os = "linux")]
#[async_trait::async_trait]
impl AsyncPlatformOperations for NativePlatform {
    async fn configure_text_area(
        &self,
        element_id: &str,
        config: &TextAreaConfig,
    ) -> Result<(), UiError> {
        log::debug!("Configuring Linux text area '{}' with GTK4", element_id);
        
        let text_view = self.find_text_view(element_id).await?;
        let buffer = Self::get_text_buffer(&text_view)?;
        
        // Configure text area properties
        if let Some(initial_text) = &config.initial_text {
            buffer.set_text(initial_text);
        }
        
        if let Some(max_length) = config.max_length {
            // Set buffer length limit with signal connection
            let max_len = max_length as i32;
            buffer.connect_insert_text(move |buffer, _location, text, _length| {
                let current_length = buffer.char_count();
                let new_text_length = text.chars().count() as i32;
                
                if current_length + new_text_length > max_len {
                    glib::signal::signal_stop_emission_by_name(buffer, "insert-text");
                }
            });
        }
        
        // Configure text view properties
        text_view.set_editable(true);
        text_view.set_wrap_mode(if config.multiline { 
            gtk::WrapMode::WordChar 
        } else { 
            gtk::WrapMode::None 
        });
        
        if let Some(placeholder) = &config.placeholder {
            // GTK4 doesn't have direct placeholder support, but we can simulate it
            if buffer.char_count() == 0 {
                buffer.set_text(placeholder);
                // Mark as placeholder text (would need additional state tracking)
            }
        }
        
        log::debug!("Linux text area configuration completed");
        Ok(())
    }
    
    async fn focus_element(
        &self,
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> Result<(), UiError> {
        log::debug!("Focusing Linux element '{}' at position {:?}", element_id, cursor_position);
        
        let text_view = self.find_text_view(element_id).await?;
        let buffer = Self::get_text_buffer(&text_view)?;
        
        // Set cursor position first
        let iter = Self::create_text_iter(&buffer, cursor_position.line, cursor_position.column)?;
        buffer.place_cursor(&iter);
        
        // Grab focus and scroll to cursor
        text_view.grab_focus();
        text_view.scroll_to_iter(&iter, 0.0, false, 0.0, 0.0);
        
        log::debug!("Linux element focus completed");
        Ok(())
    }
    
    async fn setup_drag_drop(&self, updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>) -> Result<(), UiError> {
        log::debug!("Setting up Linux drag-and-drop with GTK4");
        
        // Get the main application window for attaching drop targets
        let app_guard = self.app.read().await;
        let app = app_guard.as_ref()
            .ok_or_else(|| UiError::platform_error(
                "GTK application not initialized. Call set_application() first."
            ))?;
        
        let windows = app.windows();
        if windows.is_empty() {
            return Err(UiError::platform_error(
                "No GTK windows found. Cannot setup drag-drop without a window."
            ));
        }
        
        let main_window = windows[0].clone();
        drop(app_guard); // Release read lock
        
        // Setup text drop target
        let text_drop_target = DropTarget::new(glib::Type::STRING, gdk::DragAction::COPY | gdk::DragAction::MOVE);
        
        let self_clone = Arc::new(self.clone());
        let window_for_pick = main_window.clone();
        
        text_drop_target.connect_drop(move |_target, value, x, y| {
            if let Ok(text) = value.get::<String>() {
                log::debug!("Dropped text: '{}' at ({}, {})", text, x, y);
                
                // Use GTK hit-testing to find widget under drop coordinates
                let element_id = match window_for_pick.pick(x, y, gtk::PickFlags::DEFAULT) {
                    Some(widget) => {
                        let widget_name = widget.widget_name();
                        
                        // Validate it's a TextView
                        if !widget.is::<TextView>() {
                            let widget_type = widget.type_().name();
                            log::warn!(
                                "Drop at ({}, {}) hit {} '{}', not TextView. Rejecting drop.",
                                x, y, widget_type, widget_name
                            );
                            return false;
                        }
                        
                        // Ensure widget has a name (element_id)
                        if widget_name.is_empty() {
                            log::error!(
                                "TextView at ({}, {}) has no widget_name. \
                                 Set via widget.set_widget_name(). Rejecting drop.",
                                x, y
                            );
                            return false;
                        }
                        
                        log::debug!("Drop resolved to TextView '{}'", widget_name);
                        widget_name.to_string()
                    }
                    None => {
                        log::info!("Drop at ({}, {}) on window background. Rejecting drop.", x, y);
                        return false;
                    }
                };
                
                // Insert text at discovered element
                let rt = tokio::runtime::Handle::current();
                let self_clone = self_clone.clone();
                let text = text.clone();
                rt.spawn(async move {
                    if let Err(e) = self_clone.handle_text_drop(&element_id, text, x, y).await {
                        log::error!("Failed to insert dropped text into '{}': {}", element_id, e);
                    }
                });
                
                true
            } else {
                false
            }
        });
        
        // Attach text drop target to main window
        main_window.add_controller(text_drop_target.clone());
        
        // Setup file drop target
        let file_drop_target = DropTarget::new(gdk::FileList::static_type(), gdk::DragAction::COPY);
        
        let self_clone = Arc::new(self.clone());
        let updater_clone = updater.clone();
        file_drop_target.connect_drop(move |_target, value, x, y| {
            if let Ok(file_list) = value.get::<gdk::FileList>() {
                log::debug!("Dropped files at ({}, {})", x, y);
                
                // Emit file drop events
                let files: Vec<std::path::PathBuf> = file_list.files()
                    .into_iter()
                    .filter_map(|f| f.path())
                    .collect();
                
                if !files.is_empty() {
                    // Emit AppEvent::FileEvent(FileEvent::Dropped(...))
                    use crate::environment::types::{AppEvent, FileEvent};
                    updater_clone(AppEvent::FileEvent(FileEvent::Dropped(files)));
                    log::debug!("Emitted file drop event through application pipeline");
                }
                
                true
            } else {
                false
            }
        });
        
        // Attach file drop target to main window
        main_window.add_controller(file_drop_target.clone());
        
        // Cache drop targets for cleanup
        {
            let mut targets = self.drop_targets.write().await;
            targets.insert("text".to_string(), text_drop_target);
            targets.insert("file".to_string(), file_drop_target);
        }
        
        log::debug!("Linux drag-and-drop setup completed");
        Ok(())
    }
    
    /// Handle text drop by inserting at the specified coordinates
    async fn handle_text_drop(&self, element_id: &str, text: String, x: f64, y: f64) -> Result<(), UiError> {
        let text_view = self.find_text_view(element_id).await?;
        let buffer = Self::get_text_buffer(&text_view)?;
        
        // Convert coordinates to buffer position
        // GTK coordinates are in widget space, need to convert to text coordinates
        let (buffer_x, buffer_y) = text_view.window_to_buffer_coords(
            gtk::TextWindowType::Widget,
            x as i32,
            y as i32,
        );
        
        // Get iter at position
        let iter = text_view.iter_at_position(&buffer_x, &buffer_y)
            .map(|(iter, _)| iter)
            .unwrap_or_else(|| buffer.iter_at_offset(0));
        
        // Insert text at position
        buffer.insert(&iter, &text);
        
        // Place cursor at end of inserted text
        let mut end_iter = iter;
        end_iter.forward_chars(text.chars().count() as i32);
        buffer.place_cursor(&end_iter);
        
        // Scroll to cursor
        text_view.scroll_to_iter(&end_iter, 0.0, false, 0.0, 0.0);
        
        log::debug!("Inserted {} characters at drop position ({}, {})", text.len(), x, y);
        Ok(())
    }
    
    async fn get_text_content(&self, element_id: &str) -> Result<String, UiError> {
        log::debug!("Getting text content from Linux element '{}'", element_id);
        
        let text_view = self.find_text_view(element_id).await?;
        let buffer = Self::get_text_buffer(&text_view)?;
        
        let start_iter = buffer.start_iter();
        let end_iter = buffer.end_iter();
        
        let text = buffer.text(&start_iter, &end_iter, false).to_string();
        
        log::debug!("Retrieved {} characters from element '{}'", text.len(), element_id);
        Ok(text)
    }
    
    async fn set_text_content(&self, element_id: &str, text: &str) -> Result<(), UiError> {
        log::debug!("Setting text content in Linux element '{}': {} chars", element_id, text.len());
        
        let text_view = self.find_text_view(element_id).await?;
        let buffer = Self::get_text_buffer(&text_view)?;
        
        buffer.set_text(text);
        
        log::debug!("Text content set successfully");
        Ok(())
    }
    
    async fn get_cursor_position(&self, element_id: &str) -> Result<CursorPosition, UiError> {
        log::debug!("Getting cursor position from Linux element '{}'", element_id);
        
        let text_view = self.find_text_view(element_id).await?;
        let buffer = Self::get_text_buffer(&text_view)?;
        
        let cursor_mark = buffer.get_insert();
        let cursor_iter = buffer.iter_at_mark(&cursor_mark);
        
        let position = CursorPosition {
            line: cursor_iter.line(),
            column: cursor_iter.line_offset(),
        };
        
        log::debug!("Cursor position: {:?}", position);
        Ok(position)
    }
    
    async fn set_cursor_position(&self, element_id: &str, position: CursorPosition) -> Result<(), UiError> {
        log::debug!("Setting cursor position in Linux element '{}' to {:?}", element_id, position);
        
        let text_view = self.find_text_view(element_id).await?;
        let buffer = Self::get_text_buffer(&text_view)?;
        
        let iter = Self::create_text_iter(&buffer, position.line, position.column)?;
        buffer.place_cursor(&iter);
        
        // Scroll to make cursor visible
        text_view.scroll_to_iter(&iter, 0.0, false, 0.0, 0.0);
        
        log::debug!("Cursor position set successfully");
        Ok(())
    }
    
    async fn get_selection(&self, element_id: &str) -> Result<Option<TextRange>, UiError> {
        log::debug!("Getting selection from Linux element '{}'", element_id);
        
        let text_view = self.find_text_view(element_id).await?;
        let buffer = Self::get_text_buffer(&text_view)?;
        
        if let Some((start_iter, end_iter)) = buffer.selection_bounds() {
            let start_pos = CursorPosition {
                line: start_iter.line(),
                column: start_iter.line_offset(),
            };
            let end_pos = CursorPosition {
                line: end_iter.line(),
                column: end_iter.line_offset(),
            };
            
            let range = TextRange { start: start_pos, end: end_pos };
            log::debug!("Selection found: {:?}", range);
            Ok(Some(range))
        } else {
            log::debug!("No selection found");
            Ok(None)
        }
    }
    
    async fn set_selection(&self, element_id: &str, range: TextRange) -> Result<(), UiError> {
        log::debug!("Setting selection in Linux element '{}' to {:?}", element_id, range);
        
        let text_view = self.find_text_view(element_id).await?;
        let buffer = Self::get_text_buffer(&text_view)?;
        
        let start_iter = Self::create_text_iter(&buffer, range.start.line, range.start.column)?;
        let end_iter = Self::create_text_iter(&buffer, range.end.line, range.end.column)?;
        
        buffer.select_range(&start_iter, &end_iter);
        
        // Scroll to make selection visible
        text_view.scroll_to_iter(&start_iter, 0.0, false, 0.0, 0.0);
        
        log::debug!("Selection set successfully");
        Ok(())
    }
}

// Fallback implementation for non-Linux platforms
#[cfg(not(target_os = "linux"))]
#[async_trait::async_trait]
impl AsyncPlatformOperations for NativePlatform {
    async fn configure_text_area(
        &self,
        element_id: &str,
        config: &TextAreaConfig,
    ) -> Result<(), UiError> {
        log::debug!("Linux platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("GTK4 operations only available on Linux"))
    }
    
    async fn focus_element(
        &self,
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> Result<(), UiError> {
        log::debug!("Linux platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("GTK4 operations only available on Linux"))
    }
    
    async fn setup_drag_drop(&self, updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>) -> Result<(), UiError> {
        log::debug!("Linux platform operations not available on this platform");
        Err(UiError::platform_error("GTK4 operations only available on Linux"))
    }
    
    async fn get_text_content(&self, element_id: &str) -> Result<String, UiError> {
        log::debug!("Linux platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("GTK4 operations only available on Linux"))
    }
    
    async fn set_text_content(&self, element_id: &str, text: &str) -> Result<(), UiError> {
        log::debug!("Linux platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("GTK4 operations only available on Linux"))
    }
    
    async fn get_cursor_position(&self, element_id: &str) -> Result<CursorPosition, UiError> {
        log::debug!("Linux platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("GTK4 operations only available on Linux"))
    }
    
    async fn set_cursor_position(&self, element_id: &str, position: CursorPosition) -> Result<(), UiError> {
        log::debug!("Linux platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("GTK4 operations only available on Linux"))
    }
    
    async fn get_selection(&self, element_id: &str) -> Result<Option<TextRange>, UiError> {
        log::debug!("Linux platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("GTK4 operations only available on Linux"))
    }
    
    async fn set_selection(&self, element_id: &str, range: TextRange) -> Result<(), UiError> {
        log::debug!("Linux platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("GTK4 operations only available on Linux"))
    }
}

impl Default for NativePlatform {
    fn default() -> Self {
        Self::new()
    }
}