pub mod menu;
pub mod storage;
pub mod types;

pub mod native;
pub use native::*;

use dioxus::prelude::Element;
use std::rc::Rc;

use self::types::AppEvent;

pub trait OpenWindowState: Clone + PartialEq + 'static {
    type Action: 'static;
    fn window(
        &self,
        environment: &Environment,
        receiver: flume::Receiver<AppEvent>,
        parent_handler: Rc<dyn Fn(Self::Action)>,
    ) -> Element;
}

pub trait UploadMediaExt {
    fn id(&self) -> &str;
}

impl UploadMediaExt for megalodon::entities::UploadMedia {
    fn id(&self) -> &str {
        match self {
            megalodon::entities::UploadMedia::Attachment(a) => &a.id,
            megalodon::entities::UploadMedia::AsyncAttachment(a) => &a.id,
        }
    }
}
