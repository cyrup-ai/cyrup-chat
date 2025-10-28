//! Root application state

use crate::app::views::AuthStatus; // ← REUSE existing type
use crate::environment::Environment;

#[derive(Clone, PartialEq, Debug)]
pub enum AppStatus {
    Initializing,
    EnvironmentReady(Environment),
    EnvironmentError(String),
}

/// Root application state - single source of truth
#[derive(Clone, PartialEq, Debug)]
pub struct AppState {
    pub app_status: AppStatus,
    pub auth_status: AuthStatus, // ← Using existing type
    pub error: Option<String>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            app_status: AppStatus::Initializing,
            auth_status: AuthStatus::Loading,
            error: None,
        }
    }
}
