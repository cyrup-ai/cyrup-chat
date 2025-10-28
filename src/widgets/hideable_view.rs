use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct HideableViewProps {
    #[props(optional)]
    pub class: Option<String>,
    pub hidden: bool,
    pub children: Element,
}

pub fn HideableView(props: HideableViewProps) -> Element {
    let custom_cls = props.class.as_deref().unwrap_or_default();
    let h = props.hidden;
    let s = if h {
        "display: none;"
    } else {
        "display: flex; flex-grow: 1;"
    };
    rsx! {
        div {
            style: "{s}",
            class: "{custom_cls}",
            {props.children}
        }
    }
}
