//! Status Archive Manager with SurrealDB time-series integration
//!
//! Zero-copy status serialization and efficient archive management
//! using SurrealDB time-series tables for optimal performance.

use crate::environment::model::Status;
use crate::errors::ui::UiError;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use surrealdb::{Connection, Surreal};
use surrealdb_types::SurrealValue;

/// Archive metadata for efficient tracking
#[derive(Debug, Clone, Serialize, Deserialize, SurrealValue)]
pub struct ArchiveMetadata {
    /// Archive entry ID
    pub id: String,
    /// Original status ID
    pub status_id: String,
    /// Archive timestamp
    pub archived_at: DateTime<Utc>,
    /// Archive reason/category
    pub archive_reason: ArchiveReason,
    /// Compression ratio achieved
    pub compression_ratio: f32,
    /// Storage size in bytes
    pub storage_size: u64,
    /// Archive tags for categorization
    pub tags: Vec<String>,
    /// User who archived the status
    pub archived_by: String,
}

/// Reasons for archiving statuses
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, SurrealValue)]
pub enum ArchiveReason {
    /// User manually archived
    Manual,
    /// Automatic archiving based on age
    Automatic,
    /// Content moderation
    Moderation,
    /// Privacy/GDPR compliance
    Privacy,
    /// Storage optimization
    Storage,
}

/// Archive search criteria for efficient queries
#[derive(Debug, Clone)]
pub struct ArchiveSearchCriteria {
    /// Date range filter
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    /// Archive reason filter
    pub reason: Option<ArchiveReason>,
    /// Tag filters
    pub tags: Vec<String>,
    /// User filter
    pub archived_by: Option<String>,
    /// Full-text search in archived content
    pub content_search: Option<String>,
    /// Limit results
    pub limit: Option<usize>,
    /// Offset for pagination
    pub offset: Option<usize>,
}

/// Archive statistics for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArchiveStatistics {
    /// Total archived items
    pub total_archived: u64,
    /// Total storage used in bytes
    pub total_storage_bytes: u64,
    /// Average compression ratio
    pub avg_compression_ratio: f32,
    /// Archives by reason breakdown
    pub by_reason: HashMap<ArchiveReason, u64>,
    /// Archives by month for trending
    pub by_month: HashMap<String, u64>,
    /// Most active archivers
    pub top_archivers: Vec<(String, u64)>,
}

/// Error types for archive operations
#[derive(Debug, thiserror::Error)]
pub enum ArchiveError {
    #[error("Database connection error: {0}")]
    DatabaseError(String),
    #[error("Status not found: {0}")]
    StatusNotFound(String),
    #[error("Archive not found: {0}")]
    ArchiveNotFound(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Compression error: {0}")]
    CompressionError(String),
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    #[error("Invalid search criteria: {0}")]
    InvalidCriteria(String),
}

impl From<ArchiveError> for UiError {
    fn from(error: ArchiveError) -> Self {
        UiError::platform_error(error.to_string())
    }
}

/// Status archive manager trait with zero-copy operations
#[async_trait]
pub trait StatusArchiveManager {
    /// Archive a status with metadata
    async fn archive_status(
        &self,
        status: &Status,
        reason: ArchiveReason,
        archived_by: String,
    ) -> Result<ArchiveMetadata, ArchiveError>;

    /// Retrieve archived status by ID
    async fn retrieve_archived_status(
        &self,
        archive_id: &str,
    ) -> Result<Option<Status>, ArchiveError>;

    /// Search archived statuses with criteria
    async fn search_archives(
        &self,
        criteria: ArchiveSearchCriteria,
    ) -> Result<Vec<ArchiveMetadata>, ArchiveError>;

    /// Delete archived status permanently
    async fn delete_archive(&self, archive_id: &str) -> Result<bool, ArchiveError>;

    /// Get archive statistics
    async fn get_statistics(&self) -> Result<ArchiveStatistics, ArchiveError>;

    /// Export archives in bulk
    async fn export_archives(
        &self,
        criteria: ArchiveSearchCriteria,
    ) -> Result<Vec<(ArchiveMetadata, Status)>, ArchiveError>;

    /// Cleanup old archives based on retention policy
    async fn cleanup_old_archives(&self, retention_days: u32) -> Result<u64, ArchiveError>;
}

/// SurrealDB implementation of status archive manager
pub struct SurrealArchiveManager<C: Connection> {
    db: Surreal<C>,
    namespace: String,
    database: String,
    compression_enabled: bool,
}

impl<C: Connection> SurrealArchiveManager<C> {
    /// Create new archive manager with SurrealDB connection
    pub fn new(db: Surreal<C>, namespace: String, database: String) -> Self {
        Self {
            db,
            namespace,
            database,
            compression_enabled: true,
        }
    }

