//! Batch processing module - organized for maintainability

pub mod network;
pub mod processor;
pub mod ui_updates;
pub mod utils;

// Re-export main API
pub use processor::handle_batch_mutations;
