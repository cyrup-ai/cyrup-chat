//! Bookmarks timeline provider - displays bookmarked messages
use super::TimelineProvider;
use crate::{
    environment::{Environment, model::Status, types::TimelineDirection},
    view_model::{StatusId, StatusViewModel, message::Message},
};

use dioxus::prelude::{ReadableExt, WritableExt};
use futures_util::Future;
use std::pin::Pin;
use surrealdb_types::ToSql;

/// Provider that loads bookmarked messages from database
pub struct BookmarkTimelineProvider {
    environment: Environment,
}

impl BookmarkTimelineProvider {
    pub fn new(environment: Environment) -> Self {
        Self { environment }
    }
}

impl std::fmt::Debug for BookmarkTimelineProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BookmarkTimelineProvider").finish()
    }
}

impl TimelineProvider for BookmarkTimelineProvider {
    type Id = StatusId;
    type Element = Status; // Timeline system expects Status
    type ViewModel = StatusViewModel;
    fn should_auto_reload(&self) -> bool {
        false
    }

    fn identifier(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn forced_direction(&self) -> Option<TimelineDirection> {
        Some(TimelineDirection::NewestTop)
    }

    fn reset(&self) {
        self.environment.storage.write_unchecked().bookmarks.clear();
    }

    fn scroll_to_item(&self, updates: &[Status]) -> Option<StatusId> {
        // Find the most recent status to scroll to
        updates
            .iter()
            .max_by_key(|status| status.created_at)
            .map(|status| {
                log::debug!("Bookmarks timeline scrolling to status: {}", status.id);
                StatusId(status.id.clone())
            })
    }

    fn request_data(
        &self,
        _after: Option<StatusId>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Status>, String>> + Send>> {
        // Get bookmarked messages from database and convert to Status
        let db = self.environment.database.clone();
        let user_id = "hardcoded-david-maple"; // Q39: MVP hardcoded user
        Box::pin(async move {
            let messages = db.get_bookmarked_messages(user_id).await?;
            // Convert Message to Status for timeline system
            Ok(messages
                .iter()
                .map(message_to_status_for_bookmark)
                .collect())
        })
    }

    fn process_new_data(
        &self,
        updates: &[Status],
        _direction: TimelineDirection,
        is_reload: bool,
    ) -> bool {
        let can_load_more = !updates.is_empty();

        // Transform Status → StatusViewModel
        let view_models: Vec<StatusViewModel> = updates.iter().map(StatusViewModel::new).collect();

        // Merge into storage
        self.environment
            .storage
            .write_unchecked()
            .merge_conversation_list(view_models, is_reload);

        can_load_more
    }

    fn data(&self, _direction: TimelineDirection) -> Vec<StatusViewModel> {
        self.environment.storage.read_unchecked().bookmarks.clone()
    }
}

/// Transform Message to Status for bookmark timeline
fn message_to_status_for_bookmark(msg: &Message) -> Status {
    use crate::environment::model::Status;
    use megalodon::entities::{Account, StatusVisibility};

    Status {
        id: msg.id.to_sql(),
        uri: String::new(),
        created_at: *msg.timestamp,
        account: Account {
            id: msg.conversation_id.to_sql(),
            username: msg.author.clone(),
            acct: msg.author.clone(),
            display_name: msg.author.clone(),
            locked: false,
            discoverable: None,
            group: None,
            noindex: None,
            moved: None,
            suspended: None,
            limited: None,
            created_at: *msg.timestamp,
            followers_count: 0,
            following_count: 0,
            statuses_count: 0,
            note: String::new(),
            url: String::new(),
            avatar: String::new(),
            avatar_static: String::new(),
            header: String::new(),
            header_static: String::new(),
            emojis: Vec::new(),
            fields: Vec::new(),
            bot: false,
            source: None,
            role: None,
            mute_expires_at: None,
        },
        content: msg.content.clone(),
        visibility: StatusVisibility::Public,
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: Vec::new(),
        emojis: Vec::new(),
        reblogs_count: 0,
        favourites_count: 0,
        replies_count: 0,
        url: None,
        in_reply_to_id: msg.in_reply_to.as_ref().map(|id| id.to_sql()),
        in_reply_to_account_id: None,
        reblog: None,
        poll: None,
        card: None,
        language: None,
        plain_content: Some(msg.content.clone()),
        edited_at: None,
        favourited: None,
        reblogged: None,
        muted: None,
        bookmarked: Some(true),
        pinned: Some(msg.pinned),
        quote: false,
        application: None,
        emoji_reactions: None,
    }
}