    /// Initialize time-series schema for archives
    pub async fn initialize_schema(&self) -> Result<(), ArchiveError> {
        // Use the namespace and database
        self.db
            .use_ns(&self.namespace)
            .use_db(&self.database)
            .await
            .map_err(|e| ArchiveError::DatabaseError(e.to_string()))?;

        // Create time-series tables for efficient archive storage
        let schema_query = r"
            DEFINE TABLE status_archives SCHEMAFULL TYPE NORMAL;
            DEFINE FIELD id ON TABLE status_archives TYPE string;
            DEFINE FIELD status_id ON TABLE status_archives TYPE string;
            DEFINE FIELD archived_at ON TABLE status_archives TYPE datetime;
            DEFINE FIELD archive_reason ON TABLE status_archives TYPE string;
            DEFINE FIELD compression_ratio ON TABLE status_archives TYPE float;
            DEFINE FIELD storage_size ON TABLE status_archives TYPE int;
            DEFINE FIELD tags ON TABLE status_archives TYPE array<string>;
            DEFINE FIELD archived_by ON TABLE status_archives TYPE string;
            DEFINE FIELD compressed_data ON TABLE status_archives TYPE bytes;
            
            DEFINE INDEX idx_archive_status ON TABLE status_archives COLUMNS status_id;
            DEFINE INDEX idx_archive_date ON TABLE status_archives COLUMNS archived_at;
            DEFINE INDEX idx_archive_reason ON TABLE status_archives COLUMNS archive_reason;
            DEFINE INDEX idx_archive_user ON TABLE status_archives COLUMNS archived_by;
            DEFINE INDEX idx_archive_tags ON TABLE status_archives COLUMNS tags;
            
            DEFINE TABLE archive_metadata SCHEMAFULL TYPE NORMAL;
            DEFINE FIELD archive_id ON TABLE archive_metadata TYPE string;
            DEFINE FIELD original_size ON TABLE archive_metadata TYPE int;
            DEFINE FIELD compressed_size ON TABLE archive_metadata TYPE int;
            DEFINE FIELD checksum ON TABLE archive_metadata TYPE string;
            DEFINE FIELD content_hash ON TABLE archive_metadata TYPE string;
        ";

        self.db
            .query(schema_query)
            .await
            .map_err(|e| ArchiveError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    /// Serialize status with zero-copy patterns where possible
    fn serialize_status(&self, status: &Status) -> Result<Vec<u8>, ArchiveError> {
        // Use JSON serialization for compatibility
        serde_json::to_vec(status).map_err(|e| ArchiveError::SerializationError(e.to_string()))
    }

    /// Deserialize status from binary data
    fn deserialize_status(&self, data: &[u8]) -> Result<Status, ArchiveError> {
        serde_json::from_slice(data).map_err(|e| ArchiveError::SerializationError(e.to_string()))
    }

    /// Compress serialized data if enabled
    fn compress_data(&self, data: &[u8]) -> Result<(Vec<u8>, f32), ArchiveError> {
        if !self.compression_enabled {
            return Ok((data.to_vec(), 1.0));
        }

        // Use efficient compression algorithm
        let compressed = snap::raw::Encoder::new()
            .compress_vec(data)
            .map_err(|e| ArchiveError::CompressionError(e.to_string()))?;

        let ratio = data.len() as f32 / compressed.len() as f32;
        Ok((compressed, ratio))
    }

    /// Decompress archived data
    fn decompress_data(&self, compressed: &[u8]) -> Result<Vec<u8>, ArchiveError> {
        if !self.compression_enabled {
            return Ok(compressed.to_vec());
        }

        snap::raw::Decoder::new()
            .decompress_vec(compressed)
            .map_err(|e| ArchiveError::CompressionError(e.to_string()))
    }

    /// Generate unique archive ID
    fn generate_archive_id(&self, status_id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        status_id.hash(&mut hasher);
        Utc::now()
            .timestamp_nanos_opt()
            .unwrap_or(0)
            .hash(&mut hasher);

        format!("archive_{:x}", hasher.finish())
    }
}

#[async_trait]
impl<C: Connection + Send + Sync> StatusArchiveManager for SurrealArchiveManager<C> {
    async fn archive_status(
        &self,
        status: &Status,
        reason: ArchiveReason,
        archived_by: String,
    ) -> Result<ArchiveMetadata, ArchiveError> {
        // Serialize status with zero-copy patterns
        let serialized = self.serialize_status(status)?;
        let _original_size = serialized.len() as u64;

        // Compress data for storage efficiency
        let (compressed_data, compression_ratio) = self.compress_data(&serialized)?;
        let storage_size = compressed_data.len() as u64;

        // Generate archive metadata
        let archive_id = self.generate_archive_id(&status.id);
        let archived_at = Utc::now();

        let metadata = ArchiveMetadata {
            id: archive_id.clone(),
            status_id: status.id.clone(),
            archived_at,
            archive_reason: reason.clone(),
            compression_ratio,
            storage_size,
            tags: vec![], // Could be extracted from status content
            archived_by: archived_by.clone(),
        };

        // Store in SurrealDB time-series table
        let query = r"
            CREATE status_archives:$archive_id SET
                status_id = $status_id,
                archived_at = $archived_at,
                archive_reason = $archive_reason,
                compression_ratio = $compression_ratio,
                storage_size = $storage_size,
                tags = $tags,
                archived_by = $archived_by,
                compressed_data = $compressed_data
        ";

        self.db
            .query(query)
            .bind(("archive_id", archive_id.to_string()))
            .bind(("status_id", status.id.clone()))
            .bind(("archived_at", archived_at))
            .bind(("archive_reason", format!("{:?}", reason)))
            .bind(("compression_ratio", compression_ratio as f64))
            .bind(("storage_size", storage_size as i64))
            .bind(("tags", Vec::<String>::new()))
            .bind(("archived_by", archived_by))
            .bind(("compressed_data", compressed_data))
            .await
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;

        Ok(metadata)
    }

    async fn retrieve_archived_status(
        &self,
        archive_id: &str,
    ) -> Result<Option<Status>, ArchiveError> {
        let query = r"
            SELECT compressed_data FROM status_archives WHERE id = $archive_id LIMIT 1
        ";

        let mut response = self
            .db
            .query(query)
            .bind(("archive_id", archive_id.to_string()))
            .await
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;
        let results: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;

        if let Some(result) = results.first()
            && let Some(compressed_data) = result.get("compressed_data")
            && let Some(bytes) = compressed_data.as_str()
        {
            // Decode base64 bytes and decompress
            use base64::Engine;
            let compressed = base64::engine::general_purpose::STANDARD
                .decode(bytes)
                .map_err(|e| ArchiveError::SerializationError(e.to_string()))?;
            let decompressed = self.decompress_data(&compressed)?;
            let status = self.deserialize_status(&decompressed)?;
            return Ok(Some(status));
        }

        Ok(None)
    }

    async fn search_archives(
        &self,
        criteria: ArchiveSearchCriteria,
    ) -> Result<Vec<ArchiveMetadata>, ArchiveError> {
        let mut query = "SELECT * FROM status_archives WHERE 1=1".to_string();

        // Build dynamic query based on criteria
        let date_range = criteria.date_range;
        let reason = criteria.reason;
        let user = criteria.archived_by;

        if date_range.is_some() {
            query.push_str(" AND archived_at >= $start_date AND archived_at <= $end_date");
        }

        if reason.is_some() {
            query.push_str(" AND archive_reason = $reason");
        }

        if user.is_some() {
            query.push_str(" AND archived_by = $user");
        }

        query.push_str(" ORDER BY archived_at DESC");

        if let Some(limit) = criteria.limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        // Build query with bindings
        let mut query_builder = self.db.query(&query);

        if let Some((start, end)) = date_range {
            query_builder = query_builder
                .bind(("start_date", start))
                .bind(("end_date", end));
        }

        if let Some(reason) = reason {
            query_builder = query_builder.bind(("reason", format!("{:?}", reason)));
        }

        if let Some(user) = user {
            query_builder = query_builder.bind(("user", user));
        }

        let mut response = query_builder
            .await
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;
        let results: Vec<ArchiveMetadata> = response
            .take(0)
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;

        Ok(results)
    }

    async fn delete_archive(&self, archive_id: &str) -> Result<bool, ArchiveError> {
        let query = r"
            DELETE status_archives WHERE id = $archive_id
        ";

        let mut response = self
            .db
            .query(query)
            .bind(("archive_id", archive_id.to_string()))
            .await
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;
        let deleted: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;

        Ok(!deleted.is_empty())
    }

    async fn get_statistics(&self) -> Result<ArchiveStatistics, ArchiveError> {
        // Query 1: Basic statistics
        let stats_query = r"
            SELECT
                count() as total_archived,
                math::sum(storage_size) as total_storage_bytes,
                math::mean(compression_ratio) as avg_compression_ratio
            FROM status_archives
        ";

        let mut stats_response = self
            .db
            .query(stats_query)
            .await
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;
        let stats_results: Vec<serde_json::Value> = stats_response
            .take(0)
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;

        let (total_archived, total_storage_bytes, avg_compression_ratio) =
            if let Some(result) = stats_results.first() {
                (
                    result
                        .get("total_archived")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    result
                        .get("total_storage_bytes")
                        .and_then(|v| v.as_u64())
                        .unwrap_or(0),
                    result
                        .get("avg_compression_ratio")
                        .and_then(|v| v.as_f64())
                        .unwrap_or(1.0) as f32,
                )
            } else {
                (0, 0, 1.0)
            };

        // Query 2: Group by archive_reason
        let reason_query = r"
            SELECT 
                archive_reason,
                count() as count
            FROM status_archives 
            GROUP BY archive_reason
        ";

        let mut reason_response = self
            .db
            .query(reason_query)
            .await
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;
        let reason_results: Vec<serde_json::Value> = reason_response
            .take(0)
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;

        let mut by_reason = HashMap::new();
        for result in reason_results {
            if let (Some(reason_str), Some(count)) = (
                result.get("archive_reason").and_then(|v| v.as_str()),
                result.get("count").and_then(|v| v.as_u64()),
            ) {
                let reason = match reason_str {
                    "Manual" => ArchiveReason::Manual,
                    "Automatic" => ArchiveReason::Automatic,
                    "Moderation" => ArchiveReason::Moderation,
                    "Privacy" => ArchiveReason::Privacy,
                    "Storage" => ArchiveReason::Storage,
                    _ => continue,
                };
                by_reason.insert(reason, count);
            }
        }

        // Query 3: Group by month
        let month_query = r"
            SELECT 
                time::format(archived_at, '%Y-%m') as month,
                count() as count
            FROM status_archives 
            GROUP BY month
            ORDER BY month DESC
            LIMIT 12
        ";

        let mut month_response = self
            .db
            .query(month_query)
            .await
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;
        let month_results: Vec<serde_json::Value> = month_response
            .take(0)
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;

        let mut by_month = HashMap::new();
        for result in month_results {
            if let (Some(month), Some(count)) = (
                result.get("month").and_then(|v| v.as_str()),
                result.get("count").and_then(|v| v.as_u64()),
            ) {
                by_month.insert(month.to_string(), count);
            }
        }

        // Query 4: Top archivers
        let archivers_query = r"
            SELECT 
                archived_by,
                count() as count
            FROM status_archives 
            GROUP BY archived_by
            ORDER BY count DESC
            LIMIT 10
        ";

        let mut archivers_response = self
            .db
            .query(archivers_query)
            .await
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;
        let archivers_results: Vec<serde_json::Value> = archivers_response
            .take(0)
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;

        let mut top_archivers = Vec::new();
        for result in archivers_results {
            if let (Some(user), Some(count)) = (
                result.get("archived_by").and_then(|v| v.as_str()),
                result.get("count").and_then(|v| v.as_u64()),
            ) {
                top_archivers.push((user.to_string(), count));
            }
        }

        Ok(ArchiveStatistics {
            total_archived,
            total_storage_bytes,
            avg_compression_ratio,
            by_reason,
            by_month,
            top_archivers,
        })
    }

    async fn export_archives(
        &self,
        criteria: ArchiveSearchCriteria,
    ) -> Result<Vec<(ArchiveMetadata, Status)>, ArchiveError> {
        let metadata_list = self.search_archives(criteria).await?;
        let mut exports: Vec<(ArchiveMetadata, Status)> = Vec::with_capacity(metadata_list.len());

        for metadata in metadata_list {
            if let Some(status) = self.retrieve_archived_status(&metadata.id).await? {
                exports.push((metadata, status));
            }
        }

        Ok(exports)
    }

    async fn cleanup_old_archives(&self, retention_days: u32) -> Result<u64, ArchiveError> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);

        let query = r"
            DELETE status_archives WHERE archived_at < $cutoff_date
        ";

        let mut response = self
            .db
            .query(query)
            .bind(("cutoff_date", cutoff_date))
            .await
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;
        let deleted: Vec<serde_json::Value> = response
            .take(0)
            .map_err(|e| ArchiveError::QueryFailed(e.to_string()))?;

        Ok(deleted.len() as u64)
    }
}

/// Archive manager factory with async-task integration
pub struct ArchiveManagerFactory;

impl ArchiveManagerFactory {
    /// Create archive manager with SurrealDB connection
    pub async fn create_manager<C: Connection + Send + Sync + 'static>(
        db: Surreal<C>,
        namespace: String,
        database: String,
    ) -> Result<SurrealArchiveManager<C>, ArchiveError> {
        let manager = SurrealArchiveManager::new(db, namespace, database);

        // Initialize schema
        manager.initialize_schema().await?;

        Ok(manager)
    }

    /// Create manager with async-task spawning
    pub fn spawn_manager_creation<C: Connection + Send + Sync + 'static>(
        db: Surreal<C>,
        namespace: String,
        database: String,
    ) {
        let (runnable, _task) = async_task::spawn(
            Self::create_manager(db, namespace, database),
            |runnable: async_task::Runnable| {
                runnable.run();
            },
        );
        runnable.run();
    }
}
