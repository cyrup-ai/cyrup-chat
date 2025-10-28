//! Post reducer components for post composition and state management

mod action_handlers;
mod event_processing;
mod validation;

pub use action_handlers::{PostSignal, handle_post_action};
// pub use event_processing::handle_menu_event_with_environment; // Currently unused - enable when needed
