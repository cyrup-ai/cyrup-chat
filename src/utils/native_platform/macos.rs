// macOS native platform operations using Cocoa/Foundation APIs
//
// Complete objc2 implementation with NSResponder chain management

use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSTextField, NSTextView, NSRange, NSString, MainThreadMarker,
    NSView, NSDragOperation, NSDraggingInfo, NSPasteboard, NSToolbar, NSToolbarItem,
    NSWindow, NSButton, NSSegmentedControl
};
use objc2_foundation::{NSArray, NSPoint};
use tokio::sync::{RwLock, mpsc};
use std::collections::HashMap;
use async_trait::async_trait;
use async_task::Task;
use std::future::Future;
use log;

use crate::errors::ui::UiError;
use crate::utils::async_platform::{AsyncPlatformOperations, TextAreaConfig, CursorPosition, TextRange};

#[cfg(target_os = "macos")]
use objc2::{define_class, msg_send_id, sel, ClassType, DeclaredClass};
#[cfg(target_os = "macos")]
use objc2_foundation::{NSMutableArray, NSObject};
#[cfg(target_os = "macos")]
use std::sync::Arc;
#[cfg(target_os = "macos")]
use tokio::sync::mpsc;

/// macOS-specific platform errors
#[cfg(target_os = "macos")]
#[derive(Debug, thiserror::Error)]
pub enum MacOSPlatformError {
    #[error("Text control not found: {0}")]
    TextControlNotFound(String),
    #[error("Responder chain error: {0}")]
    ResponderChainError(String),
    #[error("Text operation failed: {0}")]
    TextOperationError(String),
    #[error("Cocoa API error: {0}")]
    CocoaError(String),
    #[error("Invalid cursor position: line {line}, column {column}")]
    InvalidCursorPosition { line: i32, column: i32 },
}

/// Native macOS platform implementation with objc2 integration
#[cfg(target_os = "macos")]
pub struct NativePlatform {
    text_views: RwLock<HashMap<String, Retained<NSTextView>>>,
    text_fields: RwLock<HashMap<String, Retained<NSTextField>>>,
    drag_enabled: RwLock<bool>,
    toolbar_event_sender: RwLock<Option<mpsc::UnboundedSender<ToolbarEvent>>>,
    toolbar_items: RwLock<HashMap<String, Retained<NSToolbarItem>>>,
}

/// Toolbar event types for macOS
#[cfg(target_os = "macos")]
#[derive(Debug, Clone)]
pub enum ToolbarEvent {
    ButtonClicked { item_id: String, button_tag: i32 },
    SegmentChanged { item_id: String, selected_segment: i32 },
    ItemValidation { item_id: String, should_enable: bool },
}

/// Fallback implementation for non-macOS platforms
#[cfg(not(target_os = "macos"))]
pub struct NativePlatform {}

#[cfg(target_os = "macos")]
impl NativePlatform {
    pub fn new() -> Self {
        Self {
            text_views: RwLock::new(HashMap::new()),
            text_fields: RwLock::new(HashMap::new()),
            drag_enabled: RwLock::new(false),
            toolbar_event_sender: RwLock::new(None),
            toolbar_items: RwLock::new(HashMap::new()),
        }
    }
    
    /// Spawn platform operation as async task
    pub fn spawn_platform_operation<F, T>(&self, operation: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (runnable, task) = async_task::spawn(operation, |runnable| {
            // Execute the runnable immediately for platform operations
            runnable.run();
        });
        runnable.run();
        task
    }
    
    /// Execute platform operation with async-task management
    pub async fn execute_platform_operation<F, T>(&self, operation: F) -> Result<T, UiError>
    where
        F: Future<Output = Result<T, UiError>> + Send + 'static,
        T: Send + 'static,
    {
        let task = self.spawn_platform_operation(operation);
        task.await
    }
    
