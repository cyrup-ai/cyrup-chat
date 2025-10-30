//! Agent template database operations
//!
//! Provides CRUD operations for agent_template table defined in src/database/schema.rs:24-36

use super::Database;
use crate::view_model::agent::AgentTemplate;
use surrealdb_types::{RecordId, ToSql};

impl Database {
    /// Create a new agent template in the database
    ///
    /// # Arguments
    /// * `template` - AgentTemplate to create (id field is ignored, DB generates new ID)
    ///
    /// # Returns
    /// * `Ok(String)` - Database-generated record ID (e.g., "agent_template:ulid")
    /// * `Err(String)` - Error message if creation fails
    ///
    /// # Database Operation
    /// Inserts into agent_template table with all fields from template struct.
    /// SurrealDB auto-generates ID and sets created_at to current time.
    ///
    /// # Example
    /// ```rust
    /// let db = Database::new().await?;
    /// let template = AgentTemplate {
    ///     name: "Code Assistant".to_string(),
    ///     model: AgentModel::Sonnet,
    ///     system_prompt: "You are a helpful coding assistant".to_string(),
    ///     max_turns: 50,
    ///     ..Default::default()
    /// };
    /// let id = db.create_template(&template).await?;
    /// println!("Created template with ID: {}", id);
    /// ```
    pub async fn create_template(&self, template: &AgentTemplate) -> Result<RecordId, String> {
        // Use .create() for type-safe insertion
        // Returns Option<T> with created record containing generated ID
        // Clone template to satisfy 'static lifetime requirement for async
        let result: Option<AgentTemplate> = self
            .client()
            .create("agent_template")
            .content(template.clone())
            .await
            .map_err(|e| format!("Failed to create template: {}", e))?;

        // Extract ID from created record
        result
            .map(|t| t.id.clone())
            .ok_or_else(|| "Create returned empty result".to_string())
    }

    /// Retrieve a single agent template by ID
    ///
    /// # Arguments
    /// * `id` - Template record ID (e.g., "ulid" or full "agent_template:ulid")
    ///
    /// # Returns
    /// * `Ok(AgentTemplate)` - Found template with all fields populated
    /// * `Err(String)` - Error if not found or query fails
    ///
    /// # Database Operation
    /// SELECT * FROM agent_template WHERE id = $id
    ///
    /// # Example
    /// ```rust
    /// let template = db.get_template("abc123").await?;
    /// println!("Template: {} using {}", template.name, template.model);
    /// ```
    pub async fn get_template(&self, id: &RecordId) -> Result<AgentTemplate, String> {
        // .select() with RecordId returns Option<T>
        let template: Option<AgentTemplate> = self
            .client()
            .select(id)
            .await
            .map_err(|e| format!("Failed to get template: {}", e))?;

        // Convert Option to Result with descriptive error
        template.ok_or_else(|| format!("Template not found: {}", id.to_sql()))
    }

    /// List all agent templates in the database
    ///
    /// # Returns
    /// * `Ok(Vec<AgentTemplate>)` - All templates ordered by created_at DESC
    /// * `Err(String)` - Error if query fails
    ///
    /// # Database Operation
    /// SELECT * FROM agent_template ORDER BY created_at DESC
    ///
    /// # Performance Note
    /// For large datasets (>1000 templates), consider adding pagination parameters.
    /// Current implementation loads all templates into memory.
    ///
    /// # Example
    /// ```rust
    /// let templates = db.list_templates().await?;
    /// println!("Found {} templates", templates.len());
    /// for template in templates {
    ///     println!("- {} ({})", template.name, template.model);
    /// }
    /// ```
    pub async fn list_templates(&self) -> Result<Vec<AgentTemplate>, String> {
        // .select() without tuple returns Vec<T> (all records)
        let templates: Vec<AgentTemplate> = self
            .client()
            .select("agent_template")
            .await
            .map_err(|e| format!("Failed to list templates: {}", e))?;

        Ok(templates)
    }

    /// Update an existing agent template
    ///
    /// # Arguments
    /// * `template` - AgentTemplate with updated fields (id must match existing record)
    ///
    /// # Returns
    /// * `Ok(())` - Update succeeded
    /// * `Err(String)` - Error if template not found or update fails
    ///
    /// # Database Operation
    /// UPDATE agent_template:$id CONTENT $template
    /// Replaces entire record with new template data (all fields updated).
    ///
    /// # Important
    /// This is a full replacement update. All fields from `template` will overwrite
    /// existing database record. Partial updates not supported in this method.
    ///
    /// # Example
    /// ```rust
    /// let mut template = db.get_template("abc123").await?;
    /// template.system_prompt = "Updated prompt".to_string();
    /// template.max_turns = 100;
    /// db.update_template(&template).await?;
    /// ```
    pub async fn update_template(&self, template: &AgentTemplate) -> Result<(), String> {
        // .update() with content performs full record replacement
        let updated: Option<AgentTemplate> = self
            .client()
            .update(&template.id)
            .content(template.clone())
            .await
            .map_err(|e| format!("Failed to update template: {}", e))?;

        // Validate that record existed and was updated
        updated
            .map(|_| ())
            .ok_or_else(|| format!("Template not found: {}", template.id.to_sql()))
    }

    /// Delete an agent template by ID
    ///
    /// # Arguments
    /// * `id` - Template record ID to delete
    ///
    /// # Returns
    /// * `Ok(())` - Deletion succeeded (or record didn't exist)
    /// * `Err(String)` - Error if database operation fails
    ///
    /// # Database Operation
    /// DELETE agent_template:$id
    ///
    /// # Side Effects
    /// WARNING: Conversations referencing this template_id will have dangling foreign keys.
    /// Consider implementing cascading delete or reference checking before deletion.
    ///
    /// # Example
    /// ```rust
    /// db.delete_template("abc123").await?;
    /// println!("Template deleted successfully");
    /// ```
    pub async fn delete_template(&self, id: &RecordId) -> Result<(), String> {
        // .delete() removes record and returns it (Option<T>)
        let deleted: Option<AgentTemplate> = self
            .client()
            .delete(id)
            .await
            .map_err(|e| format!("Failed to delete template: {}", e))?;

        // Validate that record existed and was deleted
        deleted
            .map(|_| ())
            .ok_or_else(|| format!("Template not found: {}", id.to_sql()))
    }

    /// Alias for list_templates (matches Model API naming)
    pub async fn list_agent_templates(&self) -> Result<Vec<AgentTemplate>, String> {
        self.list_templates().await
    }
}
