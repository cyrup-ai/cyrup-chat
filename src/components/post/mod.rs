mod action;
mod reducer;
mod state;
mod view;

pub use action::PostAction;
pub use reducer::{PostSignal, handle_post_action};
pub use state::{PostKind, State, Visibility};
pub use view::PostView;