    /// Setup toolbar event handling with type-safe channels
    pub async fn setup_toolbar_events(&self) -> Result<mpsc::UnboundedReceiver<ToolbarEvent>, UiError> {
        let (sender, receiver) = mpsc::unbounded_channel();
        
        let mut toolbar_sender = self.toolbar_event_sender.write().await;
        *toolbar_sender = Some(sender);
        
        log::debug!("macOS toolbar event handling initialized");
        Ok(receiver)
    }
    
    /// Register toolbar item for event handling
    pub async fn register_toolbar_item(&self, item_id: &str, item: Retained<NSToolbarItem>) -> Result<(), UiError> {
        let mut toolbar_items = self.toolbar_items.write().await;
        toolbar_items.insert(item_id.to_string(), item);
        
        log::debug!("Registered toolbar item '{}' for event handling", item_id);
        Ok(())
    }
    
    /// Create toolbar button with event handling
    pub async fn create_toolbar_button(&self, item_id: &str, title: &str, tag: i32) -> Result<Retained<NSToolbarItem>, UiError> {
        let mtm = MainThreadMarker::new().ok_or_else(|| 
            UiError::platform_error("Not running on main thread"))?;
        
        unsafe {
            let toolbar_item = NSToolbarItem::new(mtm);
            let button = NSButton::new(mtm);
            
            let ns_title = NSString::from_str(title);
            button.setTitle(&ns_title);
            button.setTag(tag as isize);
            
            // Set button as toolbar item view
            toolbar_item.setView(Some(&button));
            
            let ns_item_id = NSString::from_str(item_id);
            toolbar_item.setItemIdentifier(&ns_item_id);
            
            // Register for event handling
            self.register_toolbar_item(item_id, toolbar_item.clone()).await?;
            
            log::debug!("Created toolbar button '{}' with title '{}'", item_id, title);
            Ok(toolbar_item)
        }
    }
    
    /// Create segmented control toolbar item
    pub async fn create_toolbar_segmented_control(&self, item_id: &str, segment_titles: &[&str]) -> Result<Retained<NSToolbarItem>, UiError> {
        let mtm = MainThreadMarker::new().ok_or_else(|| 
            UiError::platform_error("Not running on main thread"))?;
        
        unsafe {
            let toolbar_item = NSToolbarItem::new(mtm);
            let segmented_control = NSSegmentedControl::new(mtm);
            
            segmented_control.setSegmentCount(segment_titles.len() as isize);
            
            for (index, title) in segment_titles.iter().enumerate() {
                let ns_title = NSString::from_str(title);
                segmented_control.setLabel_forSegment(&ns_title, index as isize);
            }
            
            toolbar_item.setView(Some(&segmented_control));
            
            let ns_item_id = NSString::from_str(item_id);
            toolbar_item.setItemIdentifier(&ns_item_id);
            
            // Register for event handling
            self.register_toolbar_item(item_id, toolbar_item.clone()).await?;
            
            log::debug!("Created segmented control '{}' with {} segments", item_id, segment_titles.len());
            Ok(toolbar_item)
        }
    }
    
    /// Send toolbar event through channel
    async fn send_toolbar_event(&self, event: ToolbarEvent) -> Result<(), UiError> {
        let toolbar_sender = self.toolbar_event_sender.read().await;
        if let Some(sender) = toolbar_sender.as_ref() {
            sender.send(event).map_err(|e| {
                UiError::platform_error(&format!("Failed to send toolbar event: {}", e))
            })?;
        }
        Ok(())
    }
    
    /// Handle toolbar button click
    pub async fn handle_toolbar_button_click(&self, item_id: &str, button_tag: i32) -> Result<(), UiError> {
        log::debug!("Toolbar button clicked: '{}' with tag {}", item_id, button_tag);
        
        let event = ToolbarEvent::ButtonClicked {
            item_id: item_id.to_string(),
            button_tag,
        };
        
        self.send_toolbar_event(event).await
    }
    
