//! Image handling components for post attachments

#![allow(non_snake_case)]

use crate::constants::ui_text;
use crate::view_model::AttachmentMedia;
use crate::widgets::*;
use dioxus::prelude::*;

use super::super::{PostAction, PostSignal, handle_post_action};

#[component]
pub fn ImagesView(
    store: PostSignal,
    environment: Signal<crate::environment::Environment>,
) -> Element {
    if store.read().images.is_empty() {
        return rsx! { div {} };
    }

    rsx! {
        VStack {
            class: "images",
            for (index, image) in store.read().images.iter().enumerate() {
                SingleImageView {
                    store: store,
                    environment: environment,
                    index: index,
                    image: image.clone()
                }
            }
        }
    }
}

#[component]
pub fn SingleImageView(
    store: PostSignal,
    environment: Signal<crate::environment::Environment>,
    index: usize,
    image: AttachmentMedia,
) -> Element {
    let mut is_editing = use_signal(|| false);
    let mut text = use_signal(String::new);
    let is_uploaded = image.server_id.is_some();
    rsx!(
        div {
            HStack { class: "p-2 items-center gap-2",
                { image.preview.as_ref().map(|preview| rsx!(img {
                    style: "object-fit: cover",
                    class: "preview-image",
                    src: "{preview}"
                }))},
                { if *is_editing.read() && is_uploaded {rsx!{
                    input {
                        class: "flex-grow",
                        placeholder: ui_text::media_description_placeholder(),
                        value: "{text.read()}",
                        oninput: move |evt| {
                            *text.write() = evt.value();
                        },
                        autofocus: "true",
                    }
                    IconButton {
                        icon: crate::icons::ICON_OK,
                        title: "Save",
                        onclick: move |_| {
                            is_editing.set(false);
                            let value = text.read().clone();
                            handle_post_action(store, PostAction::UpdateImageDescription(index, value), &environment.read());
                        }
                    }
                    IconButton {
                        icon: crate::icons::ICON_CANCEL,
                        title: "Cancel",
                        onclick: move |_| {
                            is_editing.set(false);
                        }
                    }
                }} else {rsx!{
                    span {
                        class: "text-[var(--g-font-size--3)] text-[var(--g-textColor)]",
                        "{image.filename}"
                    }
                    span {
                        class: "text-[var(--g-font-size--3)] text-[var(--g-textColorDark)] mr-auto overflow-y-hidden",
                        "{text.read()}"
                    }
                    { is_uploaded.then(|| rsx!(
                        IconButton {
                            icon: crate::icons::ICON_EDIT_CAPTION,
                            title: "Edit Description",
                            onclick: move |_| {
                                is_editing.set(true);
                            }
                        }
                    ))}
                    { (!is_uploaded).then(|| rsx!(
                        Spinner {
                            class: "mt-1 me-3"
                        }
                    ))}
                    IconButton {
                        icon: crate::icons::ICON_INFO,
                        title: "Show on Disk",
                        onclick: move |_| {
                            handle_post_action(store, PostAction::ShowImageDisk(index), &environment.read());
                        }
                    }
                    IconButton {
                        icon: crate::icons::ICON_DELETE,
                        title: "Remove",
                        onclick: move |_| {
                            handle_post_action(store, PostAction::RemoveImage(index), &environment.read());
                        }
                    }
                }}}
            }
        }
    )
}
