//! Root application reducer

pub mod actions;
pub mod handlers;
pub mod state;

pub use actions::AppAction;
pub use handlers::handle_app_action;
pub use state::AppState;
