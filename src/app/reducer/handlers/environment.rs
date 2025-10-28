//! Environment initialization handlers

use crate::app::reducer::state::{AppState, AppStatus};
use crate::database::Database;
use crate::environment::{
    Environment,
    native::{Model, Settings},
};
use dioxus::prelude::*;
use std::sync::Arc;

pub fn handle_initialize(signal: Signal<AppState>) -> Result<(), String> {
    spawn(async move {
        let result = initialize_environment().await;
        let _ = handle_ready(signal, result);
    });
    Ok(())
}

async fn initialize_environment() -> Result<Environment, String> {
    log::info!("Initializing database and environment...");

    // REUSE existing initialization code from app_logic.rs
    let database = Arc::new(Database::new().await.map_err(|e| {
        log::error!("Database initialization failed: {}", e);
        format!("Database error: {}", e)
    })?);

    crate::database::init_schema(database.client())
        .await
        .map_err(|e| {
            log::error!("Schema initialization failed: {}", e);
            format!("Schema error: {}", e)
        })?;

    let model = Model::new(Arc::clone(&database)).await.map_err(|e| {
        log::error!("Model creation failed: {}", e);
        format!("Model error: {}", e)
    })?;

    let settings = Settings::new().await;
    let env = Environment::new(database, model, settings);

    log::info!("Environment initialized successfully");
    Ok(env)
}

pub fn handle_ready(
    mut signal: Signal<AppState>,
    result: Result<Environment, String>,
) -> Result<(), String> {
    signal.with_mut(|state| {
        state.app_status = match result {
            Ok(env) => AppStatus::EnvironmentReady(env),
            Err(error) => AppStatus::EnvironmentError(error),
        };
    });
    Ok(())
}