    /// Handle segmented control change
    pub async fn handle_segmented_control_change(&self, item_id: &str, selected_segment: i32) -> Result<(), UiError> {
        log::debug!("Segmented control changed: '{}' selected segment {}", item_id, selected_segment);
        
        let event = ToolbarEvent::SegmentChanged {
            item_id: item_id.to_string(),
            selected_segment,
        };
        
        self.send_toolbar_event(event).await
    }

    /// Find text control by element ID
    async fn find_text_control(&self, element_id: &str) -> Result<Either<Retained<NSTextView>, Retained<NSTextField>>, UiError> {
        {
            let text_views = self.text_views.read().await;
            if let Some(text_view) = text_views.get(element_id) {
                return Ok(Either::Left(text_view.clone()));
            }
        }
        
        {
            let text_fields = self.text_fields.read().await;
            if let Some(text_field) = text_fields.get(element_id) {
                return Ok(Either::Right(text_field.clone()));
            }
        }
        
        Err(UiError::platform_error(&format!("Text control not found: {}", element_id)))
    }

    /// Convert line/column position to character index in NSTextView
    fn line_column_to_char_index(&self, text_view: &NSTextView, position: CursorPosition) -> Result<usize, UiError> {
        unsafe {
            let text_storage = text_view.textStorage();
            if let Some(storage) = text_storage {
                let text = storage.string().to_string();
                let mut char_index = 0;
                let mut current_line = 0;
                
                for (i, ch) in text.char_indices() {
                    if current_line == position.line {
                        if char_index == position.column as usize {
                            return Ok(i);
                        }
                        char_index += 1;
                    }
                    if ch == '\n' {
                        current_line += 1;
                        char_index = 0;
                    }
                }
                
                Ok(text.len())
            } else {
                Err(UiError::platform_error("Text storage not available"))
            }
        }
    }

    /// Convert character index to line/column position
    fn char_index_to_line_column(&self, text_view: &NSTextView, char_index: usize) -> Result<CursorPosition, UiError> {
        unsafe {
            let text_storage = text_view.textStorage();
            if let Some(storage) = text_storage {
                let text = storage.string().to_string();
                let mut line = 0;
                let mut column = 0;
                
                for (i, ch) in text.char_indices() {
                    if i >= char_index {
                        break;
                    }
                    if ch == '\n' {
                        line += 1;
                        column = 0;
                    } else {
                        column += 1;
                    }
                }
                
                Ok(CursorPosition { line, column })
            } else {
                Err(UiError::platform_error("Text storage not available"))
            }
        }
    }
}

/// Helper enum for text control types
#[cfg(target_os = "macos")]
enum Either<L, R> {
    Left(L),
    Right(R),
}

#[cfg(not(target_os = "macos"))]
impl NativePlatform {
    pub fn new() -> Self {
        Self {}
    }
}

#[cfg(target_os = "macos")]
#[async_trait::async_trait]
impl AsyncPlatformOperations for NativePlatform {
    async fn configure_text_area(
        &self,
        element_id: &str,
        config: &TextAreaConfig,
    ) -> Result<(), UiError> {
        log::debug!("Configuring macOS text area '{}' with objc2 Cocoa APIs", element_id);
        
        let mtm = MainThreadMarker::new().ok_or_else(|| 
            UiError::platform_error("Not running on main thread"))?;
        
        // Create appropriate text control based on configuration
        if config.multiline {
            let text_view = NSTextView::new(mtm);
            
            unsafe {
                text_view.setEditable(!config.readonly);
                text_view.setRichText(false); // Plain text only
                text_view.setImportsGraphics(false);
                
                if let Some(initial_text) = &config.initial_text {
                    let ns_text = NSString::from_str(initial_text);
                    text_view.setString(&ns_text);
                }
                
                // Get main window and add to content view
                let app = NSApplication::sharedApplication(mtm);
                if let Some(main_window) = app.mainWindow() {
                    if let Some(content_view) = main_window.contentView() {
                        content_view.addSubview(&text_view);
                    }
                }
            }
            
            let mut text_views = self.text_views.write().await;
            text_views.insert(element_id.to_string(), text_view);
        } else {
            let text_field = NSTextField::new(mtm);
            
            unsafe {
                text_field.setEditable(!config.readonly);
                text_field.setBordered(true);
                text_field.setBezeled(true);
                
                if let Some(initial_text) = &config.initial_text {
                    let ns_text = NSString::from_str(initial_text);
                    text_field.setStringValue(&ns_text);
                }
                
                if let Some(placeholder) = &config.placeholder {
                    let placeholder_ns = NSString::from_str(placeholder);
                    text_field.setPlaceholderString(Some(&placeholder_ns));
                }
                
                // Get main window and add to content view
                let app = NSApplication::sharedApplication(mtm);
                if let Some(main_window) = app.mainWindow() {
                    if let Some(content_view) = main_window.contentView() {
                        content_view.addSubview(&text_field);
                    }
                }
            }
            
            let mut text_fields = self.text_fields.write().await;
            text_fields.insert(element_id.to_string(), text_field);
        }
        
        log::debug!("macOS text area configuration completed");
        Ok(())
    }
    
