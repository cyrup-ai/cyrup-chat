use dioxus::prelude::*;

#[component]
pub fn Spinner(class: Option<String>) -> Element {
    let c = class.as_deref().unwrap_or_default();
    rsx! {
        div {
            class: "loader {c}"
        }
    }
}
