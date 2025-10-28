//! Network processing for status mutations

use crate::StatusMutation;
use crate::environment::Environment;

/// Process a single mutation asynchronously with zero allocation patterns
///
/// This function handles the actual network request for the mutation
/// and returns the result for further processing.
#[inline(always)]
pub async fn process_single_mutation(
    environment: &Environment,
    status_id: &str,
    mutation: &StatusMutation,
) -> Result<crate::environment::model::Status, String> {
    match mutation {
        StatusMutation::Like | StatusMutation::Favorite | StatusMutation::Favourite(true) => {
            environment
                .model
                .set_favourite(status_id.to_string(), true)
                .await
                .map_err(|e| e.to_string())
        }
        StatusMutation::Unlike | StatusMutation::Unfavorite | StatusMutation::Favourite(false) => {
            environment
                .model
                .set_favourite(status_id.to_string(), false)
                .await
                .map_err(|e| e.to_string())
        }
        StatusMutation::Repost | StatusMutation::Boost(true) => environment
            .model
            .set_reblog(status_id.to_string(), true)
            .await
            .map_err(|e| e.to_string()),
        StatusMutation::Boost(false) => environment
            .model
            .set_reblog(status_id.to_string(), false)
            .await
            .map_err(|e| e.to_string()),
        StatusMutation::Bookmark(true) => environment
            .model
            .set_bookmark(status_id.to_string(), true)
            .await
            .map_err(|e| e.to_string()),
        StatusMutation::Bookmark(false) => environment
            .model
            .set_bookmark(status_id.to_string(), false)
            .await
            .map_err(|e| e.to_string()),
        StatusMutation::Delete => environment
            .model
            .delete_status(status_id.to_string())
            .await
            .map_err(|e| e.to_string())
            .and_then(|_| Err("Delete operations don't return updated status".to_string())),
        StatusMutation::Pin => environment
            .model
            .pin_status(status_id.to_string())
            .await
            .map_err(|e| e.to_string()),
        StatusMutation::Unpin => environment
            .model
            .unpin_status(status_id.to_string())
            .await
            .map_err(|e| e.to_string()),
        StatusMutation::Create => {
            // Create operations don't have a status_id context - this is invalid for batch processing
            Err("Create operations require content and cannot be batch processed with status_id only".to_string())
        }
        StatusMutation::Update => {
            // Update operations require new content - this is invalid for batch processing
            Err("Update operations require content and cannot be batch processed with status_id only".to_string())
        }
        StatusMutation::Reply => {
            // Reply operations require content - this is invalid for batch processing
            Err("Reply operations require content and cannot be batch processed with status_id only".to_string())
        }
        StatusMutation::Archive => {
            // Archive via status retrieval (effectively caching the status)
            environment
                .model
                .archive_status(status_id.to_string())
                .await
                .map_err(|e| e.to_string())
        }
    }
}