    async fn focus_element(
        &self,
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> Result<(), UiError> {
        log::debug!("Focusing macOS element '{}' at position {:?}", element_id, cursor_position);
        
        let mtm = MainThreadMarker::new().ok_or_else(|| 
            UiError::platform_error("Not running on main thread"))?;
        
        unsafe {
            let app = NSApplication::sharedApplication(mtm);
            let main_window = app.mainWindow().ok_or_else(|| 
                UiError::platform_error("Main window not available"))?;
            
            match self.find_text_control(element_id).await? {
                Either::Left(text_view) => {
                    let success = main_window.makeFirstResponder(Some(&*text_view));
                    if !success {
                        return Err(UiError::platform_error("Failed to make text view first responder"));
                    }
                    
                    // Set cursor position
                    let char_pos = self.line_column_to_char_index(&text_view, cursor_position)?;
                    let range = NSRange::new(char_pos, 0);
                    text_view.setSelectedRange(range);
                    text_view.scrollRangeToVisible(range);
                }
                Either::Right(text_field) => {
                    let success = main_window.makeFirstResponder(Some(&*text_field));
                    if !success {
                        return Err(UiError::platform_error("Failed to make text field first responder"));
                    }
                    
                    // Set cursor position for text field (single line)
                    let range = NSRange::new(cursor_position.column as usize, 0);
                    if let Some(field_editor) = text_field.currentEditor() {
                        field_editor.setSelectedRange(range);
                    }
                }
            }
        }
        
        log::debug!("macOS element focus completed");
        Ok(())
    }
    
    async fn setup_drag_drop(&self, _updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>) -> Result<(), UiError> {
        log::debug!("Setting up macOS drag-and-drop with objc2 APIs");
        
        let mtm = MainThreadMarker::new().ok_or_else(|| 
            UiError::platform_error("Not running on main thread"))?;
        
        unsafe {
            let drag_types = NSArray::from_slice(&[
                NSString::from_str("public.plain-text"),
                NSString::from_str("public.file-url"),
                NSString::from_str("public.utf8-plain-text"),
            ]);
            
            // Register drag types for text views with enhanced drag operations
            let text_views = self.text_views.read().await;
            for text_view in text_views.values() {
                text_view.registerForDraggedTypes(&drag_types);
                
                // Enable drag operations
                text_view.setAutomaticDragDetectionEnabled(true);
                text_view.setAllowsDocumentBackgroundColorChange(false);
            }
            
            // Register drag types for text fields
            let text_fields = self.text_fields.read().await;
            for text_field in text_fields.values() {
                let text_drag_types = NSArray::from_slice(&[
                    NSString::from_str("public.plain-text"),
                    NSString::from_str("public.utf8-plain-text"),
                ]);
                text_field.registerForDraggedTypes(&text_drag_types);
            }
            
            // Setup main window for global drag-drop handling
            let app = NSApplication::sharedApplication(mtm);
            if let Some(main_window) = app.mainWindow() {
                if let Some(content_view) = main_window.contentView() {
                    content_view.registerForDraggedTypes(&drag_types);
                }
            }
        }
        
        log::debug!("macOS drag-and-drop setup completed with enhanced operations");
        Ok(())
    }
    
    async fn get_text_content(&self, element_id: &str) -> Result<String, UiError> {
        log::debug!("Getting text content from macOS element '{}'", element_id);
        
        unsafe {
            match self.find_text_control(element_id).await? {
                Either::Left(text_view) => {
                    let ns_string = text_view.string();
                    let text = ns_string.to_string();
                    log::debug!("Retrieved {} characters from text view '{}'", text.len(), element_id);
                    Ok(text)
                }
                Either::Right(text_field) => {
                    let ns_string = text_field.stringValue();
                    let text = ns_string.to_string();
                    log::debug!("Retrieved {} characters from text field '{}'", text.len(), element_id);
                    Ok(text)
                }
            }
        }
    }
    
    async fn set_text_content(&self, element_id: &str, text: &str) -> Result<(), UiError> {
        log::debug!("Setting text content in macOS element '{}': {} chars", element_id, text.len());
        
        let ns_text = NSString::from_str(text);
        
        unsafe {
            match self.find_text_control(element_id).await? {
                Either::Left(text_view) => {
                    text_view.setString(&ns_text);
                }
                Either::Right(text_field) => {
                    text_field.setStringValue(&ns_text);
                }
            }
        }
        
        log::debug!("Text content set successfully");
        Ok(())
    }
    
    async fn get_cursor_position(&self, element_id: &str) -> Result<CursorPosition, UiError> {
        log::debug!("Getting cursor position from macOS element '{}'", element_id);
        
        unsafe {
            match self.find_text_control(element_id).await? {
                Either::Left(text_view) => {
                    let selected_range = text_view.selectedRange();
                    let char_index = selected_range.location;
                    let position = self.char_index_to_line_column(&text_view, char_index)?;
                    log::debug!("Cursor position: {:?}", position);
                    Ok(position)
                }
                Either::Right(text_field) => {
                    if let Some(field_editor) = text_field.currentEditor() {
                        let selected_range = field_editor.selectedRange();
                        let position = CursorPosition {
                            line: 0, // Text fields are single-line
                            column: selected_range.location as i32,
                        };
                        log::debug!("Cursor position: {:?}", position);
                        Ok(position)
                    } else {
                        Ok(CursorPosition { line: 0, column: 0 })
                    }
                }
            }
        }
    }
    
    async fn set_cursor_position(&self, element_id: &str, position: CursorPosition) -> Result<(), UiError> {
        log::debug!("Setting cursor position in macOS element '{}' to {:?}", element_id, position);
        
        unsafe {
            match self.find_text_control(element_id).await? {
                Either::Left(text_view) => {
                    let char_pos = self.line_column_to_char_index(&text_view, position)?;
                    let range = NSRange::new(char_pos, 0);
                    text_view.setSelectedRange(range);
                    text_view.scrollRangeToVisible(range);
                }
                Either::Right(text_field) => {
                    let range = NSRange::new(position.column as usize, 0);
                    if let Some(field_editor) = text_field.currentEditor() {
                        field_editor.setSelectedRange(range);
                    }
                }
            }
        }
        
        log::debug!("Cursor position set successfully");
        Ok(())
    }
    
    async fn get_selection(&self, element_id: &str) -> Result<Option<TextRange>, UiError> {
        log::debug!("Getting selection from macOS element '{}'", element_id);
        
        unsafe {
            match self.find_text_control(element_id).await? {
                Either::Left(text_view) => {
                    let selected_range = text_view.selectedRange();
                    if selected_range.length > 0 {
                        let start_pos = self.char_index_to_line_column(&text_view, selected_range.location)?;
                        let end_pos = self.char_index_to_line_column(&text_view, selected_range.location + selected_range.length)?;
                        let range = TextRange { start: start_pos, end: end_pos };
                        log::debug!("Selection found: {:?}", range);
                        Ok(Some(range))
                    } else {
                        log::debug!("No selection found");
                        Ok(None)
                    }
                }
                Either::Right(text_field) => {
                    if let Some(field_editor) = text_field.currentEditor() {
                        let selected_range = field_editor.selectedRange();
                        if selected_range.length > 0 {
                            let start_pos = CursorPosition { line: 0, column: selected_range.location as i32 };
                            let end_pos = CursorPosition { line: 0, column: (selected_range.location + selected_range.length) as i32 };
                            let range = TextRange { start: start_pos, end: end_pos };
                            log::debug!("Selection found: {:?}", range);
                            Ok(Some(range))
                        } else {
                            log::debug!("No selection found");
                            Ok(None)
                        }
                    } else {
                        Ok(None)
                    }
                }
            }
        }
    }
    
    async fn set_selection(&self, element_id: &str, range: TextRange) -> Result<(), UiError> {
        log::debug!("Setting selection in macOS element '{}' to {:?}", element_id, range);
        
        unsafe {
            match self.find_text_control(element_id).await? {
                Either::Left(text_view) => {
                    let start_char = self.line_column_to_char_index(&text_view, range.start)?;
                    let end_char = self.line_column_to_char_index(&text_view, range.end)?;
                    let ns_range = NSRange::new(start_char, end_char - start_char);
                    text_view.setSelectedRange(ns_range);
                    text_view.scrollRangeToVisible(ns_range);
                }
                Either::Right(text_field) => {
                    let ns_range = NSRange::new(range.start.column as usize, (range.end.column - range.start.column) as usize);
                    if let Some(field_editor) = text_field.currentEditor() {
                        field_editor.setSelectedRange(ns_range);
                    }
                }
            }
        }
        
        log::debug!("Selection set successfully");
        Ok(())
    }
    
    fn spawn_platform_operation<F, T>(&self, operation: F) -> Task<T>
    where
        F: Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        let (runnable, task) = async_task::spawn(operation, |runnable| {
            // Execute the runnable immediately for platform operations
            runnable.run();
        });
        runnable.run();
        task
    }
    
