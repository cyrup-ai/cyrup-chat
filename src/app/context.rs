//! Environment context management
//!
//! This module provides context access functions for the Environment,
//! enabling components to access shared application state and services.
//!
//! # Architecture
//!
//! The context system provides three access patterns:
//! - `try_use_environment()` - Returns Result for explicit error handling
//! - `use_environment_or_default()` - Provides fallback for resilient components  
//! - `use_environment()` - Backwards compatible with panic on missing context
//!
//! # Usage
//!
//! ```rust,no_run
//! use crate::app::context::try_use_environment;
//!
//! // Recommended: Explicit error handling
//! match try_use_environment() {
//!     Ok(env) => { /* use environment */ },
//!     Err(e) => { /* handle missing context */ }
//! }
//!
//! // Alternative: Resilient components
//! let env = use_environment_or_default();
//! ```

use crate::environment::Environment;
use dioxus::prelude::*;

/// Error type for Environment context access failures
#[derive(Debug)]
pub enum EnvironmentContextError {
    /// Environment context was not found in the component tree
    ContextNotFound,
}

impl std::fmt::Display for EnvironmentContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ContextNotFound => write!(
                f,
                "Environment context not found in component tree. Ensure App component provides Environment context."
            ),
        }
    }
}

impl std::error::Error for EnvironmentContextError {}

/// Get Environment from context with explicit error handling
///
/// Returns Result to enable graceful error handling based on runtime configuration.
/// Callers should handle the error case appropriately for their use case.
///
/// # Returns
/// - `Ok(Signal<Environment>)` - Environment context found and accessible
/// - `Err(EnvironmentContextError)` - Context not found in component tree
///
/// # Examples
/// ```rust,no_run
/// match try_use_environment() {
///     Ok(env) => {
///         // Use environment for operations
///         env.with(|e| e.model.timeline());
///     },
///     Err(_) => {
///         // Handle missing context gracefully
///         log::warn!("Environment context unavailable");
///     }
/// }
/// ```
pub fn try_use_environment() -> Result<Signal<Environment>, EnvironmentContextError> {
    try_use_context::<Signal<Environment>>().ok_or(EnvironmentContextError::ContextNotFound)
}

/// Helper function that provides a default Environment if context is not found
/// Use this for components that need to continue functioning even without proper context
pub fn use_environment_or_default() -> Signal<Environment> {
    match try_use_environment() {
        Ok(env) => env,
        Err(e) => {
            log::error!("Environment context error: {e}. Cannot create Environment without async.");
            panic!(
                "CRITICAL: Environment context missing - application cannot function without it. Error: {e}"
            );
        }
    }
}

/// Backwards compatibility function - use try_use_environment() for new code
/// This maintains existing behavior while providing better error context
pub fn use_environment() -> Signal<Environment> {
    let config = crate::config::use_app_config();

    match try_use_environment() {
        Ok(env) => env,
        Err(e) => {
            if config.read().use_verbose_errors() {
                log::error!("CRITICAL: Environment context missing - {e}");
                log::error!("This indicates a programming error in component hierarchy setup");
            } else {
                log::warn!("Environment context not found, using default");
            }

            // Environment context must be provided by parent component
            panic!("Environment context missing - check component hierarchy setup");
        }
    }
}
