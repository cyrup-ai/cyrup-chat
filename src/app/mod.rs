pub mod context;
pub mod errors;
pub mod icons;
pub mod oauth;
pub mod reducer;
pub mod views;

pub use context::*;
pub use errors::*;
pub use icons::*;
pub use oauth::*;
pub use views::*;

/// Main application entry point
pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize vault first
    tokio::runtime::Runtime::new()?.block_on(async { crate::auth::initialize_vault().await })?;

    // Initialize OAuth system after vault is ready
    crate::app::oauth::initialize_oauth_system()?;

    // Modern Dioxus application launcher with proper window configuration
    #[cfg(target_os = "macos")]
    {
        use crate::environment::Environment;
        let window = Environment::macos_window();
        dioxus::LaunchBuilder::desktop()
            .with_cfg(dioxus_desktop::Config::new().with_window(window))
            .launch(crate::app::views::App);
    }

    #[cfg(not(target_os = "macos"))]
    {
        dioxus::launch(crate::app::views::App);
    }

    Ok(())
}
