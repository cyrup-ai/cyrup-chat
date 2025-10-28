//! Native environment module - organized for maintainability

pub mod core;
pub mod file_handling;
pub mod model;
pub mod platform;
pub mod settings;
mod toolbar;
pub mod window;

// Re-export main API
pub use core::Environment;
pub use file_handling::handle_file_event;
pub use model::Model;
pub use settings::Settings;
pub use window::{NewWindowPopup, NewWindowPopupProps};
