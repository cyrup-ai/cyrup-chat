//! Database layer using SurrealDB for agent chat storage
//!
//! Uses forked SurrealDB with SurrealKV embedded storage engine.
//! Database file: ~/.local/share/cyrup/chat.db (macOS: ~/Library/Application Support/cyrup/chat.db)

use surrealdb::{
    Surreal,
    engine::local::{Db, SurrealKv},
};

// Module declarations for database operations (created in later tasks)
pub mod bookmarks;
pub mod conversations;
pub mod messages;
pub mod reactions;
pub mod rooms;
pub mod templates;

// Re-export schema initialization
pub mod schema;
pub use schema::init_schema;

// Re-export token budget configuration
pub use crate::view_model::TokenBudgetConfig;

/// Database connection wrapper for SurrealKV embedded database
#[derive(Clone)]
pub struct Database {
    client: Surreal<Db>,
    token_budget_config: crate::view_model::TokenBudgetConfig,
}

impl Database {
    /// Create new database connection with SurrealKV engine
    ///
    /// # Platform-specific paths
    /// - macOS: `~/Library/Application Support/cyrup/chat.db`
    /// - Linux: `~/.local/share/cyrup/chat.db`
    /// - Windows: `%LOCALAPPDATA%\cyrup\chat.db`
    ///
    /// # Errors
    /// Returns error if:
    /// - Cannot determine data directory
    /// - Cannot create directory
    /// - Database connection fails
    /// - Authentication fails
    pub async fn new() -> Result<Self, String> {
        // Get platform-specific data directory using `dirs` crate
        let data_dir = dirs::data_local_dir()
            .ok_or("Could not determine data directory")?
            .join("cyrup");

        // Create directory if it doesn't exist
        std::fs::create_dir_all(&data_dir)
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = data_dir.join("chat.db");

        // IMPORTANT: Use SurrealKv (not RocksDb) from forked SurrealDB
        let client = Surreal::new::<SurrealKv>(db_path)
            .await
            .map_err(|e| format!("Database connection failed: {}", e))?;

        // NOTE: Embedded databases (SurrealKV, RocksDB, Mem) don't support authentication
        // Authentication is only for remote SurrealDB server connections
        // For local embedded databases, skip signin and go directly to namespace/database selection

        // Select namespace and database
        client
            .use_ns("cyrup")
            .use_db("chat")
            .await
            .map_err(|e| format!("Database selection failed: {}", e))?;

        Ok(Self {
            client,
            token_budget_config: crate::view_model::TokenBudgetConfig::default(),
        })
    }

    /// Get reference to SurrealDB client for direct queries
    pub fn client(&self) -> &Surreal<Db> {
        &self.client
    }

    /// Get reference to token budget configuration
    pub fn token_budget_config(&self) -> &crate::view_model::TokenBudgetConfig {
        &self.token_budget_config
    }

    /// Update token budget configuration
    ///
    /// # Arguments
    /// * `config` - New token budget configuration
    ///
    /// # Design Note
    /// Allows runtime tuning of token budget parameters without recompilation
    pub fn set_token_budget_config(&mut self, config: crate::view_model::TokenBudgetConfig) {
        self.token_budget_config = config;
    }
}
