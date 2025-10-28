//! Login page components for the multi-step authentication flow

use crate::environment::Environment;
use dioxus::prelude::*;

use super::super::reducer::{LoginAction, LoginSignal, Selection, handle_login_action};
use super::main_components::PageVisibility;
use super::ui_elements::InstanceList;
use crate::constants::ui_text;
use crate::loc;
use crate::widgets::*;

#[component]
pub fn Page1(
    visibility: PageVisibility,
    login_signal: LoginSignal,
    environment: Signal<Environment>,
) -> Element {
    let class = visibility.class();
    rsx! {
        VStack { class: "page1 page gap-4 {class}",
            input {
                r#type: "text",
                placeholder: ui_text::server_selection_placeholder(),
                autocomplete: "off",
                spellcheck: "false",
                oninput: move |evt| {
                    let value = evt.value().clone();
                    handle_login_action(login_signal, LoginAction::SelectInstance(Selection::Host(value)), &environment.read());
                }
            }
            InstanceList { login_signal: login_signal, environment: environment }
        }
    }
}

#[component]
pub fn Page2(
    visibility: PageVisibility,
    login_signal: LoginSignal,
    environment: Signal<Environment>,
    code: Signal<String>,
) -> Element {
    use copypasta::ClipboardProvider;
    let class = visibility.class();
    let website_text = loc!("A website should just have opened in your browser.");
    let authorize_text =
        loc!("Please authorize CYRUP and then copy & paste the code into the box below.");
    let paste_text = loc!("Paste");
    let copy_url_text = loc!("Copy the browser URL to the clipboard");
    rsx! {
        VStack { class: "page2 page gap-4 {class}",
            div { class: "p-2",
                Paragraph { "{website_text}" }
                Paragraph { style: TextStyle::Secondary, "{authorize_text}" }
            }
            HStack { class: "gap-2 items-center",
                input {
                    r#type: "text",
                    class: "flex-grow",
                    placeholder: ui_text::code_placeholder(),
                    autocomplete: "off",
                    spellcheck: "false",
                    value: "{code.read()}",
                    oninput: move |evt| {
                        let value = evt.value().clone();
                        code.set(value);
                    }
                }
                button {
                    class: "paste-button",
                    onclick: move |_| {
                        if let Ok(mut ctx) = copypasta::ClipboardContext::new()
                            && let Ok(s) = ctx.get_contents() {
                                code.set(s);
                            }
                    },
                    "{paste_text}"
                }
            }
            small { class: "p-2 text-[var(--g-font-size--3)] text-[var(--g-textColorDark)]",
                a {
                    style: "text-decoration: underline; cursor: pointer;",
                    onclick: move |_| {
                        if let Some(url) = login_signal.read().app_data.as_ref().and_then(|e| e.url.clone())
                            && let Ok(mut ctx) = copypasta::ClipboardContext::new() {
                                let _ = ctx.set_contents(url);
                            }
                    },
                    "{copy_url_text}"
                }
            }
        }
    }
}

#[component]
pub fn Page3(
    visibility: PageVisibility,
    login_signal: LoginSignal,
    environment: Signal<Environment>,
) -> Element {
    let name = login_signal
        .read()
        .account
        .as_ref()
        .map(|a| a.display_name.clone())
        .unwrap_or_default();

    let class = visibility.class();
    let alpha_text = loc!("CYRUP is still a very early alpha. Expect bugs and missing features.");
    let feedback_text = loc!("You can report feedback by sending me a private message.");
    let tip_text =
        loc!("One Tip: Tap a selection in the left column twice, to scroll to the timeline bottom");

    let follow_button = (!login_signal.read().did_follow).then(|| {
        rsx! {
            button {
                onclick: move |_| handle_login_action(login_signal, LoginAction::ActionFollow, &environment.read()),
                "Follow me (@terhechte@mastodon.social)",
            }
        }
    });

    rsx! {
        VStack {
            class: "page3 page gap-4 {class}",
            div {
                class: "p-2",
                h5 { "👋 {name}" }
                Paragraph { "{alpha_text}" }
                Paragraph {
                    style: TextStyle::Secondary,
                    "{feedback_text}"
                }
                {follow_button}
                Paragraph {
                    style: TextStyle::Secondary,
                    "{tip_text}"
                }
            }
        }
    }
}
