//! Room list provider - displays conversations as chat rooms
use super::TimelineProvider;
use crate::{
    environment::{Environment, types::TimelineDirection},
    view_model::{ConversationSummary, StatusId, StatusViewModel},
};

use dioxus::prelude::{ReadableExt, WritableExt};
use futures_util::Future;
use std::pin::Pin;
use surrealdb_types::ToSql;

/// Provider that loads conversations as "rooms" from database
/// 
/// In the context of agent conversations, "rooms" represent individual
/// conversation threads that can be joined and interacted with.
pub struct RoomListProvider {
    environment: Environment,
}

impl RoomListProvider {
    pub fn new(environment: Environment) -> Self {
        Self { environment }
    }
}

impl std::fmt::Debug for RoomListProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RoomListProvider").finish()
    }
}

impl TimelineProvider for RoomListProvider {
    type Id = StatusId; // Room ID mapped to StatusId
    type Element = crate::environment::model::Status; // Timeline system expects Status
    type ViewModel = StatusViewModel; // Transformed for UI

    fn should_auto_reload(&self) -> bool {
        false // User manually refreshes room list
    }

    fn identifier(&self) -> &str {
        "RoomListProvider"
    }

    fn forced_direction(&self) -> Option<TimelineDirection> {
        Some(TimelineDirection::NewestTop) // Newest rooms at top
    }

    fn reset(&self) {
        self.environment
            .storage
            .write_unchecked()
            .room_list
            .clear();
    }

    fn scroll_to_item(&self, updates: &[crate::environment::model::Status]) -> Option<StatusId> {
        // Scroll to most recent status
        updates
            .iter()
            .max_by_key(|status| status.created_at)
            .map(|status| {
                log::debug!("Room list scrolling to status: {}", status.id);
                StatusId(status.id.clone())
            })
    }

    fn request_data(
        &self,
        _after: Option<StatusId>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<crate::environment::model::Status>, String>> + Send>>
    {
        // Load conversations as "rooms"
        let db = self.environment.database.clone();
        Box::pin(async move {
            let summaries = db.list_conversations().await?;
            // Convert ConversationSummary to Status for timeline system
            Ok(summaries
                .iter()
                .map(conversation_summary_to_room_status)
                .collect())
        })
    }

    fn process_new_data(
        &self,
        updates: &[crate::environment::model::Status],
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
            .merge_room_list(view_models, is_reload);

        can_load_more
    }

    fn data(&self, _direction: TimelineDirection) -> Vec<StatusViewModel> {
        self.environment
            .storage
            .read_unchecked()
            .room_list
            .clone()
    }
}

/// Transform ConversationSummary to Status representing a chat room
fn conversation_summary_to_room_status(
    summary: &ConversationSummary,
) -> crate::environment::model::Status {
    use crate::environment::model::Status;
    use megalodon::entities::{Account, StatusVisibility};

    Status {
        id: summary.id.0.to_sql(),
        uri: String::new(),
        created_at: *summary.last_message_timestamp,
        account: Account {
            id: summary.id.0.to_sql(),
            username: format!("room_{}", &summary.id.0.to_sql()[..8]),
            acct: format!("room_{}", &summary.id.0.to_sql()[..8]),
            display_name: summary.title.clone(),
            locked: false,
            discoverable: None,
            group: None,
            noindex: None,
            moved: None,
            suspended: None,
            limited: None,
            created_at: *summary.last_message_timestamp,
            followers_count: 0,
            following_count: 0,
            statuses_count: 0,
            note: String::new(),
            url: String::new(),
            avatar: summary.agent_avatar.clone().unwrap_or_default(),
            avatar_static: summary.agent_avatar.clone().unwrap_or_default(),
            header: String::new(),
            header_static: String::new(),
            emojis: Vec::new(),
            fields: Vec::new(),
            bot: false,
            source: None,
            role: None,
            mute_expires_at: None,
        },
        content: summary.last_message_preview.clone(),
        visibility: StatusVisibility::Public,
        replies_count: summary.unread_count,
        in_reply_to_id: None,
        sensitive: false,
        spoiler_text: String::new(),
        media_attachments: Vec::new(),
        mentions: Vec::new(),
        tags: Vec::new(),
        emojis: Vec::new(),
        reblogs_count: 0,
        favourites_count: 0,
        url: None,
        in_reply_to_account_id: None,
        reblog: None,
        poll: None,
        card: None,
        language: None,
        plain_content: Some(summary.last_message_preview.clone()),
        edited_at: None,
        favourited: None,
        reblogged: None,
        muted: None,
        bookmarked: None,
        pinned: None,
        quote: false,
        application: None,
        emoji_reactions: None,
    }
}
