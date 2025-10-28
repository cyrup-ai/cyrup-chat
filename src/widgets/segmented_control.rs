use dioxus::prelude::*;

pub trait Segment: Eq + PartialEq + Clone {
    fn id(&self) -> u64;
    fn label(&self) -> String;
    fn selected(&self) -> bool;
    fn dot(&self) -> bool;
}

#[component]
pub fn SegmentedControl<Item: Segment + 'static>(
    items: Vec<Item>,
    onclick: EventHandler<Item>,
) -> Element {
    let items_list = items.clone();
    rsx! {
        div {
            class: "tabbar",
            {
                items_list.iter().map(|item| {
                    let item_clone = item.clone();
                    rsx! {
                        TabButton {
                            key: "{item.id()}",
                            label: item.label(),
                            onclick: move |_| onclick.call(item_clone.clone()),
                            selected: item.selected(),
                            dot: item.dot(),
                        }
                    }
                })
            }
        }
    }
}

#[component]
fn TabButton(label: String, onclick: EventHandler<()>, selected: bool, dot: bool) -> Element {
    let dot_element = if dot {
        Some(rsx! { span { class: "dot" } })
    } else {
        None
    };

    let class = if selected {
        "button selected"
    } else {
        "button"
    };

    rsx! {
        button {
            class: "{class}",
            onclick: move |_| onclick.call(()),
            {dot_element}
            "{label}"
        }
    }
}
