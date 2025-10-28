use dioxus::prelude::*;

#[component]
pub fn Checkbox(checked: bool, onchange: EventHandler<bool>) -> Element {
    rsx! {
        input {
            r#type: "checkbox",
            checked: checked,
            onchange: move |event| {
                onchange.call(event.checked())
            }
        }
    }
}