    async fn execute_platform_operation<F, T>(&self, operation: F) -> Result<T, UiError>
    where
        F: Future<Output = Result<T, UiError>> + Send + 'static,
        T: Send + 'static,
    {
        let task = self.spawn_platform_operation(operation);
        task.await
    }
}

// Fallback implementation for non-macOS platforms
#[cfg(not(target_os = "macos"))]
#[async_trait::async_trait]
impl AsyncPlatformOperations for NativePlatform {
    async fn configure_text_area(
        &self,
        element_id: &str,
        _config: &TextAreaConfig,
    ) -> Result<(), UiError> {
        log::debug!("macOS platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Cocoa operations only available on macOS"))
    }
    
    async fn focus_element(
        &self,
        element_id: &str,
        _cursor_position: CursorPosition,
    ) -> Result<(), UiError> {
        log::debug!("macOS platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Cocoa operations only available on macOS"))
    }
    
    async fn setup_drag_drop(&self, _updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>) -> Result<(), UiError> {
        log::debug!("macOS platform operations not available on this platform");
        Err(UiError::platform_error("Cocoa operations only available on macOS"))
    }
    
    async fn get_text_content(&self, element_id: &str) -> Result<String, UiError> {
        log::debug!("macOS platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Cocoa operations only available on macOS"))
    }
    
    async fn set_text_content(&self, element_id: &str, _text: &str) -> Result<(), UiError> {
        log::debug!("macOS platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Cocoa operations only available on macOS"))
    }
    
    async fn get_cursor_position(&self, element_id: &str) -> Result<CursorPosition, UiError> {
        log::debug!("macOS platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Cocoa operations only available on macOS"))
    }
    
    async fn set_cursor_position(&self, element_id: &str, _position: CursorPosition) -> Result<(), UiError> {
        log::debug!("macOS platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Cocoa operations only available on macOS"))
    }
    
    async fn get_selection(&self, element_id: &str) -> Result<Option<TextRange>, UiError> {
        log::debug!("macOS platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Cocoa operations only available on macOS"))
    }
    
    async fn set_selection(&self, element_id: &str, _range: TextRange) -> Result<(), UiError> {
        log::debug!("macOS platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Cocoa operations only available on macOS"))
    }
}

