use super::*;

#[component]
pub fn SidebarTextHeadline(text: String) -> Element {
    rsx! {
        Label {
            style: TextStyle::Quartery,
            class: "mb-1 mt-2",
            "{text}"
        }
    }
}

#[component]
pub fn SidebarTextEntry(
    icon: String,
    text: String,
    selected: bool,
    onclick: EventHandler<()>,
) -> Element {
    let class = if selected { "selected" } else { "" };
    rsx! {
        div {
            class: "sidebar-text-entry {class} no-selection force-pointer",
            onclick: move |_| onclick.call(()),
            span {
                class: "icon no-selection force-pointer",
                "{icon}"
            }
            Label {
                onclick: move |_| onclick.call(()),
                style: TextStyle::Secondary,
                clickable: false,
                pointer_style: PointerStyle::Pointer,
                "{text}"
            }
        }
    }
}
