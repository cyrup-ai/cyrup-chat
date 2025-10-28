//! Summary card component for conversation context display
//!
//! Shows conversation summary and last update time at the top of
//! conversation view to provide transparency about agent context.

pub mod view;

// Re-export main component for convenience
#[allow(unused_imports)]
pub use view::SummaryCard;
