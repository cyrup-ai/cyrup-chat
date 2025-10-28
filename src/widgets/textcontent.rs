use dioxus::prelude::*;

use crate::helper::HtmlItem;

#[derive(Debug, Clone)]
pub enum TextContentAction {
    Tag(String),
    Link(String),
    Account(String),
}

#[component]
pub fn TextContent(
    content: Vec<HtmlItem>,
    onclick: EventHandler<TextContentAction>,
    class: String,
) -> Element {
    use crate::helper::HtmlItem::*;

    let div_class = format!("attributed-text {class}");

    // Collect items into owned data to avoid borrow checker issues with closures
    let content_elements: Vec<_> = content.into_iter().map(|item| match item {
        Text { content } => rsx! { span { "{content} " } },
        Mention { url, name } => {
            let url_owned = url.clone();
            rsx! {
                span {
                    class: "mention",
                    onclick: move |_| onclick.call(TextContentAction::Account(url_owned.clone())),
                    "{name}"
                }
            }
        },
        Link { name, url } => {
            let url_owned = url.clone();
            rsx! {
                span {
                    class: "link",
                    onclick: move |_| onclick.call(TextContentAction::Link(url_owned.clone())),
                    "{name} "
                }
            }
        },
        Hashtag { name } => {
            let name_owned = name.clone();
            rsx! {
                span {
                    class: "tag",
                    onclick: move |_| onclick.call(TextContentAction::Tag(name_owned.clone())),
                    "{name} "
                }
            }
        },
        Image { url } => rsx! {
            img {
                src: "{url}",
                class: "emoji-entry"
            }
        },
        Break => rsx! { br {} }
    }).collect();

    rsx! {
        div {
            class: div_class,
            p {
                {content_elements.into_iter()}
            }
        }
    }
}
