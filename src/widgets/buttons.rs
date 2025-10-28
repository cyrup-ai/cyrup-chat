use super::{Label, PointerStyle};
use dioxus::prelude::*;

#[component]
pub fn EmojiButton() -> Element {
    // On windows and linux, emoji popup functionality is not available
    #[cfg(any(target_os = "windows", target_os = "linux"))]
    return rsx! { div {} };

    #[cfg(target_os = "macos")]
    {
        // Get environment from context to access platform functionality
        let environment = crate::app::use_environment();

        rsx! {
            div {
                class: "icon-button",
                title: "Emoji & Symbols",
                button {
                    onmousedown: move |_| {
                        // Use environment platform to show emoji popup
                        let env = environment.read();
                        if let Err(e) = env.platform.show_emoji_popup() {
                            log::error!("Failed to show emoji popup: {}", e);
                        }
                    },
                    dangerous_inner_html: crate::icons::ICON_EMOJI
                }
            }
        }
    }
}

#[derive(Props, Clone, PartialEq)]
pub struct IconButtonProps<S: AsRef<str> + Clone + PartialEq + 'static> {
    pub icon: &'static str,
    pub title: S,
    #[props(optional)]
    pub class: Option<&'static str>,
    pub onclick: EventHandler<MouseEvent>,
}

pub fn IconButton<S: AsRef<str> + Clone + PartialEq + 'static>(
    props: IconButtonProps<S>,
) -> Element {
    let class = props.class.unwrap_or_default();
    rsx!(
        div { class: "icon-button {class}", title: "{props.title.as_ref()}", button { r#type: "button", onclick: move |e| props.onclick.call(e), dangerous_inner_html: props.icon } }
    )
}

#[derive(PartialEq, Props, Clone)]
pub struct IconProps<S: AsRef<str> + Clone + PartialEq + 'static> {
    pub icon: &'static str,
    pub title: S,
    #[props(optional)]
    pub class: Option<String>,
}

#[allow(dead_code)] // Widget component - pending usage in UI redesign
pub fn Icon<S: AsRef<str> + Clone + PartialEq + 'static>(props: IconProps<S>) -> Element {
    let class = props.class.unwrap_or_default();
    rsx!(
        div { class: "icon {class}", title: "{props.title.as_ref()}", span { dangerous_inner_html: "{props.icon}" } }
    )
}

#[derive(Props, Clone, PartialEq)]
pub struct IconTextButtonProps<
    S: AsRef<str> + Clone + PartialEq + 'static,
    ST: AsRef<str> + Clone + PartialEq + 'static,
> {
    pub icon: &'static str,
    pub text: S,
    pub title: ST,
    #[props(optional)]
    pub class: Option<&'static str>,
    #[props(optional)]
    pub disabled: Option<bool>,
    pub onclick: EventHandler<MouseEvent>,
}

pub fn IconTextButton<
    S: AsRef<str> + Clone + PartialEq + 'static,
    ST: AsRef<str> + Clone + PartialEq + 'static,
>(
    props: IconTextButtonProps<S, ST>,
) -> Element {
    let disabled = props
        .disabled
        .and_then(|e| e.then_some("disabled"))
        .unwrap_or("");
    let pointer_style = props
        .disabled
        .and_then(|e| e.then_some(PointerStyle::Default))
        .unwrap_or(PointerStyle::Pointer);
    let rule = pointer_style.rule();
    let class = props.class.unwrap_or_default();
    rsx!(
        div {
            style: "{rule}",
            class: "icon-button text {disabled} {class}",
            title: "{props.title.as_ref()}",
            onclick: move |e| props.onclick.call(e),
            button { r#type: "button", style: "{rule}", dangerous_inner_html: props.icon }
            Label { onclick: move |e| props.onclick.call(e), pointer_style: pointer_style, "{props.text.as_ref()}" }
        }
    )
}

#[derive(Props, Clone, PartialEq)]
pub struct TextButtonProps<
    S: AsRef<str> + Clone + PartialEq + 'static,
    ST: AsRef<str> + Clone + PartialEq + 'static,
> {
    pub text: S,
    pub title: ST,
    #[props(optional)]
    pub class: Option<&'static str>,
    #[props(optional)]
    pub disabled: Option<bool>,
    pub onclick: EventHandler<MouseEvent>,
}

#[allow(unused)]
pub fn TextButton<
    S: AsRef<str> + Clone + PartialEq + 'static,
    ST: AsRef<str> + Clone + PartialEq + 'static,
>(
    props: TextButtonProps<S, ST>,
) -> Element {
    let disabled = props
        .disabled
        .and_then(|e| e.then_some("disabled"))
        .unwrap_or("");
    let pointer_style = props
        .disabled
        .and_then(|e| e.then_some(PointerStyle::Default))
        .unwrap_or(PointerStyle::Pointer);
    let rule = pointer_style.rule();
    let class = props.class.unwrap_or_default();
    rsx!(
        div {
            style: "{rule}",
            class: "text-button {disabled} {class}",
            title: "{props.title.as_ref()}",
            onclick: move |e| props.onclick.call(e),
            Label { onclick: move |e| props.onclick.call(e), pointer_style: pointer_style, "{props.text.as_ref()}" }
        }
    )
}
