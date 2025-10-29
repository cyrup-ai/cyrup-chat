mod reducer;
#[allow(dead_code)] // More menu component system - pending UI integration
mod view;

pub use reducer::{State, Action, handle_action};
pub use view::MoreViewComponent;

pub struct MoreReducer;
// Modern Dioxus signal-based state management
#[allow(dead_code)] // More signal type alias - pending integration
pub type MoreSignal = dioxus::prelude::Signal<reducer::State>;

// Modern Signal-based reducer pattern - integrates with zero-allocation state management
impl MoreReducer {
    #[allow(dead_code)] // Modern Signal-based constructor - integrates with reactive state management
    pub fn new() -> Self {
        Self
    }
}
