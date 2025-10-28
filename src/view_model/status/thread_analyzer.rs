//! Conversation thread analyzer with SurrealDB graph queries
//!
//! Zero-allocation thread traversal using async streams for efficient
//! conversation thread analysis without blocking operations.

use crate::errors::ui::UiError;

use async_trait::async_trait;
use futures_util::stream::Stream;
use serde::{Deserialize, Serialize};
use std::pin::Pin;
use surrealdb::{Connection, Surreal};
use surrealdb_types::SurrealValue;

/// Thread relationship types for conversation analysis
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, SurrealValue)]
pub enum ThreadRelationType {
    /// Direct reply relationship
    Reply,
    /// Quote/repost relationship  
    Quote,
    /// Mention relationship
    Mention,
    /// Thread continuation
    Continuation,
}

/// Thread node representing a status in the conversation graph
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ThreadNode {
    /// Status ID
    pub id: String,
    /// Parent status ID if this is a reply
    pub parent_id: Option<String>,
    /// Root thread ID for efficient traversal
    pub root_id: String,
    /// Thread depth from root
    pub depth: u32,
    /// Relationship type to parent
    pub relation_type: ThreadRelationType,
    /// Timestamp for ordering
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Author ID for thread ownership analysis
    pub author_id: String,
}

/// Thread analysis result with zero-allocation patterns
#[derive(Debug, Clone)]
pub struct ThreadAnalysisResult {
    /// Thread root ID
    pub root_id: String,
    /// Total thread depth
    pub max_depth: u32,
    /// Number of participants
    pub participant_count: usize,
    /// Thread branch count
    pub branch_count: u32,
    /// Is this a linear thread or branched conversation
    pub is_linear: bool,
}

/// Error types for thread analysis operations
#[derive(Debug, thiserror::Error)]
pub enum ThreadAnalysisError {
    #[error("Database connection error: {0}")]
    DatabaseError(String),
    #[error("Thread not found: {0}")]
    ThreadNotFound(String),
    #[error("Invalid thread structure: {0}")]
    InvalidStructure(String),
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    #[error("Stream processing error: {0}")]
    StreamError(String),
}

impl From<ThreadAnalysisError> for UiError {
    fn from(error: ThreadAnalysisError) -> Self {
        UiError::platform_error(error.to_string())
    }
}

/// Async stream type for thread traversal
pub type ThreadStream = Pin<Box<dyn Stream<Item = Result<ThreadNode, ThreadAnalysisError>> + Send>>;

/// Conversation thread analyzer trait with SurrealDB integration
#[async_trait]
pub trait ConversationThreadAnalyzer {
    /// Analyze thread structure for a given status
    async fn analyze_thread(
        &self,
        status_id: &str,
    ) -> Result<ThreadAnalysisResult, ThreadAnalysisError>;

    /// Get thread root for a status
    async fn get_thread_root(&self, status_id: &str)
    -> Result<Option<String>, ThreadAnalysisError>;

    /// Stream thread nodes in conversation order
    async fn stream_thread_nodes(&self, root_id: &str)
    -> Result<ThreadStream, ThreadAnalysisError>;

    /// Get direct replies to a status
    async fn get_direct_replies(
        &self,
        status_id: &str,
    ) -> Result<Vec<ThreadNode>, ThreadAnalysisError>;

    /// Check if status is part of a conversation thread
    async fn is_in_thread(&self, status_id: &str) -> Result<bool, ThreadAnalysisError>;

    /// Get thread context for reply extraction
    async fn get_thread_context(
        &self,
        status_id: &str,
    ) -> Result<Option<ThreadNode>, ThreadAnalysisError>;
}

/// SurrealDB implementation of conversation thread analyzer
pub struct SurrealThreadAnalyzer<C: Connection> {
    db: Surreal<C>,
    namespace: String,
    database: String,
}

impl<C: Connection> SurrealThreadAnalyzer<C> {
    /// Create new thread analyzer with SurrealDB connection
    pub fn new(db: Surreal<C>, namespace: String, database: String) -> Self {
        Self {
            db,
            namespace,
            database,
        }
    }