#[cfg(target_os = "macos")]
impl NativePlatform {
    /// Find text control by element ID, returning either NSTextView or NSTextField
    async fn find_text_control(&self, element_id: &str) -> Result<Either<Retained<NSTextView>, Retained<NSTextField>>, UiError> {
        // First check text views
        let text_views = self.text_views.read().await;
        if let Some(text_view) = text_views.get(element_id) {
            return Ok(Either::Left(text_view.clone()));
        }
        drop(text_views);
        
        // Then check text fields
        let text_fields = self.text_fields.read().await;
        if let Some(text_field) = text_fields.get(element_id) {
            return Ok(Either::Right(text_field.clone()));
        }
        
        Err(UiError::platform_error(&format!("Text control '{}' not found", element_id)))
    }
    
    /// Convert line/column position to character index for NSTextView
    fn line_column_to_char_index(&self, text_view: &NSTextView, position: CursorPosition) -> Result<usize, UiError> {
        unsafe {
            let text_storage = text_view.textStorage().ok_or_else(|| 
                UiError::platform_error("Text view has no text storage"))?;
            let text = text_storage.string();
            let text_str = text.to_string();
            
            let mut char_index = 0;
            let mut current_line = 0;
            
            for (i, ch) in text_str.char_indices() {
                if current_line == position.line {
                    if char_index == position.column as usize {
                        return Ok(i);
                    }
                    char_index += 1;
                }
                
                if ch == '\n' {
                    current_line += 1;
                    char_index = 0;
                }
            }
            
            // If we reach here, position is at end of line or beyond
            Ok(text_str.len())
        }
    }
    
