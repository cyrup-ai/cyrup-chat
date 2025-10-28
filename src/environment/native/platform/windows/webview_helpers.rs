//! WebView JavaScript execution helpers for Windows platform
//!
//! This module provides utilities for executing JavaScript in Dioxus WebView
//! from the Windows platform layer.

use std::sync::Arc;
use tokio::sync::oneshot;

/// WebView JavaScript execution result
#[derive(Debug)]
pub enum WebViewResult {
    Success(String),
    Error(String),
    NotFound,
}

/// Execute JavaScript in the WebView and return the result
pub async fn execute_webview_script(script: &str) -> Result<WebViewResult, String> {
    use dioxus::document;

    log::debug!(
        "Executing WebView script: {}...",
        &script[..script.len().min(50)]
    );

    // Execute the script using Dioxus WebView API
    let eval_result = document::eval(script);

    // Wait for the result with a timeout
    match tokio::time::timeout(std::time::Duration::from_secs(5), eval_result).await {
        Ok(Ok(result)) => {
            let result_str = result.as_str().unwrap_or("undefined");
            log::debug!("WebView script executed successfully: {}", result_str);
            Ok(WebViewResult::Success(result_str.to_string()))
        }
        Ok(Err(e)) => {
            let error_msg = format!("Script execution error: {:?}", e);
            log::error!("{}", error_msg);
            Ok(WebViewResult::Error(error_msg))
        }
        Err(_) => {
            let timeout_msg = "Script execution timed out".to_string();
            log::error!("{}", timeout_msg);
            Ok(WebViewResult::Error(timeout_msg))
        }
    }
}

/// Generate JavaScript for scrolling to an element
pub fn generate_scroll_script(element_id: &str, behavior: &str) -> String {
    format!(
        r#"
        (function() {{
            try {{
                const element = document.getElementById('{}');
                if (element) {{
                    element.scrollIntoView({{
                        behavior: '{}',
                        block: 'start',
                        inline: 'nearest'
                    }});
                    return 'success';
                }} else {{
                    // Try querySelector if ID not found
                    const queryElement = document.querySelector('{}');
                    if (queryElement) {{
                        queryElement.scrollIntoView({{
                            behavior: '{}',
                            block: 'start',
                            inline: 'nearest'
                        }});
                        return 'success';
                    }}
                    return 'element_not_found';
                }}
            }} catch (error) {{
                return 'error: ' + error.message;
            }}
        }})();
        "#,
        element_id.replace('\'', "\\'"),
        behavior,
        element_id.replace('\'', "\\'"),
        behavior
    )
}

/// Generate JavaScript for setting text size
pub fn generate_text_size_script(size: f32) -> String {
    format!(
        r#"
        (function() {{
            try {{
                // Method 1: CSS zoom property
                document.body.style.zoom = '{}';
                
                // Method 2: CSS transform scale (fallback)
                if (!document.body.style.zoom) {{
                    document.body.style.transform = 'scale({})';
                    document.body.style.transformOrigin = 'top left';
                }}
                
                // Method 3: CSS custom property for fine-grained control
                document.documentElement.style.setProperty('--text-scale', '{}');
                
                // Store preference in localStorage
                localStorage.setItem('textSize', '{}');
                
                return 'success';
            }} catch (error) {{
                return 'error: ' + error.message;
            }}
        }})();
        "#,
        size, size, size, size
    )
}

/// Generate JavaScript for sidebar visibility
pub fn generate_sidebar_visibility_script(visible: bool) -> String {
    let action = if visible { "remove" } else { "add" };
    let state = if visible { "true" } else { "false" };

    format!(
        r#"
        (function() {{
            try {{
                const sidebar = document.querySelector('.sidebar, #sidebar, [data-sidebar]');
                if (sidebar) {{
                    // Toggle hidden class
                    sidebar.classList.{}('hidden');
                    
                    // Update aria-hidden for accessibility
                    sidebar.setAttribute('aria-hidden', '{}');
                    
                    // Store preference in localStorage
                    localStorage.setItem('sidebarVisible', '{}');
                    
                    // Trigger custom event for other components
                    window.dispatchEvent(new CustomEvent('sidebarToggle', {{
                        detail: {{ visible: {} }}
                    }}));
                    
                    return 'success';
                }} else {{
                    return 'sidebar_not_found';
                }}
            }} catch (error) {{
                return 'error: ' + error.message;
            }}
        }})();
        "#,
        action, !visible, state, visible
    )
}

/// Generate JavaScript for context menu positioning
pub fn generate_context_menu_script(x: i32, y: i32, items: &[&str]) -> String {
    let items_json = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            format!(
                r#"{{"id": {}, "label": "{}"}}"#,
                i,
                item.replace('"', "\\\"")
            )
        })
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        r#"
        (function() {{
            try {{
                // Remove existing context menu
                const existingMenu = document.getElementById('cyrup-context-menu');
                if (existingMenu) {{
                    existingMenu.remove();
                }}
                
                // Create context menu element
                const menu = document.createElement('div');
                menu.id = 'cyrup-context-menu';
                menu.style.cssText = `
                    position: fixed;
                    left: {}px;
                    top: {}px;
                    background: white;
                    border: 1px solid #ccc;
                    border-radius: 4px;
                    box-shadow: 0 2px 10px rgba(0,0,0,0.1);
                    z-index: 10000;
                    min-width: 150px;
                `;
                
                const items = [{}];
                items.forEach(item => {{
                    const menuItem = document.createElement('div');
                    menuItem.textContent = item.label;
                    menuItem.style.cssText = `
                        padding: 8px 16px;
                        cursor: pointer;
                        border-bottom: 1px solid #eee;
                    `;
                    menuItem.addEventListener('click', () => {{
                        window.cyruspContextMenuCallback(item.id);
                        menu.remove();
                    }});
                    menu.appendChild(menuItem);
                }});
                
                document.body.appendChild(menu);
                
                // Close menu on outside click
                setTimeout(() => {{
                    document.addEventListener('click', function closeMenu(e) {{
                        if (!menu.contains(e.target)) {{
                            menu.remove();
                            document.removeEventListener('click', closeMenu);
                        }}
                    }});
                }}, 0);
                
                return 'success';
            }} catch (error) {{
                return 'error: ' + error.message;
            }}
        }})();
        "#,
        x, y, items_json
    )
}
