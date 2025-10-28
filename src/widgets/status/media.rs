//! Media handling component for status content

use super::actions::StatusAction;
use crate::view_model::StatusViewModel;
use crate::widgets::*;
use dioxus::prelude::{Element, EventHandler, component, rsx};

#[component]
pub fn ContentCellMedia(
    status: StatusViewModel,
    onclick: EventHandler<StatusAction>,
    sender: EventHandler<StatusAction>,
) -> Element {
    // Collect image elements to avoid borrow checker issues
    let image_elements: Vec<_> = status
        .status_images
        .iter()
        .map(|(description, preview, url)| {
            let url_owned = url.clone();
            let url_for_context = url.clone(); // Clone for the context menu
            let description_owned = description.clone();
            let preview_owned = preview.clone();
            rsx! {
                div {
                    class: "media-object",
                    img {
                        src: "{preview_owned}",
                        alt: "{description_owned}",
                        onclick: move |_| onclick.call(StatusAction::OpenImage(url_owned.clone())),
                        oncontextmenu: move |e| {
                            e.prevent_default();
                            // Use modern Dioxus 0.7 context menu pattern for image actions
                            let environment = crate::app::use_environment();
                            let image_url = url_for_context.clone();
                            let menu_items = vec![
                                ("Open Image", StatusAction::OpenImage(image_url.clone())),
                                ("Copy Image URL", StatusAction::Copy(image_url)),
                            ];
                            let coords = e.client_coordinates();
                            environment.read().platform.show_context_menu(
                                (coords.x as i32, coords.y as i32),
                                "Image",
                                menu_items,
                                move |action| {
                                    sender.call(action);
                                }
                            );
                        },
                    }
                }
            }
        })
        .collect();

    // Collect media elements to avoid borrow checker issues
    let media_elements: Vec<_> = status.media.iter().map(|video| {
        let preview = video.preview_url.as_ref().cloned().unwrap_or_default();
        let video_url = video.video_url.clone();
        let video_url_for_button = video.video_url.clone(); // Clone for the button click
        let description = video.description.clone();
        rsx! {
            div {
                class: "enable-pointer-events",
                video {
                    class: "video-media-player",
                    controls: "true",
                    poster: "{preview}",
                    source {
                        src: "{video_url}"
                    }
                }
                div {
                    class: "flex flex-row relative justify-center mt-2",
                    IconTextButton {
                        icon: crate::icons::ICON_OPEN_WINDOW,
                        text: "Open in Window",
                        title: "Open in Window",
                        class: "mb-4",
                        disabled: false,
                        onclick: move |_| onclick.call(StatusAction::OpenVideo(video_url_for_button.clone())),
                    },
                }
                Paragraph {
                    style: TextStyle::Tertiary,
                    class: "p-3",
                    "{description}"
                }
            }
        }
    }).collect();

    rsx! {
        // Status images
        {image_elements.into_iter()}

        // Media (videos)
        {media_elements.into_iter()}

        // Card preview with proper ownership to avoid temporary value issues
        if let Some(card) = &status.card {
            {
                let mut desc = card.description.clone();
                if desc.len() > 300 {
                    desc = desc.chars().take(300).collect();
                    desc.push('…');
                }
                let card_url = card.url.clone(); // Clone for first closure
                let card_url_for_label = card.url.clone(); // Clone for second closure
                let card_title = card.title.clone(); // Clone the title
                let card_image = card.image.clone(); // Clone the image option
                rsx! {
                    div {
                        class: "link-object",
                        onclick: move |_| onclick.call(StatusAction::OpenLink(card_url.clone())),
                        if let Some(image_url) = &card_image {
                            img {
                                src: "{image_url}",
                            }
                        }
                        VStack {
                            class: "mr-auto gap-1",
                            Label {
                                style: TextStyle::Primary,
                                onclick: move |_| onclick.call(StatusAction::OpenLink(card_url_for_label.clone())),
                                pointer_style: PointerStyle::Pointer,
                                "{card_title}"
                            }
                            Paragraph {
                                style: TextStyle::Secondary,
                                pointer_style: PointerStyle::Pointer,
                                "{desc}"
                            }
                        }
                    }
                }
            }
        }
    }
}
