mod providers;
#[allow(dead_code)]
// Timeline system - comprehensive status timeline management pending UI integration
mod reducer;
mod view;

pub use providers::*;
pub use reducer::State;
#[allow(unused_imports)]
pub use view::{TimelineComponent, TimelineContents};

#[allow(dead_code)] // Timeline reducer - architectural scaffolding pending integration
pub struct TimelineReducer;

// Navicula imports removed - using modern Dioxus patterns

// TimelineSignal is already defined and exported from reducer.rs
