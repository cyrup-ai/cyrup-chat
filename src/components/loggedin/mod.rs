pub mod reducer;
pub mod view;
pub mod view_simple;

use reducer::ReducerState;

pub use reducer::{Action, handle_action};

// Modern Dioxus signal-based state management
pub type RootSignal = dioxus::prelude::Signal<ReducerState>;
