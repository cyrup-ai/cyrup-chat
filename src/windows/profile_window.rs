#![allow(non_snake_case)]

use std::rc::Rc;

use crate::environment::OpenWindowState;
use crate::environment::{Environment, types::AppEvent};
#[allow(dead_code)] // Profile window system - pending UI integration
use crate::view_model::AccountViewModel;

use dioxus::prelude::*;

#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)] // Profile window state - architectural scaffolding pending integration
pub struct ProfileWindowState {
    profile: AccountViewModel,
}

impl ProfileWindowState {
    #[allow(dead_code)] // Profile window constructor - pending profile window integration
    pub fn new(profile: AccountViewModel) -> Self {
        Self { profile }
    }
}

#[derive(Clone, Debug)]
#[allow(dead_code)] // Profile window actions - architectural scaffolding pending integration
pub enum ProfileAction {
    Save(Box<AccountViewModel>),
    Cancel,
    AppEvent(AppEvent),
}

impl OpenWindowState for ProfileWindowState {
    type Action = ProfileAction;

    fn window(
        &self,
        _environment: &Environment,
        receiver: flume::Receiver<AppEvent>,
        parent_handler: Rc<dyn Fn(Self::Action)>,
    ) -> Element {
        let profile = self.profile.clone();
        let mut display_name = use_signal(|| profile.display_name.clone());
        let mut note = use_signal(|| profile.note_plain.clone());
        let mut locked = use_signal(|| profile.locked);
        let mut bot = use_signal(|| profile.bot);

        // Handle events from receiver
        // Handle events using modern use_future pattern
        use_future({
            let parent_handler = parent_handler.clone();
            let receiver = receiver.clone();
            move || {
                let handler_clone = parent_handler.clone();
                let receiver_clone = receiver.clone();
                async move {
                    for event in receiver_clone.try_iter() {
                        handler_clone(ProfileAction::AppEvent(event));
                    }
                }
            }
        });

        rsx! {
            div {
                class: "profile-edit-window",
                style: "padding: 20px; max-width: 600px;",

                h2 { "Edit Profile" }

                div {
                    class: "form-group",
                    style: "margin-bottom: 15px;",

                    label {
                        style: "display: block; margin-bottom: 5px; font-weight: bold;",
                        "Display Name"
                    }
                    input {
                        style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px;",
                        r#type: "text",
                        value: "{display_name}",
                        oninput: move |evt| display_name.set(evt.value()),
                        placeholder: "Your display name"
                    }
                }

                div {
                    class: "form-group",
                    style: "margin-bottom: 15px;",

                    label {
                        style: "display: block; margin-bottom: 5px; font-weight: bold;",
                        "Bio"
                    }
                    textarea {
                        style: "width: 100%; padding: 8px; border: 1px solid #ccc; border-radius: 4px; min-height: 100px; resize: vertical;",
                        value: "{note}",
                        oninput: move |evt| note.set(evt.value()),
                        placeholder: "Tell us about yourself..."
                    }
                }

                div {
                    class: "form-group checkbox-group",
                    style: "margin-bottom: 15px;",

                    label {
                        style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: locked(),
                            onchange: move |evt| locked.set(evt.checked()),
                        }
                        "Lock account (require approval for new followers)"
                    }
                }

                div {
                    class: "form-group checkbox-group",
                    style: "margin-bottom: 20px;",

                    label {
                        style: "display: flex; align-items: center; gap: 8px; cursor: pointer;",
                        input {
                            r#type: "checkbox",
                            checked: bot(),
                            onchange: move |evt| bot.set(evt.checked()),
                        }
                        "This is a bot account"
                    }
                }

                div {
                    class: "button-group",
                    style: "display: flex; gap: 10px; justify-content: flex-end;",

                    button {
                        style: "padding: 10px 20px; border: 1px solid #ccc; background: white; border-radius: 4px; cursor: pointer;",
                        onclick: {
                            let parent_handler = parent_handler.clone();
                            move |_| {
                                parent_handler(ProfileAction::Cancel);
                            }
                        },
                        "Cancel"
                    }

                    button {
                        style: "padding: 10px 20px; border: none; background: #1976d2; color: white; border-radius: 4px; cursor: pointer;",
                        onclick: {
                            let parent_handler = parent_handler.clone();
                            move |_| {
                                // Create a modified AccountViewModel
                                let mut updated_profile = profile.clone();
                                updated_profile.display_name = display_name();
                                updated_profile.note_plain = note();
                                updated_profile.locked = locked();
                                updated_profile.bot = bot();

                                parent_handler(ProfileAction::Save(Box::new(updated_profile)));
                            }
                        },
                        "Save Changes"
                    }
                }
            }
        }
    }
}