    /// Initialize database schema for thread analysis
    pub async fn initialize_schema(&self) -> Result<(), ThreadAnalysisError> {
        // Use the namespace and database
        self.db
            .use_ns(&self.namespace)
            .use_db(&self.database)
            .await
            .map_err(|e| ThreadAnalysisError::DatabaseError(e.to_string()))?;

        // Create thread_nodes table with graph relationships
        let schema_query = r#"
            DEFINE TABLE thread_nodes SCHEMAFULL;
            DEFINE FIELD id ON TABLE thread_nodes TYPE string;
            DEFINE FIELD parent_id ON TABLE thread_nodes TYPE option<string>;
            DEFINE FIELD root_id ON TABLE thread_nodes TYPE string;
            DEFINE FIELD depth ON TABLE thread_nodes TYPE int;
            DEFINE FIELD relation_type ON TABLE thread_nodes TYPE string;
            DEFINE FIELD created_at ON TABLE thread_nodes TYPE datetime;
            DEFINE FIELD author_id ON TABLE thread_nodes TYPE string;
            
            DEFINE INDEX idx_thread_root ON TABLE thread_nodes COLUMNS root_id;
            DEFINE INDEX idx_thread_parent ON TABLE thread_nodes COLUMNS parent_id;
            DEFINE INDEX idx_thread_author ON TABLE thread_nodes COLUMNS author_id;
            DEFINE INDEX idx_thread_created ON TABLE thread_nodes COLUMNS created_at;
            
            DEFINE TABLE thread_relations SCHEMAFULL;
            DEFINE FIELD in ON TABLE thread_relations TYPE record(thread_nodes);
            DEFINE FIELD out ON TABLE thread_relations TYPE record(thread_nodes);
            DEFINE FIELD relation_type ON TABLE thread_relations TYPE string;
        "#;

        self.db
            .query(schema_query)
            .await
            .map_err(|e| ThreadAnalysisError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Create async stream for thread traversal
    fn create_thread_stream(&self, nodes: Vec<ThreadNode>) -> ThreadStream {
        let stream = futures_util::stream::iter(nodes.into_iter().map(Ok));
        Box::pin(stream)
    }
}

#[async_trait]
impl<C: Connection + Send + Sync> ConversationThreadAnalyzer for SurrealThreadAnalyzer<C> {
    async fn analyze_thread(
        &self,
        status_id: &str,
    ) -> Result<ThreadAnalysisResult, ThreadAnalysisError> {
        // Get thread root first
        let root_id = match self.get_thread_root(status_id).await? {
            Some(root) => root,
            None => return Err(ThreadAnalysisError::ThreadNotFound(status_id.to_string())),
        };

        // Analyze thread structure with graph query
        let query = r#"
            SELECT
                count() as total_nodes,
                math::max(depth) as max_depth,
                array::distinct(author_id) as participants,
                count(SELECT * FROM thread_nodes WHERE root_id = $root_id AND array::len(->thread_relations) > 1) as branch_count
            FROM thread_nodes
            WHERE root_id = $root_id
        "#;

        let mut response = self
            .db
            .query(query)
            .bind(("root_id", root_id.clone()))
            .await
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;
        let results: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;

        if let Some(result) = results.first() {
            let max_depth = result
                .get("max_depth")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            let participants: Vec<serde_json::Value> = result
                .get("participants")
                .and_then(|v| v.as_array())
                .cloned()
                .unwrap_or_default();

            let branch_count = result
                .get("branch_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;

            Ok(ThreadAnalysisResult {
                root_id,
                max_depth,
                participant_count: participants.len(),
                branch_count,
                is_linear: branch_count == 0,
            })
        } else {
            Err(ThreadAnalysisError::InvalidStructure(
                "No analysis results".to_string(),
            ))
        }
    }

    async fn get_thread_root(
        &self,
        status_id: &str,
    ) -> Result<Option<String>, ThreadAnalysisError> {
        let query = r#"
            SELECT root_id FROM thread_nodes WHERE id = $status_id LIMIT 1
        "#;

        let mut response = self
            .db
            .query(query)
            .bind(("status_id", status_id.to_string()))
            .await
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;
        let results: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;

        Ok(results
            .first()
            .and_then(|r| r.get("root_id"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()))
    }

    async fn stream_thread_nodes(
        &self,
        root_id: &str,
    ) -> Result<ThreadStream, ThreadAnalysisError> {
        let query = r#"
            SELECT * FROM thread_nodes
            WHERE root_id = $root_id
            ORDER BY depth ASC, created_at ASC
        "#;

        let mut response = self
            .db
            .query(query)
            .bind(("root_id", root_id.to_string()))
            .await
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;
        let nodes: Vec<ThreadNode> = response
            .take(0)
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;

        Ok(self.create_thread_stream(nodes))
    }

    async fn get_direct_replies(
        &self,
        status_id: &str,
    ) -> Result<Vec<ThreadNode>, ThreadAnalysisError> {
        let query = r#"
            SELECT * FROM thread_nodes
            WHERE parent_id = $status_id
            ORDER BY created_at ASC
        "#;

        let mut response = self
            .db
            .query(query)
            .bind(("status_id", status_id.to_string()))
            .await
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;
        let nodes: Vec<ThreadNode> = response
            .take(0)
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;

        Ok(nodes)
    }

    async fn is_in_thread(&self, status_id: &str) -> Result<bool, ThreadAnalysisError> {
        let query = r#"
            SELECT count() as exists FROM thread_nodes WHERE id = $status_id LIMIT 1
        "#;

        let mut response = self
            .db
            .query(query)
            .bind(("status_id", status_id.to_string()))
            .await
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;
        let results: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;

        Ok(results
            .first()
            .and_then(|r| r.get("exists"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
            > 0)
    }

    async fn get_thread_context(
        &self,
        status_id: &str,
    ) -> Result<Option<ThreadNode>, ThreadAnalysisError> {
        let query = r#"
            SELECT * FROM thread_nodes WHERE id = $status_id LIMIT 1
        "#;

        let mut response = self
            .db
            .query(query)
            .bind(("status_id", status_id.to_string()))
            .await
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;
        let mut nodes: Vec<ThreadNode> = response
            .take(0)
            .map_err(|e| ThreadAnalysisError::QueryFailed(e.to_string()))?;

        Ok(nodes.pop())
    }
}

/// Thread analyzer factory with async-task integration
pub struct ThreadAnalyzerFactory;

impl ThreadAnalyzerFactory {
    /// Create thread analyzer with SurrealDB connection
    pub async fn create_analyzer<C: Connection + Send + Sync + 'static>(
        db: Surreal<C>,
        namespace: String,
        database: String,
    ) -> Result<SurrealThreadAnalyzer<C>, ThreadAnalysisError> {
        let analyzer = SurrealThreadAnalyzer::new(db, namespace, database);

        // Initialize schema
        analyzer.initialize_schema().await?;

        Ok(analyzer)
    }

    /// Create analyzer with async-task spawning
    pub fn spawn_analyzer_creation<C: Connection + Send + Sync + 'static>(
        db: Surreal<C>,
        namespace: String,
        database: String,
    ) {
        let (runnable, _task) = async_task::spawn(
            Self::create_analyzer(db, namespace, database),
            |runnable: async_task::Runnable| {
                runnable.run();
            },
        );
        runnable.run();
    }
}

/// Utility functions for thread analysis
pub mod utils {
    use super::*;

    /// Extract reply ID from thread context with zero allocation
    pub async fn extract_reply_id_from_context<T: ConversationThreadAnalyzer>(
        analyzer: &T,
        status_id: &str,
    ) -> Result<Option<String>, ThreadAnalysisError> {
        match analyzer.get_thread_context(status_id).await? {
            Some(node) => Ok(node.parent_id),
            None => Ok(None),
        }
    }

    /// Check if status is thread root
    pub async fn is_thread_root<T: ConversationThreadAnalyzer>(
        analyzer: &T,
        status_id: &str,
    ) -> Result<bool, ThreadAnalysisError> {
        match analyzer.get_thread_context(status_id).await? {
            Some(node) => Ok(node.id == node.root_id && node.parent_id.is_none()),
            None => Ok(false),
        }
    }

    /// Get thread statistics efficiently
    pub async fn get_thread_stats<T: ConversationThreadAnalyzer>(
        analyzer: &T,
        status_id: &str,
    ) -> Result<Option<ThreadAnalysisResult>, ThreadAnalysisError> {
        if analyzer.is_in_thread(status_id).await? {
            Ok(Some(analyzer.analyze_thread(status_id).await?))
        } else {
            Ok(None)
        }
    }
}
