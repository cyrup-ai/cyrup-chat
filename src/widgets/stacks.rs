use dioxus::prelude::*;

#[derive(Props, Clone, PartialEq)]
pub struct VStackProps {
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

pub fn VStack(props: VStackProps) -> Element {
    let custom_cls = props.class.unwrap_or_default();
    rsx!(
        div { class: "flex flex-col relative {custom_cls}", {props.children} }
    )
}

#[derive(Props, Clone, PartialEq)]
pub struct HStackProps {
    #[props(optional)]
    pub class: Option<String>,
    pub children: Element,
}

pub fn HStack(props: HStackProps) -> Element {
    let custom_cls = props.class.unwrap_or_default();
    rsx!(
        div { class: "flex flex-row relative {custom_cls}", {props.children} }
    )
}
