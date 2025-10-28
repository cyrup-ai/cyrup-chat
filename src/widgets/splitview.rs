#![allow(non_snake_case)]
use dioxus::prelude::*;

#[allow(dead_code)] // Split view CSS - pending integration
const STYLE: &str = r#"
    .sb-main {
        width: 100%;
        height: 100vh;
        display: flex;
        user-select: none;
    }
    .sb-sidebar {
        box-sizing: border-box;
        flex-shrink: 0;
        height: 100vh;
        width: 290px;
        min-width: 190px;
        max-width: 450px;
    }
    .sb-resize {
        box-sizing: border-box;
        width: 5px;
        flex: 0 0 auto;
        cursor: ew-resize;
        padding: 0;
        user-select: none;
        -webkit-user-select: none;
        /*background-color: var(--g-backgroundWindow);*/
        background-color: transparent;
        border-right: 1px solid #000;
    }
    .sb-content {
        flex-grow: 1;
        box-sizing: border-box;
        height: 100%;
    }
    .sb-is-resizing * {
        user-select: none;
        -webkit-user-select: none;
        pointer-events: none;
    }
    .sb-resize.sb-is-resizing {
        border-right: 1px solid var(--g-selectedContentBackgroundColor);
    }
    .sb-resize:hover {
        border-right: 2px solid #333;
    }
    /* .sb-resize.sb-is-resizing {
        background-color: var(--g-selectedContentBackgroundColor);
    }*/
    "#;

#[component]
pub fn SplitViewComponent(sidebar: Element, content: Element) -> Element {
    let style_html = format!("<style>{STYLE}</style>");

    let mut size = use_signal(|| Some(290.0f64));
    let mut is_resizing = use_signal(|| false);

    rsx! {
        div {
            class: "sb-main",
            onmouseup: move |_| {
                if *is_resizing.read() {
                    is_resizing.set(false);
                }
            },
            onmousemove: move |event| {
                if *is_resizing.read() {
                    let value = event.data.page_coordinates().x - 8.;
                    size.set(Some(value));
                }
            },
            div {
                dangerous_inner_html: "{style_html}"
            },
            SidebarComponent {
                size: size,
                is_resizing: is_resizing,
                children: sidebar
            },
            ResizeComponent {
                size: size,
                is_resizing: is_resizing
            },
            ContentComponent {
                is_resizing: is_resizing,
                children: content
            }
        }
    }
}

#[component]
fn SidebarComponent(
    size: Signal<Option<f64>>,
    is_resizing: Signal<bool>,
    children: Element,
) -> Element {
    let style = if let Some(s) = *size.read() {
        format!("width: {s}px;")
    } else {
        "".to_string()
    };
    let class = if *is_resizing.read() {
        "sb-is-resizing"
    } else {
        ""
    };

    rsx! {
        div {
            class: format!("sb-sidebar {class}"),
            style: "{style}",
            {children}
        }
    }
}

#[component]
fn ResizeComponent(size: Signal<Option<f64>>, is_resizing: Signal<bool>) -> Element {
    let class = if *is_resizing.read() {
        "sb-is-resizing"
    } else {
        ""
    };

    rsx! {
        div {
            class: format!("sb-resize {class}"),
            onmousedown: move |_event| {
                is_resizing.set(true);
            },
            onmouseup: move |_event| {
                is_resizing.set(false);
            },
            onmouseout: move |event| {
                if *is_resizing.read() {
                    let value = event.data.page_coordinates().x - 8.;
                    size.set(Some(value));
                }
            }
        }
    }
}

#[component]
fn ContentComponent(is_resizing: Signal<bool>, children: Element) -> Element {
    let class = if *is_resizing.read() {
        "sb-is-resizing"
    } else {
        ""
    };

    rsx! {
        div {
            class: format!("sb-content {class}"),
            {children}
        }
    }
}
