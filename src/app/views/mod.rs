//! UI view components for authentication and main application
//!
//! This module contains all the UI view components including login,
//! loading, error, and main application views.

mod app_logic;
mod auth_components;
mod main_components;
mod timeline_view;
mod notifications_view;
mod rooms_view;
mod more_view;

pub use app_logic::App;
pub use auth_components::{AuthStatus, ErrorView, LoadingView, LoginView};
pub use main_components::{ChatHistorySidebar, MainView};
pub use timeline_view::TimelineView;
pub use notifications_view::NotificationsView;
pub use rooms_view::RoomsView;
pub use more_view::MoreView;
