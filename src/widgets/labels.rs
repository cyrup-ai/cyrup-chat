use dioxus::prelude::*;

#[allow(dead_code)] // UI text styling system - pending full integration
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub enum TextStyle {
    #[default]
    Primary,
    Secondary,
    Tertiary,
    Quartery,
}

impl TextStyle {
    fn as_css(&self) -> &'static str {
        match self {
            TextStyle::Primary => {
                "text-[var(--g-font-size--3)] text-[var(--g-textColor)] font-semibold"
            }
            TextStyle::Secondary => "text-[var(--g-font-size--3)] text-[var(--g-textColor)]",
            TextStyle::Tertiary => "text-[var(--g-font-size--3)] text-[var(--g-textColorDark)]",
            TextStyle::Quartery => {
                "text-[0.65rem] text-[var(--g-textColorDark)] uppercase font-bold"
            }
        }
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TextAlign {
    Left,
    Center,
    Right,
}

impl TextAlign {
    fn rule(&self) -> &'static str {
        match self {
            TextAlign::Left => "text-align: left;",
            TextAlign::Center => "text-align: center;",
            TextAlign::Right => "text-align: right;",
        }
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum VerticalTextAlign {
    #[default]
    Baseline,
    Top,
    Middle,
    Bottom,
}

impl VerticalTextAlign {
    fn rule(&self) -> &'static str {
        match self {
            VerticalTextAlign::Baseline => "vertical-align: baseline;",
            VerticalTextAlign::Top => "vertical-align: top;",
            VerticalTextAlign::Middle => "vertical-align: middle;",
            VerticalTextAlign::Bottom => "vertical-align: bottom;",
        }
    }
}

#[allow(unused)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Default)]
pub enum PointerStyle {
    #[default]
    Default,
    Pointer,
}

impl PointerStyle {
    // this `pub` is needed for IconTextButton. Once we have a proper button abstraction, remove
    pub fn rule(&self) -> &'static str {
        match self {
            PointerStyle::Default => "cursor: default;",
            PointerStyle::Pointer => "cursor: pointer;",
        }
    }
}

#[component]
pub fn Paragraph(
    style: Option<TextStyle>,
    class: Option<&'static str>,
    pointer_style: Option<PointerStyle>,
    children: Element,
) -> Element {
    let style_class = style.unwrap_or_default().as_css();
    let class = class.unwrap_or_default();
    let pointer_style = pointer_style.unwrap_or_default().rule();

    rsx!(p {
        class: "{style_class} {class} no-selection",
        style: "{pointer_style}",
        {children}
    })
}

#[derive(Props, Clone, PartialEq)]
pub struct LabelProps {
    #[props(optional)]
    pub style: Option<TextStyle>,
    #[props(optional)]
    pub class: Option<&'static str>,
    #[props(optional)]
    pub force_singleline: Option<bool>,
    #[props(optional)]
    pub selectable: Option<bool>,
    #[props(optional)]
    pub clickable: Option<bool>,
    #[props(optional)]
    pub title: Option<String>,
    #[props(optional)]
    pub onclick: Option<EventHandler<MouseEvent>>,
    #[props(optional)]
    pub alignment: Option<TextAlign>,
    #[props(optional)]
    pub vertical_alignment: Option<VerticalTextAlign>,
    #[props(optional)]
    pub pointer_style: Option<PointerStyle>,
    #[props(optional)]
    pub dangerous_content: Option<String>,
    pub children: Element,
}

pub fn Label(props: LabelProps) -> Element {
    let style_class = props.style.unwrap_or_default().as_css();
    let class = props.class.unwrap_or_default();
    let alignment = props.alignment.map(|e| e.rule()).unwrap_or_default();
    let valign = props
        .vertical_alignment
        .map(|e| e.rule())
        .unwrap_or_default();
    let singleline = wrp(&props.force_singleline, "overflow-y-hidden no-wrap");
    let selection = if props.selectable.unwrap_or_default() {
        ""
    } else {
        "no-selection"
    };
    let clickable = wrp(&props.clickable, "hover:underline cursor-pointer");
    let pointer_style = props.pointer_style.unwrap_or_default().rule();

    let onclick_handler = props.onclick;
    let handler = move |ev: MouseEvent| {
        if let Some(o) = &onclick_handler {
            o.call(ev)
        }
    };

    // lots of overhead just so we can support custom emoji via dangerous content
    if let Some(dangerdanger) = props.dangerous_content {
        rsx!(
                    span {
                        class: "{style_class} {selection} {singleline} {clickable} {class}",
                        style: "{alignment} {valign} {pointer_style}",
                        title: props.title,
                        onclick: handler,
                        dangerous_inner_html: "{dangerdanger}",
        {props.children}
                    }
                )
    } else {
        rsx!(
                    span {
                        class: "{style_class} {selection} {singleline} {clickable} {class}",
                        style: "{alignment} {valign} {pointer_style}",
                        title: props.title,
                        onclick: handler,
        {props.children}
                    }
                )
    }
}

fn wrp(input: &Option<bool>, k: &'static str) -> &'static str {
    match input {
        Some(true) => k,
        _ => "",
    }
}
