//! Database layer using SurrealDB for agent chat storage
//!
//! Uses forked SurrealDB with SurrealKV embedded storage engine.
//! Database file: ~/.local/share/cyrup/chat.db (macOS: ~/Library/Application Support/cyrup/chat.db)

use surrealdb::{
    Surreal,
    engine::local::{Db, SurrealKv},
    opt::{capabilities::{Capabilities, ExperimentalFeature}, Config},
};
use surrealdb_types::SurrealValue;

// Module declarations for database operations (created in later tasks)
pub mod bookmarks;
pub mod conversations;
pub mod messages;
pub mod migration;
pub mod reactions;
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
        tokio::fs::create_dir_all(&data_dir)
            .await
            .map_err(|e| format!("Failed to create data directory: {}", e))?;

        let db_path = data_dir.join("chat.db");

        // Enable experimental features (record references) for schema constraints
        let capabilities = Capabilities::new()
            .with_experimental_feature_allowed(ExperimentalFeature::RecordReferences);
        let config = Config::new().capabilities(capabilities);

        // IMPORTANT: Use SurrealKv (not RocksDb) from forked SurrealDB
        let client = Surreal::new::<SurrealKv>((db_path, config))
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

        let db = Self {
            client,
            token_budget_config: crate::view_model::TokenBudgetConfig::default(),
        };

        // Initialize schema (safe to call multiple times)
        init_schema(db.client()).await?;

        // Auto-run migrations if needed
        db.auto_migrate().await?;

        Ok(db)
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

    /// Get current schema version (0 if no version set)
    async fn get_schema_version(&self) -> Result<i64, String> {
        let query = "SELECT version FROM schema_version ORDER BY applied_at DESC LIMIT 1";

        let mut response = self.client()
            .query(query)
            .await
            .map_err(|e| format!("Failed to get schema version: {}", e))?;

        #[derive(serde::Deserialize, surrealdb_types::SurrealValue)]
        struct VersionRecord {
            version: i64,
        }

        let versions: Vec<VersionRecord> = response
            .take(0)
            .unwrap_or_else(|_| Vec::new());

        Ok(versions.first().map(|v| v.version).unwrap_or(0))
    }

    /// Set schema version after successful migration
    async fn set_schema_version(&self, version: i64) -> Result<(), String> {
        let query = "CREATE schema_version CONTENT { version: $version }";

        self.client()
            .query(query)
            .bind(("version", version))
            .await
            .map_err(|e| format!("Failed to set schema version: {}", e))?;

        Ok(())
    }

    /// Auto-run migrations based on schema version
    async fn auto_migrate(&self) -> Result<(), String> {
        let current_version = self.get_schema_version().await.unwrap_or(0);

        log::info!("[Database] Current schema version: {}", current_version);

        // Migration 1: Unify conversations and rooms
        if current_version < 1 {
            log::info!("[Database] Running migration 1: Unify conversations/rooms");

            let (convos, rooms) = self.migrate_to_unified_conversations().await?;

            log::info!("[Database] Migrated {} conversations, {} rooms", convos, rooms);

            self.set_schema_version(1).await?;
        } else {
            log::info!("[Database] Schema up to date (version {})", current_version);
        }

        Ok(())
    }
}
