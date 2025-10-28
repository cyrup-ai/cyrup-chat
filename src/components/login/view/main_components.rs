//! Main login components and application flow

use crate::environment::Environment;
use dioxus::prelude::*;

use super::super::reducer::{LoginAction, LoginSignal, LoginState, handle_login_action};
use super::page_components::{Page1, Page2, Page3};
use super::ui_elements::PageButtons;
use crate::loc;
use crate::widgets::*;

#[component]
pub fn LoginApp(environment: Signal<Environment>, should_show_login: Signal<bool>) -> Element {
    log::trace!("rerender login app");

    let login_signal: LoginSignal = use_signal(LoginState::default);

    let mut did_close = use_signal(|| false);

    // We're done, return the model
    // Q9: MVP hardcoded auth - no token checking needed
    if login_signal.read().done
        && let Some(model) = login_signal
            .read()
            .send_model
            .borrow()
            .as_ref()
            .map(|o| o.cloned())
    {
        let mut mutable_environment = environment().clone();
        mutable_environment.update_model(model);
        environment.set(mutable_environment);
    }

    if !(did_close()) && login_signal.read().close && login_signal.read().done {
        did_close.set(true);
        should_show_login.set(false);
    }

    rsx! {
        div { class: "login-container", MainView { login_signal: login_signal, environment: environment } }
    }
}

#[component]
pub fn MainView(login_signal: LoginSignal, environment: Signal<Environment>) -> Element {
    rsx! {
        div { class: "flex flex-col relative p-2 m-2 flex-grow items-center justify-center justify-items-center",
            div { class: "justify-self-center", Welcome { login_signal: login_signal, environment: environment } }
        }
    }
}

#[component]
pub fn Welcome(login_signal: LoginSignal, environment: Signal<Environment>) -> Element {
    use LoginAction::*;
    use PageVisibility::*;

    let entered_code = use_signal(String::new);

    let (a, b, c, action) = match (
        login_signal.read().app_data.is_some(),
        login_signal.read().done,
    ) {
        (false, false) => (Visible, Pre, Pre, ChosenInstance),
        (true, false) => (Post, Visible, Pre, EnteredCode(entered_code.read().clone())),
        (true, true) => (Post, Post, Visible, CloseLogin),
        (false, true) => return rsx!( div { "Error" } ),
    };

    let (enabled, visible_l, visible_r, t1, mut t2) = match (
        b,
        login_signal.read().selected_instance.is_some()
            || login_signal.read().selected_instance_url.is_some(),
        !(entered_code.read().is_empty()),
    ) {
        (Pre, false, _) => (false, true, true, loc!("Register"), loc!("Continue")),
        (Pre, true, _) => (true, true, true, loc!("Register"), loc!("Continue")),
        (Visible, _, false) => (true, true, true, loc!("Back"), loc!("Confirm")),
        (Visible, _, true) => (true, false, true, String::new(), loc!("Confirm")),
        _ => (true, false, true, String::new(), loc!("Done")),
    };

    if login_signal.read().selected_instance_url.is_some()
        && matches!(action, LoginAction::ChosenInstance)
    {
        t2 = loc!("Use Custom");
    }

    let has_entered_code = matches!(action, LoginAction::EnteredCode(_));

    let welcome_text = loc!("Welcome to CYRUP");
    let error_box = login_signal.read().error_message.as_ref().map(|error| {
        rsx!(ErrorBox {
            content: error.clone(),
            onclick: move |_| {}
        })
    });
    rsx! {
        div { class: "login-form no-selection",
            VStack { class: "gap-4",
                HStack { class: "gap-4 items-center",
                    h3 { "{welcome_text}" }
                    span { "𝛼" }
                }
                div { class: "page-container",
                    Page1 { visibility: a, login_signal: login_signal, environment: environment }
                    Page2 { visibility: b, login_signal: login_signal, environment: environment, code: entered_code }
                    Page3 { visibility: c, login_signal: login_signal, environment: environment }
                }
                PageButtons {
                    loading: login_signal.read().is_loading,
                    left: ButtonConfig {
                        visible: visible_l,
                        enabled,
                        title: t1,
                        onclick: EventHandler::new(move |_| {
                            if has_entered_code {
                                handle_login_action(login_signal, LoginAction::ChosenInstance, &environment.read());
                            } else {
                                handle_login_action(login_signal, LoginAction::ActionRegister, &environment.read());
                            }
                        }),
                    },
                    right: ButtonConfig {
                        visible: visible_r,
                        enabled,
                        title: t2,
                        onclick: EventHandler::new(move |_| handle_login_action(login_signal, action.clone(), &environment.read())),
                    }
                }
                {error_box}
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[allow(dead_code)] // Page visibility states - pending login animation integration
pub enum PageVisibility {
    Pre,
    Visible,
    Post,
}

impl PageVisibility {
    #[allow(dead_code)] // Page visibility CSS class mapping - pending login animation
    pub fn class(&self) -> &'static str {
        match self {
            PageVisibility::Pre => "pre-appear",
            PageVisibility::Visible => "appear",
            PageVisibility::Post => "post-appear",
        }
    }
}

#[derive(Clone, PartialEq)]
pub struct ButtonConfig {
    pub visible: bool,
    pub enabled: bool,
    pub title: String,
    pub onclick: EventHandler<()>,
}

impl ButtonConfig {
    #[allow(dead_code)] // Button CSS classes - pending login UI theming
    pub fn classes(&self) -> String {
        let mut base = "button ".to_string();
        if self.visible {
            // base.push_str("visible ");
        } else {
            base.push_str("hidden-button ");
        }
        base
    }

    #[allow(dead_code)] // Button disabled states - pending login UI theming
    pub fn disabled(&self) -> String {
        if self.enabled { "false" } else { "true" }.to_string()
    }
}
