//! Status mutation handlers module - organized for maintainability

pub mod core;
pub mod failure;
pub mod success;

// Re-export main API
pub use core::handle_status_mutation_result;
#[allow(unused_imports)] // Used by core handler for mutation failure processing
pub use failure::handle_failed_mutation;
#[allow(unused_imports)] // Used by core handler for mutation success processing
pub use success::handle_successful_mutation;
