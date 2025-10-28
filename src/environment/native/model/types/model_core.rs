//! Core Model struct for agent chat operations

use super::super::agent_manager::ModelAgentManager;
use super::super::archive_manager::{ArchiveManagerFactory, SurrealArchiveManager};
use super::error_types::ModelError;
use crate::database::Database;
use crate::environment::types::Instance;
use std::sync::Arc;
use surrealdb::engine::local::Db;

/// Core Model struct for agent conversation management
#[derive(Clone)]
pub struct Model {
    /// Agent manager for spawning and messaging agent sessions
    agent_manager: Arc<ModelAgentManager>,

    /// Database connection for conversation/message operations
    db: Arc<Database>,

    /// Archive manager for status archiving with compression
    archive_manager: Arc<SurrealArchiveManager<Db>>,

    /// Application instance metadata (static configuration)
    instance: Instance,
}

impl Model {
    /// Create a new Model with database connection
    ///
    /// # Arguments
    /// * `db` - Shared database connection
    ///
    /// # Returns
    /// * `Ok(Model)` - Initialized model with agent manager
    /// * `Err(ModelError)` - Database initialization failed
    ///
    /// # Example
    /// ```rust
    /// let db = Arc::new(Database::new().await?);
    /// let model = Model::new(db).await?;
    /// ```
    pub async fn new(db: Arc<Database>) -> Result<Self, ModelError> {
        // Create agent manager with database reference
        let agent_manager = Arc::new(ModelAgentManager::new(Arc::clone(&db)));

        // Create archive manager with database connection
        let archive_manager = Arc::new(
            ArchiveManagerFactory::create_manager(
                db.client().clone(),
                "cyrup".to_string(),
                "chat".to_string(),
            )
            .await
            .map_err(|e| {
                ModelError::QueryFailed(format!("Failed to initialize archive manager: {}", e))
            })?,
        );

        // Initialize instance metadata from cargo package info
        let instance = Instance {
            id: env!("CARGO_PKG_NAME").to_string(),
            name: "CYRUP Chat".to_string(),
            users: "1".to_string(), // Single-user application
            thumbnail: None,
        };

        Ok(Self {
            agent_manager,
            db,
            archive_manager,
            instance,
        })
    }

    /// Get reference to agent manager for advanced operations
    pub fn agent_manager(&self) -> &Arc<ModelAgentManager> {
        &self.agent_manager
    }

    /// Get reference to database for direct queries
    pub fn database(&self) -> &Arc<Database> {
        &self.db
    }

    /// Get reference to archive manager for archiving operations
    pub fn archive_manager(&self) -> &Arc<SurrealArchiveManager<Db>> {
        &self.archive_manager
    }

    /// Get application instance metadata
    ///
    /// Returns static instance information about this CYRUP Chat installation.
    /// This is a local application identity, not a remote server.
    ///
    /// # Returns
    /// Reference to cached Instance struct with application metadata
    pub fn instance(&self) -> &Instance {
        &self.instance
    }
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model").finish()
    }
}
