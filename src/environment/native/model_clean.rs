pub use megalodon::entities::attachment::*;
pub use megalodon::entities::{
    Attachment, Card, Context, Conversation, Emoji, Instance, Notification, Relationship, Status,
    StatusVisibility, Tag, UploadMedia, notification::NotificationType,
};
use megalodon::megalodon::{
    GetArrayOptions, GetArrayWithSinceOptions, GetListTimelineInputOptions,
    GetNotificationsInputOptions, GetTimelineOptions, PostStatusOutput, SearchAccountInputOptions,
};
pub use megalodon::streaming::Message;
use megalodon::{entities::List, megalodon::AccountFollowersInputOptions};

use megalodon::{
    Megalodon,
    megalodon::{
        FollowAccountInputOptions, GetAccountStatusesInputOptions, GetTimelineOptionsWithLocal,
        PostStatusInputOptions, UpdateMediaInputOptions, UploadMediaInputOptions,
    },
};
use reqwest::header::HeaderValue;
use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

/// Error type for Model creation failures
#[derive(Debug)]
pub enum ModelError {
    /// Failed to create Megalodon client
    ClientCreationFailed(String),
}

impl std::fmt::Display for ModelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ClientCreationFailed(msg) => {
                write!(f, "Failed to create Megalodon client: {msg}")
            }
        }
    }
}

impl std::error::Error for ModelError {}

#[derive(Clone)]
pub struct Model {
    pub url: String,
    pub has_token: bool,
    client: Arc<Box<dyn Megalodon + Send + Sync>>,
    instance: Arc<Mutex<Option<Instance>>>,
    is_logged_in: Arc<AtomicBool>,
}

impl Model {
    /// Create a new Model - either works or fails, no fuckery
    pub fn new(url: String, token: Option<String>) -> Result<Self, ModelError> {
        let has_token = token.is_some();
        let client = megalodon::generator(megalodon::SNS::Mastodon, url.clone(), token, None)
            .map_err(|e| ModelError::ClientCreationFailed(format!("{:?}", e)))?;

        Ok(Self {
            url,
            has_token,
            client: Arc::new(client),
            instance: Arc::default(),
            is_logged_in: Arc::new(AtomicBool::new(false)),
        })
    }

    /// List accounts from a specific list
    /// Fetches accounts from Mastodon API list endpoint
    pub async fn list_accounts(
        &self,
        account_id: String,
        list_id: &str,
        after: Option<String>,
    ) -> Result<Vec<Account>, String> {
        log::debug!(
            "list_accounts called for account {} list {} after {:?}",
            account_id,
            list_id,
            after
        );
        // Implement actual list accounts API call through megalodon client
        match self
            .client
            .get_accounts_in_list(list_id.to_string(), None)
            .await
        {
            Ok(response) => {
                let accounts: Vec<Account> = response.json.into_iter().map(Account::from).collect();
                Ok(accounts)
            }
            Err(e) => {
                log::error!("Failed to fetch list accounts: {:?}", e);
                Err(format!("API Error: {e:?}"))
            }
        }
    }

    /// Delete a status
    /// Uses megalodon client to delete a status by ID
    pub async fn delete_status(&self, status_id: String) -> Result<(), String> {
        log::debug!("Deleting status: {}", status_id);
        match self.client.delete_status(status_id).await {
            Ok(_) => Ok(()),
            Err(e) => {
                log::error!("Failed to delete status: {:?}", e);
                Err(format!("Delete status failed: {e:?}"))
            }
        }
    }

    /// Reply to a status
    /// Creates a new status as a reply to the specified status
    pub async fn reply_to_status(
        &self,
        status_id: String,
        content: String,
    ) -> Result<Status, String> {
        log::debug!("Replying to status {} with content: {}", status_id, content);

        let options = megalodon::megalodon::PostStatusInputOptions {
            in_reply_to_id: Some(status_id),
            visibility: Some(StatusVisibility::Public),
            ..Default::default()
        };

        match self.client.post_status(content, Some(&options)).await {
            Ok(response) => match response.json() {
                megalodon::megalodon::PostStatusOutput::Status(status) => Ok(status),
                megalodon::megalodon::PostStatusOutput::ScheduledStatus(scheduled) => {
                    log::warn!("Reply was scheduled: {:?}", scheduled);
                    Err("Reply was scheduled instead of posted immediately".to_string())
                }
            },
            Err(e) => {
                log::error!("Failed to reply to status: {:?}", e);
                Err(format!("Reply failed: {e:?}"))
            }
        }
    }

    /// Pin a status
    /// Pins a status to the user's profile
    pub async fn pin_status(&self, status_id: String) -> Result<Status, String> {
        log::debug!("Pinning status: {}", status_id);
        match self.client.pin_status(status_id).await {
            Ok(response) => Ok(response.json()),
            Err(e) => {
                log::error!("Failed to pin status: {:?}", e);
                Err(format!("Pin status failed: {e:?}"))
            }
        }
    }

    /// Unpin a status
    /// Unpins a previously pinned status from the user's profile
    pub async fn unpin_status(&self, status_id: String) -> Result<Status, String> {
        log::debug!("Unpinning status: {}", status_id);
        match self.client.unpin_status(status_id).await {
            Ok(response) => Ok(response.json()),
            Err(e) => {
                log::error!("Failed to unpin status: {:?}", e);
                Err(format!("Unpin status failed: {e:?}"))
            }
        }
    }

    /// Archive a status by bookmarking it
    /// 
    /// Since Mastodon doesn't have a native archive function, this implementation
    /// uses the bookmark API to provide archive functionality. Bookmarked statuses
    /// can be retrieved later and serve as an effective archival mechanism.
    /// 
    /// # Arguments
    /// * `status_id` - The ID of the status to archive
    /// 
    /// # Returns
    /// * `Result<Status, String>` - The archived status or an error message
    pub async fn archive_status(&self, status_id: String) -> Result<Status, String> {
        log::debug!("Archiving status by bookmarking: {}", status_id);
        // Use bookmark API as the archival mechanism
        match self.client.bookmark_status(status_id.clone()).await {
            Ok(response) => {
                log::info!("Status {} archived by bookmarking", status_id);
                Ok(response.json())
            }
            Err(e) => {
                log::error!("Failed to archive status: {:?}", e);
                Err(format!("Archive status failed: {e:?}"))
            }
        }
    }
}

impl std::fmt::Debug for Model {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Model").finish()
    }
}