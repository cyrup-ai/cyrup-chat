//! UI elements and interactive components for the login interface

use crate::environment::Environment;
use dioxus::prelude::*;

use super::super::reducer::{LoginAction, LoginSignal, Selection, handle_login_action};
use super::main_components::ButtonConfig;
use crate::widgets::*;

#[component]
pub fn PageButtons(loading: bool, left: ButtonConfig, right: ButtonConfig) -> Element {
    let lc = left.classes();
    let rc = right.classes();
    let spinner = loading.then(|| rsx!(Spinner {}));
    rsx! {
        HStack { class: "justify-between items-center",
            button { class: "{lc}", onclick: move |_| left.onclick.call(()), disabled: "{left.disabled()}", "{left.title}" }
            {spinner}
            button {
                class: "{rc} highlighted",
                onclick: move |_| right.onclick.call(()),
                disabled: "{right.disabled()}",
                "{right.title}"
            }
        }
    }
}

#[component]
pub fn InstanceList(login_signal: LoginSignal, environment: Signal<Environment>) -> Element {
    // Use a memo to get the current instances and selected instance
    let instances = use_memo(move || login_signal.read().instances.clone());
    let selected_instance = use_memo(move || login_signal.read().selected_instance.clone());

    rsx! {
        div {
            class: "login-instance-list scroll",
            for x in instances() {
                InstanceView {
                    key: "{x.name}",
                    image: x.thumbnail.as_deref().map(|s| s.to_string()),
                    name: x.name.clone(),
                    users: x.users.clone(),
                    selected: selected_instance().as_ref() == Some(&x),
                    onclick: {
                        let x_clone = x.clone();
                        let login_signal_clone = login_signal;
                        let env_clone = environment;
                        move |_| {
                            handle_login_action(
                                login_signal_clone,
                                LoginAction::SelectInstance(Selection::Instance(x_clone.clone())),
                                &env_clone.read()
                            );
                        }
                    }
                }
            }
        }
    }
}

#[component]
pub fn InstanceView(
    image: Option<Option<String>>,
    name: String,
    users: String,
    selected: bool,
    onclick: EventHandler<()>,
) -> Element {
    // Safe number formatting with proper error handling
    use numfmt::*;

    // Create formatter with fallback if separator fails
    let mut f = match Formatter::new().separator(',') {
        Ok(formatter) => formatter.precision(Precision::Decimals(0)),
        Err(e) => {
            log::warn!("Failed to create number formatter with separator: {e}");
            Formatter::new().precision(Precision::Decimals(0))
        }
    };

    // Parse user count safely
    let value: f64 = users.parse().unwrap_or_default();
    #[allow(deprecated)]
    let rendered = f.fmt(value);
    let class = if selected { "selected" } else { "" };
    rsx! {
        div { onclick: move |_| onclick.call(()),
            HStack { class: "gap-4 p-3 login-instance {class} items-center",
                { image.flatten().map(|img| rsx!(img {
                width: 32,
                height: 32,
                src: "{img}"
            })).unwrap_or_else(|| rsx!(div {
                class: "no-img"
            }))},
                Label { onclick: move |_| onclick.call(()), clickable: true, pointer_style: PointerStyle::Pointer, class: "mr-auto", "https://{name}" }
                Label { clickable: true, onclick: move |_| onclick.call(()), pointer_style: PointerStyle::Pointer, style: TextStyle::Secondary, "{rendered} Users" }
            }
        }
    }
}
