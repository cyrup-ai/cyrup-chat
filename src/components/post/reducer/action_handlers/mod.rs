//! Post action handlers organized by logical concerns

pub mod content;
pub mod dispatcher;
pub mod files;
pub mod images;
pub mod lifecycle;

// Re-export main public API
pub use dispatcher::{PostSignal, handle_post_action};
