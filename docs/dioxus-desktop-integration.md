# Dioxus Desktop Integration Patterns

## JavaScript Evaluation in Dioxus Desktop

Dioxus provides the `document::eval()` function for JavaScript injection in desktop applications:

```rust
use dioxus::prelude::*;

fn component() -> Element {
    use_effect(move || {
        spawn(async {
            let result = document::eval(r#"
                document.querySelector('.sidebar')?.style.setProperty('display', 'block', 'important');
                return 'success';
            "#).await;
            println!("Script result: {:?}", result);
        });
    });
    
    rsx! { div { "Component content" } }
}
```

## Key Requirements

1. **Must use `use_effect` or event handlers** - Never call `eval` in component body
2. **Must spawn async tasks** - JavaScript evaluation is asynchronous
3. **Proper error handling** - Always handle potential JavaScript errors

## WebView Access Pattern

Dioxus desktop uses the native WebView, so JavaScript injection works through `document::eval()` rather than direct webview handle access.

## Context Menu Implementation

For NSMenu callbacks, need to research Objective-C target-action pattern and bridge to Rust closures.

## Notification Management

macOS notification clearing requires NSUserNotificationCenter or UNUserNotificationCenter APIs.