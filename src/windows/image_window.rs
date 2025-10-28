#![allow(unused)]
use std::rc::Rc;

use crate::environment::{Environment, OpenWindowState, types::AppEvent};
use dioxus::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ImageWindowKind {
    Image,
    Video,
}

#[derive(Clone, PartialEq)]
pub struct ImageWindowState(pub String, pub ImageWindowKind);

impl OpenWindowState for ImageWindowState {
    type Action = ();

    fn window(
        &self,
        _environment: &Environment,
        _receiver: flume::Receiver<AppEvent>,
        _parent_handler: Rc<dyn Fn(Self::Action)>,
    ) -> Element {
        let url = self.0.clone();

        match self.1 {
            ImageWindowKind::Image => rsx! {
                div {
                    img {
                        style: "object-fit: contain; width: 100%; height: 100vh; object-position: center center;",
                        width: "100%",
                        height: "100%",
                        src: "{url}",
                    }
                }
            },
            ImageWindowKind::Video => rsx! {
                div {
                    video {
                        width: "100%",
                        height: "100%",
                        controls: true,
                        source {
                            src: "{url}"
                        }
                    }
                }
            },
        }
    }
}