    /// Convert character index to line/column position for NSTextView
    fn char_index_to_line_column(&self, text_view: &NSTextView, char_index: usize) -> Result<CursorPosition, UiError> {
        unsafe {
            let text_storage = text_view.textStorage().ok_or_else(|| 
                UiError::platform_error("Text view has no text storage"))?;
            let text = text_storage.string();
            let text_str = text.to_string();
            
            let mut line = 0;
            let mut column = 0;
            
            for (i, ch) in text_str.char_indices() {
                if i >= char_index {
                    break;
                }
                
                if ch == '\n' {
                    line += 1;
                    column = 0;
                } else {
                    column += 1;
                }
            }
            
            Ok(CursorPosition { line, column: column as i32 })
        }
    }
    
    /// Handle drag operation for text controls
    async fn handle_drag_operation(&self, pasteboard: &NSPasteboard, element_id: &str) -> Result<bool, UiError> {
        unsafe {
            // Check for text content
            let string_type = NSString::from_str("public.plain-text");
            if let Some(dragged_text) = pasteboard.stringForType(&string_type) {
                let text_to_insert = dragged_text.to_string();
                log::debug!("Drag operation received text: {} chars for element '{}'", 
                    text_to_insert.len(), element_id);
                
                // Insert text at cursor position
                match self.find_text_control(element_id).await? {
                    Either::Left(text_view) => {
                        // Get current text and cursor position
                        let current_text = text_view.string().to_string();
                        let selected_range = text_view.selectedRange();
                        let cursor_pos = selected_range.location;
                        
                        // Split text at cursor and insert
                        let char_indices: Vec<_> = current_text.char_indices().collect();
                        let (before, after) = if cursor_pos < char_indices.len() {
                            let byte_pos = char_indices[cursor_pos].0;
                            current_text.split_at(byte_pos)
                        } else {
                            (current_text.as_str(), "")
                        };
                        
                        // Build and set new text
                        let new_text = format!("{}{}{}", before, text_to_insert, after);
                        let ns_new_text = NSString::from_str(&new_text);
                        text_view.setString(&ns_new_text);
                        
                        // Move cursor after inserted text
                        let new_cursor_pos = cursor_pos + text_to_insert.chars().count();
                        let new_range = NSRange::new(new_cursor_pos, 0);
                        text_view.setSelectedRange(new_range);
                        text_view.scrollRangeToVisible(new_range);
                        
                        log::info!("Inserted {} chars into text view '{}'", text_to_insert.len(), element_id);
                    }
                    Either::Right(text_field) => {
                        // Similar logic for NSTextField
                        let current_text = text_field.stringValue().to_string();
                        let cursor_pos = if let Some(field_editor) = text_field.currentEditor() {
                            field_editor.selectedRange().location
                        } else {
                            current_text.chars().count()
                        };
                        
                        let char_indices: Vec<_> = current_text.char_indices().collect();
                        let (before, after) = if cursor_pos < char_indices.len() {
                            let byte_pos = char_indices[cursor_pos].0;
                            current_text.split_at(byte_pos)
                        } else {
                            (current_text.as_str(), "")
                        };
                        
                        let new_text = format!("{}{}{}", before, text_to_insert, after);
                        let ns_new_text = NSString::from_str(&new_text);
                        text_field.setStringValue(&ns_new_text);
                        
                        if let Some(field_editor) = text_field.currentEditor() {
                            let new_cursor_pos = cursor_pos + text_to_insert.chars().count();
                            let new_range = NSRange::new(new_cursor_pos, 0);
                            field_editor.setSelectedRange(new_range);
                        }
                        
                        log::info!("Inserted {} chars into text field '{}'", text_to_insert.len(), element_id);
                    }
                }
                
                return Ok(true);
            }
            
            // Check for file URLs (keep existing logic)
            let url_type = NSString::from_str("public.file-url");
            if pasteboard.types().containsObject(&url_type) {
                log::debug!("Drag operation received file URL for element '{}'", element_id);
                return Ok(true);
            }
            
            log::debug!("Drag operation with unsupported type for element '{}'", element_id);
            Ok(false)
        }
    }
    
