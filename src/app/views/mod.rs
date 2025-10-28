//! UI view components for authentication and main application
//!
//! This module contains all the UI view components including login,
//! loading, error, and main application views.

mod app_logic;
mod auth_components;
mod main_components;

pub use app_logic::App;
pub use auth_components::{AuthStatus, ErrorView, LoadingView, LoginView};
pub use main_components::{ChatHistorySidebar, MainView};
