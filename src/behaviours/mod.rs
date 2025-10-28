use dioxus::prelude::*;

#[derive(Clone, Debug)]
pub struct WindowContext {
    pub zoom_level: f64,
    pub is_fullscreen: bool,
}

impl Default for WindowContext {
    fn default() -> Self {
        Self {
            zoom_level: 1.0,
            is_fullscreen: false,
        }
    }
}

pub static WINDOW_CONTEXT: GlobalSignal<WindowContext> = Signal::global(WindowContext::default);

mod change_textsize;
pub use change_textsize::ChangeTextsizeBehaviour;

pub trait Behaviour {
    type InputAction;
    type Environment;
    type InputState;
    fn setup(environment: &Self::Environment);
    fn handle(
        action: Self::InputAction,
        state: &mut Self::InputState,
        environment: &Self::Environment,
    );
}