    /// Get toolbar item by identifier
    pub async fn get_toolbar_item(&self, item_id: &str) -> Result<Option<Retained<NSToolbarItem>>, UiError> {
        let toolbar_items = self.toolbar_items.read().await;
        Ok(toolbar_items.get(item_id).cloned())
    }
    
    /// Enable/disable toolbar item
    pub async fn set_toolbar_item_enabled(&self, item_id: &str, enabled: bool) -> Result<(), UiError> {
        if let Some(item) = self.get_toolbar_item(item_id).await? {
            unsafe {
                if let Some(view) = item.view() {
                    // Try to cast to NSButton first
                    if view.isKindOfClass(&NSButton::class()) {
                        let button = view.downcast::<NSButton>().ok_or_else(|| 
                            UiError::platform_error("Failed to downcast to NSButton"))?;
                        button.setEnabled(enabled);
                    }
                    // Try NSSegmentedControl
                    else if view.isKindOfClass(&NSSegmentedControl::class()) {
                        let segmented_control = view.downcast::<NSSegmentedControl>().ok_or_else(|| 
                            UiError::platform_error("Failed to downcast to NSSegmentedControl"))?;
                        segmented_control.setEnabled(enabled);
                    }
                }
            }
            
            log::debug!("Toolbar item '{}' enabled state set to {}", item_id, enabled);
        }
        
        Ok(())
    }
}

impl Default for NativePlatform {
    fn default() -> Self {
        Self::new()
    }
}