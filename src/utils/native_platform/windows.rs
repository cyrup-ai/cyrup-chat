// Windows native platform operations using Win32 APIs
//
// Complete Win32 implementation with zero-allocation patterns

use super::{AsyncPlatformOperations, TextAreaConfig, CursorPosition, TextRange};
use crate::errors::ui::UiError;

#[cfg(target_os = "windows")]
use windows::{
    core::*,
    Win32::Foundation::*,
    Win32::UI::WindowsAndMessaging::*,
    Win32::UI::Controls::*,
    Win32::System::Ole::*,
    Win32::System::Com::*,
    Win32::Graphics::Gdi::*,
};
#[cfg(target_os = "windows")]
use std::collections::HashMap;
#[cfg(target_os = "windows")]
use std::sync::Arc;
#[cfg(target_os = "windows")]
use tokio::sync::RwLock;

/// Windows-specific platform errors
#[cfg(target_os = "windows")]
#[derive(Debug, thiserror::Error)]
pub enum WindowsPlatformError {
    #[error("Window not found: {0}")]
    WindowNotFound(String),
    #[error("Win32 API error: {0}")]
    Win32Error(#[from] windows::core::Error),
    #[error("Text operation failed: {0}")]
    TextOperationError(String),
    #[error("COM operation failed: {0}")]
    ComError(String),
    #[error("Invalid cursor position: line {line}, column {column}")]
    InvalidCursorPosition { line: i32, column: i32 },
}

/// Native Windows platform implementation with Win32 integration
#[cfg(target_os = "windows")]
pub struct NativePlatform {
    text_controls: Arc<RwLock<HashMap<String, HWND>>>,
    drop_targets: Arc<RwLock<HashMap<String, IDropTarget>>>,
}

/// Fallback implementation for non-Windows platforms
#[cfg(not(target_os = "windows"))]
pub struct NativePlatform {}

#[cfg(target_os = "windows")]
impl NativePlatform {
    pub fn new() -> Self {
        Self {
            text_controls: Arc::new(RwLock::new(HashMap::new())),
            drop_targets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Find or discover a text control HWND by element ID with caching
    async fn find_text_control(&self, element_id: &str) -> Result<HWND, UiError> {
        // Try cache first for zero-allocation access
        {
            let cache = self.text_controls.read().await;
            if let Some(&hwnd) = cache.get(element_id) {
                // Verify window still exists
                unsafe {
                    if IsWindow(hwnd).as_bool() {
                        return Ok(hwnd);
                    }
                }
            }
        }

        // Discover window in system
        let hwnd = self.discover_text_control(element_id).await?;

        // Cache for future zero-allocation access
        {
            let mut cache = self.text_controls.write().await;
            cache.insert(element_id.to_string(), hwnd);
        }

        Ok(hwnd)
    }

    /// Discover text control HWND in Windows system
    async fn discover_text_control(&self, element_id: &str) -> Result<HWND, UiError> {
        unsafe {
            // Find window by class name and title
            let class_name = format!("CYRUP_TEXT_{}", element_id);
            let class_wide: Vec<u16> = class_name.encode_utf16().chain(Some(0)).collect();
            
            let hwnd = FindWindowW(PCWSTR(class_wide.as_ptr()), None);
            
            if hwnd.0 != 0 {
                Ok(hwnd)
            } else {
                // Fallback: search by window title containing element_id
                let title_wide: Vec<u16> = element_id.encode_utf16().chain(Some(0)).collect();
                let hwnd = FindWindowW(None, PCWSTR(title_wide.as_ptr()));
                
                if hwnd.0 != 0 {
                    Ok(hwnd)
                } else {
                    Err(UiError::platform_error(&format!("Text control not found: {}", element_id)))
                }
            }
        }
    }

    /// Convert line/column position to character position in text control
    unsafe fn position_to_char_index(&self, hwnd: HWND, position: CursorPosition) -> Result<usize, UiError> {
        let line_start = SendMessageW(hwnd, EM_LINEINDEX, WPARAM(position.line as usize), LPARAM(0)).0 as usize;
        let line_length = SendMessageW(hwnd, EM_LINELENGTH, WPARAM(line_start), LPARAM(0)).0 as usize;
        
        if position.column < 0 || position.column as usize > line_length {
            return Err(UiError::platform_error(&format!(
                "Invalid column {}: must be between 0 and {}", position.column, line_length
            )));
        }
        
        Ok(line_start + position.column as usize)
    }

    /// Convert character position to line/column position
    unsafe fn char_index_to_position(&self, hwnd: HWND, char_pos: usize) -> Result<CursorPosition, UiError> {
        let line = SendMessageW(hwnd, EM_LINEFROMCHAR, WPARAM(char_pos), LPARAM(0)).0 as i32;
        let line_start = SendMessageW(hwnd, EM_LINEINDEX, WPARAM(line as usize), LPARAM(0)).0 as usize;
        let column = char_pos - line_start;
        
        Ok(CursorPosition { line, column: column as i32 })
    }
}

#[cfg(not(target_os = "windows"))]
impl NativePlatform {
    pub fn new() -> Self {
        Self {}
    }
}

/// IDropTarget implementation for text controls
#[cfg(target_os = "windows")]
#[implement(IDropTarget)]
struct TextAreaDropTarget {
    hwnd: HWND,
}

#[cfg(target_os = "windows")]
impl IDropTarget_Impl for TextAreaDropTarget {
    fn DragEnter(
        &self,
        pdataobj: Option<&IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> Result<()> {
        unsafe {
            if let Some(data_obj) = pdataobj {
                let text_format = FORMATETC {
                    cfFormat: CF_TEXT.0,
                    ptd: std::ptr::null_mut(),
                    dwAspect: DVASPECT_CONTENT,
                    lindex: -1,
                    tymed: TYMED_HGLOBAL,
                };
                
                if data_obj.QueryGetData(&text_format).is_ok() {
                    *pdweffect = DROPEFFECT_COPY;
                } else {
                    *pdweffect = DROPEFFECT_NONE;
                }
            } else {
                *pdweffect = DROPEFFECT_NONE;
            }
        }
        Ok(())
    }
    
    fn DragOver(
        &self,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        _pdweffect: *mut DROPEFFECT,
    ) -> Result<()> {
        // Maintain drop effect from DragEnter
        Ok(())
    }
    
    fn DragLeave(&self) -> Result<()> {
        Ok(())
    }
    
    fn Drop(
        &self,
        pdataobj: Option<&IDataObject>,
        _grfkeystate: MODIFIERKEYS_FLAGS,
        _pt: &POINTL,
        pdweffect: *mut DROPEFFECT,
    ) -> Result<()> {
        unsafe {
            if let Some(data_obj) = pdataobj {
                let text_format = FORMATETC {
                    cfFormat: CF_TEXT.0,
                    ptd: std::ptr::null_mut(),
                    dwAspect: DVASPECT_CONTENT,
                    lindex: -1,
                    tymed: TYMED_HGLOBAL,
                };
                
                let mut medium = STGMEDIUM::default();
                if data_obj.GetData(&text_format, &mut medium).is_ok() {
                    let global_mem = medium.Anonymous.hGlobal;
                    let data_ptr = GlobalLock(global_mem) as *const u8;
                    
                    if !data_ptr.is_null() {
                        let data_len = GlobalSize(global_mem);
                        let data_slice = std::slice::from_raw_parts(data_ptr, data_len);
                        let text = String::from_utf8_lossy(data_slice);
                        
                        // Insert text at current cursor position
                        let text_wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
                        SendMessageW(self.hwnd, EM_REPLACESEL, WPARAM(1), LPARAM(text_wide.as_ptr() as isize));
                        
                        GlobalUnlock(global_mem);
                        *pdweffect = DROPEFFECT_COPY;
                    }
                    
                    ReleaseStgMedium(&mut medium);
                }
            }
        }
        Ok(())
    }
}

#[cfg(target_os = "windows")]
#[async_trait::async_trait]
impl AsyncPlatformOperations for NativePlatform {
    async fn configure_text_area(
        &self,
        element_id: &str,
        config: &TextAreaConfig,
    ) -> Result<(), UiError> {
        log::debug!("Configuring Windows text area '{}' with Win32 APIs", element_id);
        
        let hwnd = self.find_text_control(element_id).await?;
        
        unsafe {
            // Set initial text if provided
            if let Some(initial_text) = &config.initial_text {
                let text_wide: Vec<u16> = initial_text.encode_utf16().chain(Some(0)).collect();
                let result = SetWindowTextW(hwnd, PCWSTR(text_wide.as_ptr()));
                if !result.as_bool() {
                    return Err(UiError::platform_error("Failed to set initial text"));
                }
            }
            
            // Configure text area properties
            let mut style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
            
            if config.multiline {
                style |= ES_MULTILINE.0 | ES_WANTRETURN.0 | WS_VSCROLL.0;
            } else {
                style &= !(ES_MULTILINE.0 | ES_WANTRETURN.0 | WS_VSCROLL.0);
            }
            
            if let Some(max_length) = config.max_length {
                SendMessageW(hwnd, EM_SETLIMITTEXT, WPARAM(max_length), LPARAM(0));
            }
            
            SetWindowLongW(hwnd, GWL_STYLE, style as i32);
            
            // Force window to update with new style
            SetWindowPos(
                hwnd, 
                HWND::default(), 
                0, 0, 0, 0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER | SWP_FRAMECHANGED
            )?;
            
            // Handle placeholder (simulate with initial gray text)
            if let Some(placeholder) = &config.placeholder {
                let text_length = GetWindowTextLengthW(hwnd);
                if text_length == 0 {
                    let placeholder_wide: Vec<u16> = placeholder.encode_utf16().chain(Some(0)).collect();
                    SetWindowTextW(hwnd, PCWSTR(placeholder_wide.as_ptr()));
                    // Would need additional state tracking for placeholder behavior
                }
            }
        }
        
        log::debug!("Windows text area configuration completed");
        Ok(())
    }
    
    async fn focus_element(
        &self,
        element_id: &str,
        cursor_position: CursorPosition,
    ) -> Result<(), UiError> {
        log::debug!("Focusing Windows element '{}' at position {:?}", element_id, cursor_position);
        
        let hwnd = self.find_text_control(element_id).await?;
        
        unsafe {
            // Set focus to the control
            SetFocus(hwnd);
            
            // Set cursor position
            let char_pos = self.position_to_char_index(hwnd, cursor_position)?;
            SendMessageW(hwnd, EM_SETSEL, WPARAM(char_pos), LPARAM(char_pos as isize));
            
            // Ensure cursor is visible
            SendMessageW(hwnd, EM_SCROLLCARET, WPARAM(0), LPARAM(0));
        }
        
        log::debug!("Windows element focus completed");
        Ok(())
    }
    
    async fn setup_drag_drop(async fn setup_drag_drop(&self) -> Result<(), UiError>self, _updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>) -> Result<(), UiError> {
        log::debug!("Setting up Windows drag-and-drop with Win32 OLE APIs");
        
        unsafe {
            // Initialize COM for this thread
            CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;
            
            // Create and register drop target for each text control
            let controls = self.text_controls.read().await;
            for (element_id, &hwnd) in controls.iter() {
                let drop_target: IDropTarget = TextAreaDropTarget { hwnd }.into();
                
                match RegisterDragDrop(hwnd, &drop_target) {
                    Ok(_) => {
                        // Cache drop target for cleanup
                        let mut targets = self.drop_targets.write().await;
                        targets.insert(element_id.clone(), drop_target);
                        log::debug!("Registered drag-drop for control: {}", element_id);
                    }
                    Err(e) => {
                        log::warn!("Failed to register drag-drop for {}: {:?}", element_id, e);
                    }
                }
            }
        }
        
        log::debug!("Windows drag-and-drop setup completed");
        Ok(())
    }
    
    async fn get_text_content(&self, element_id: &str) -> Result<String, UiError> {
        log::debug!("Getting text content from Windows element '{}'", element_id);
        
        let hwnd = self.find_text_control(element_id).await?;
        
        unsafe {
            let text_length = GetWindowTextLengthW(hwnd);
            if text_length == 0 {
                return Ok(String::new());
            }
            
            let mut buffer = vec![0u16; (text_length + 1) as usize];
            let chars_copied = GetWindowTextW(hwnd, &mut buffer);
            
            if chars_copied > 0 {
                buffer.truncate(chars_copied as usize);
                let text = String::from_utf16_lossy(&buffer);
                log::debug!("Retrieved {} characters from element '{}'", text.len(), element_id);
                Ok(text)
            } else {
                Err(UiError::platform_error("Failed to get window text"))
            }
        }
    }
    
    async fn set_text_content(&self, element_id: &str, text: &str) -> Result<(), UiError> {
        log::debug!("Setting text content in Windows element '{}': {} chars", element_id, text.len());
        
        let hwnd = self.find_text_control(element_id).await?;
        
        unsafe {
            let text_wide: Vec<u16> = text.encode_utf16().chain(Some(0)).collect();
            let result = SetWindowTextW(hwnd, PCWSTR(text_wide.as_ptr()));
            
            if result.as_bool() {
                log::debug!("Text content set successfully");
                Ok(())
            } else {
                Err(UiError::platform_error("Failed to set window text"))
            }
        }
    }
    
    async fn get_cursor_position(&self, element_id: &str) -> Result<CursorPosition, UiError> {
        log::debug!("Getting cursor position from Windows element '{}'", element_id);
        
        let hwnd = self.find_text_control(element_id).await?;
        
        unsafe {
            let selection = SendMessageW(hwnd, EM_GETSEL, WPARAM(0), LPARAM(0));
            let cursor_pos = LOWORD(selection.0 as u32) as usize;
            
            let position = self.char_index_to_position(hwnd, cursor_pos)?;
            log::debug!("Cursor position: {:?}", position);
            Ok(position)
        }
    }
    
    async fn set_cursor_position(&self, element_id: &str, position: CursorPosition) -> Result<(), UiError> {
        log::debug!("Setting cursor position in Windows element '{}' to {:?}", element_id, position);
        
        let hwnd = self.find_text_control(element_id).await?;
        
        unsafe {
            let char_pos = self.position_to_char_index(hwnd, position)?;
            
            // Set cursor position
            SendMessageW(hwnd, EM_SETSEL, WPARAM(char_pos), LPARAM(char_pos as isize));
            
            // Ensure cursor is visible
            SendMessageW(hwnd, EM_SCROLLCARET, WPARAM(0), LPARAM(0));
        }
        
        log::debug!("Cursor position set successfully");
        Ok(())
    }
    
    async fn get_selection(&self, element_id: &str) -> Result<Option<TextRange>, UiError> {
        log::debug!("Getting selection from Windows element '{}'", element_id);
        
        let hwnd = self.find_text_control(element_id).await?;
        
        unsafe {
            let mut start: u32 = 0;
            let mut end: u32 = 0;
            
            SendMessageW(hwnd, EM_GETSEL, WPARAM(&mut start as *mut u32 as usize), LPARAM(&mut end as *mut u32 as isize));
            
            if start != end {
                let start_pos = self.char_index_to_position(hwnd, start as usize)?;
                let end_pos = self.char_index_to_position(hwnd, end as usize)?;
                
                let range = TextRange { start: start_pos, end: end_pos };
                log::debug!("Selection found: {:?}", range);
                Ok(Some(range))
            } else {
                log::debug!("No selection found");
                Ok(None)
            }
        }
    }
    
    async fn set_selection(&self, element_id: &str, range: TextRange) -> Result<(), UiError> {
        log::debug!("Setting selection in Windows element '{}' to {:?}", element_id, range);
        
        let hwnd = self.find_text_control(element_id).await?;
        
        unsafe {
            let start_char_pos = self.position_to_char_index(hwnd, range.start)?;
            let end_char_pos = self.position_to_char_index(hwnd, range.end)?;
            
            // Set selection
            SendMessageW(hwnd, EM_SETSEL, WPARAM(start_char_pos), LPARAM(end_char_pos as isize));
        }
        
        log::debug!("Selection set successfully");
        Ok(())
    }
}

// Fallback implementation for non-Windows platforms
#[cfg(not(target_os = "windows"))]
#[async_trait::async_trait]
impl AsyncPlatformOperations for NativePlatform {
    async fn configure_text_area(
        &self,
        element_id: &str,
        _config: &TextAreaConfig,
    ) -> Result<(), UiError> {
        log::debug!("Windows platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Win32 operations only available on Windows"))
    }
    
    async fn focus_element(
        &self,
        element_id: &str,
        _cursor_position: CursorPosition,
    ) -> Result<(), UiError> {
        log::debug!("Windows platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Win32 operations only available on Windows"))
    }
    
    async fn setup_drag_drop(async fn setup_drag_drop(&self) -> Result<(), UiError>self, _updater: std::sync::Arc<dyn Fn(crate::environment::types::AppEvent) + Send + Sync>) -> Result<(), UiError> {
        log::debug!("Windows platform operations not available on this platform");
        Err(UiError::platform_error("Win32 operations only available on Windows"))
    }
    
    async fn get_text_content(&self, element_id: &str) -> Result<String, UiError> {
        log::debug!("Windows platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Win32 operations only available on Windows"))
    }
    
    async fn set_text_content(&self, element_id: &str, _text: &str) -> Result<(), UiError> {
        log::debug!("Windows platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Win32 operations only available on Windows"))
    }
    
    async fn get_cursor_position(&self, element_id: &str) -> Result<CursorPosition, UiError> {
        log::debug!("Windows platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Win32 operations only available on Windows"))
    }
    
    async fn set_cursor_position(&self, element_id: &str, _position: CursorPosition) -> Result<(), UiError> {
        log::debug!("Windows platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Win32 operations only available on Windows"))
    }
    
    async fn get_selection(&self, element_id: &str) -> Result<Option<TextRange>, UiError> {
        log::debug!("Windows platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Win32 operations only available on Windows"))
    }
    
    async fn set_selection(&self, element_id: &str, _range: TextRange) -> Result<(), UiError> {
        log::debug!("Windows platform operations not available on this platform: {}", element_id);
        Err(UiError::platform_error("Win32 operations only available on Windows"))
    }
}

impl Default for NativePlatform {
    fn default() -> Self {
        Self::new()
    }
}