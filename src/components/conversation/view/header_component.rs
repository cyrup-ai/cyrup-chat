use crate::icons;
use crate::view_model::*;
use crate::widgets::*;
use dioxus::prelude::*;

use super::super::ConversationSignal;
use super::super::reducer::Action;
#[allow(unused_imports)] // Used in onclick handler for PublicAction::OpenLink
use crate::PublicAction;

#[component]
pub fn UserConversationHeader(
    status: StatusViewModel,
    store: ConversationSignal,
    on_action: EventHandler<Action>,
) -> Element {
    let status_clone = status.clone();
    let on_action_clone = on_action;

    rsx! {
        div {
            HStack {
                class: "toolbar justify-between justify-items-center p-2 flex-grow items-center",
                div {
                    class: "icon-button",
                    button {
                        r#type: "button",
                        onclick: move |_| { on_action(Action::Close); },
                        dangerous_inner_html: "{icons::ICON_CANCEL}"
                    }
                }
                div {
                    class: "mr-auto p-1 no-selection",
                    Label {
                        style: TextStyle::Primary,
                        "Conversation"
                    }
                }
                div {
                    class: "icon-button",
                    button {
                        r#type: "button",
                        onclick: move |_evt| {
                            use crate::PublicAction::*;
                            // Native context menu - open browser link directly
                            on_action_clone(Action::Public(Box::new(OpenLink(status_clone.uri.clone()))));
                        },
                        dangerous_inner_html: "{icons::ICON_MORE}"
                    }
                }
            }
        }
    }
}
