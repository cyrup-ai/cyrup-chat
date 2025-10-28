use std::path::PathBuf;
use dioxus::prelude::spawn;

use crate::view_model::{self, AttachmentMedia};

pub fn show_emoji_popup() {
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        
        // Trigger browser's native emoji picker if available
        let js_code = r#"
            // Try to focus an input element to trigger mobile emoji keyboard
            const input = document.createElement('input');
            input.type = 'text';
            input.style.position = 'fixed';
            input.style.left = '-9999px';
            input.style.opacity = '0';
            document.body.appendChild(input);
            input.focus();
            
            // Clean up after a short delay
            setTimeout(() => {
                document.body.removeChild(input);
            }, 100);
        "#;
        
        match js_sys::eval(js_code) {
            Ok(_) => log::debug!("Emoji picker trigger attempted"),
            Err(e) => log::error!("Failed to trigger emoji picker: {:?}", e),
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        log::warn!("Emoji popup only available in WASM builds");
    }
}

pub fn open_file_dialog(directory: &str) -> Option<view_model::AttachmentMedia> {
    Some(crate::view_model::AttachmentMedia {
        preview: None,
        path: std::path::PathBuf::new(),
        filename: "Mockfile.mp4".to_string(),
        description: None,
        is_uploaded: false,
        server_id: None,
    })
}

pub fn read_file_to_attachment(path: PathBuf) -> Option<view_model::AttachmentMedia> {
    None
}

pub fn open_file(path: impl AsRef<std::path::Path>) {
    let path = path.as_ref();
    
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        
        // In WASM, we can't open arbitrary file paths
        // Instead, we can trigger a download or open a URL
        let path_str = path.to_string_lossy();
        
        if path_str.starts_with("http") {
            // Open URL in new tab
            let js_code = format!("window.open('{}', '_blank');", path_str);
            match js_sys::eval(&js_code) {
                Ok(_) => log::debug!("Opened URL: {}", path_str),
                Err(e) => log::error!("Failed to open URL: {:?}", e),
            }
        } else {
            log::warn!("Cannot open local file path in WASM: {}", path_str);
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        log::warn!("File opening only available in WASM builds for URLs: {:?}", path);
    }
}

pub fn execute_js(js: &str) {
    // Implement JavaScript execution for Dioxus 0.7 WASM using web-sys
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        
        // Execute JavaScript directly in the browser
        match js_sys::eval(js) {
            Ok(_) => {
                log::debug!("JavaScript executed successfully: {}", js);
            }
            Err(e) => {
                log::error!("JavaScript execution failed: {:?}", e);
            }
        }
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        log::warn!("JavaScript execution only available in WASM builds: {}", js);
    }
}


pub fn copy_to_clipboard(content: impl AsRef<str>) {
    let content = content.as_ref();
    
    #[cfg(target_arch = "wasm32")]
    {
        use wasm_bindgen::prelude::*;
        use wasm_bindgen_futures::JsFuture;
        
        let content = content.to_string();
        spawn(async move {
            // Use modern Clipboard API if available
            let js_code = format!(
                r#"
                if (navigator.clipboard && navigator.clipboard.writeText) {{
                    navigator.clipboard.writeText('{}').then(() => {{
                        console.log('Text copied to clipboard successfully');
                    }}).catch(err => {{
                        console.error('Failed to copy text: ', err);
                        // Fallback to execCommand
                        const textarea = document.createElement('textarea');
                        textarea.value = '{}';
                        document.body.appendChild(textarea);
                        textarea.select();
                        document.execCommand('copy');
                        document.body.removeChild(textarea);
                    }});
                }} else {{
                    // Fallback for older browsers
                    const textarea = document.createElement('textarea');
                    textarea.value = '{}';
                    document.body.appendChild(textarea);
                    textarea.select();
                    document.execCommand('copy');
                    document.body.removeChild(textarea);
                }}
                "#,
                content.replace('\\', '\\\\').replace('\'', '\\\'').replace('\n', '\\n'),
                content.replace('\\', '\\\\').replace('\'', '\\\'').replace('\n', '\\n'),
                content.replace('\\', '\\\\').replace('\'', '\\\'').replace('\n', '\\n')
            );
            
            match js_sys::eval(&js_code) {
                Ok(_) => log::debug!("Text copied to clipboard: {}", content),
                Err(e) => log::error!("Failed to copy to clipboard: {:?}", e),
            }
        });
    }
    
    #[cfg(not(target_arch = "wasm32"))]
    {
        log::warn!("Clipboard operations only available in WASM builds: {}", content);
    }
}
