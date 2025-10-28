//! Status widget module - organized for maintainability

pub mod actions;
pub mod component;
pub mod media;

// Re-export main API
pub use actions::StatusAction;
pub use component::StatusComponent;
// pub use media::ContentCellMedia; // Currently unused - enable when needed
