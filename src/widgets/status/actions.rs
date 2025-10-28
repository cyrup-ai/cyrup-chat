//! Status action definitions and types

#[derive(Clone, Eq, PartialEq, Debug)]
pub enum StatusAction {
    Clicked, // e.g. open conversation
    OpenTag(String),
    OpenLink(String),
    OpenAccount(String),
    Reply,
    Boost(bool),
    Favorite(bool),
    Bookmark(bool),
    OpenImage(String),
    OpenVideo(String),
    Copy(String),
}
