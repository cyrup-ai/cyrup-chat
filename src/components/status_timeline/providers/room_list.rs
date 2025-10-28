//! Room list provider - displays multi-agent rooms as timeline
//!
//! Cloned from conversation_list.rs with rooms-specific adaptations.
//! Implements TimelineProvider trait for room list display.

use super::TimelineProvider;
use crate::{
    environment::{types::TimelineDirection, Environment},
    view_model::{RoomSummary, StatusId, StatusViewModel},
};

use dioxus::prelude::{ReadableExt, WritableExt};
use futures_util::Future;
use std::pin::Pin;

/// Provider that loads room list from database
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
    type Id = StatusId;
    type Element = crate::environment::model::Status;
    type ViewModel = StatusViewModel;

    fn should_auto_reload(&self) -> bool {
        false // User manually refreshes room list
    }

    fn identifier(&self) -> &str {
        "RoomListProvider"
    }

    fn forced_direction(&self) -> Option<TimelineDirection> {
        Some(TimelineDirection::NewestTop)
    }

    fn reset(&self) {
        self.environment
            .storage
            .write_unchecked()
            .room_list
            .clear();
    }

    fn scroll_to_item(&self, updates: &[crate::environment::model::Status]) -> Option<StatusId> {
        updates
            .iter()
            .max_by_key(|status| status.created_at)
            .map(|status| {
                log::debug!("[RoomList] Scrolling to room: {}", status.id);
                StatusId(status.id.clone())
            })
    }

    fn request_data(
        &self,
        _after: Option<StatusId>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<crate::environment::model::Status>, String>> + Send>>
    {
        let db = self.environment.database.clone();
        Box::pin(async move {
            log::debug!("[RoomList] Fetching rooms from database");
            let summaries = db.list_rooms().await?;
            log::info!("[RoomList] Loaded {} rooms", summaries.len());

            Ok(summaries
                .iter()
                .map(room_summary_to_status)
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
        let view_models: Vec<StatusViewModel> = updates.iter().map(StatusViewModel::new).collect();

        log::debug!(
            "[RoomList] Processing {} room updates (reload={})",
            view_models.len(),
            is_reload
        );

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

/// Transform RoomSummary to Status for timeline system compatibility
fn room_summary_to_status(summary: &RoomSummary) -> crate::environment::model::Status {
    use crate::environment::model::Status;
    use megalodon::entities::{Account, StatusVisibility};

    let participant_names: Vec<String> = summary
        .participants
        .iter()
        .map(|p| p.0.clone())
        .collect();

    Status {
        id: summary.id.0.clone(),
        uri: String::new(),
        created_at: summary.last_message_timestamp,
        account: Account {
            id: summary.id.0.clone(),
            username: format!("room_{}", &summary.id.0[..8]),
            acct: format!("room_{}", &summary.id.0[..8]),
            display_name: summary.title.clone(),
            locked: false,
            discoverable: None,
            group: None,
            noindex: None,
            moved: None,
            suspended: None,
            limited: None,
            created_at: summary.last_message_timestamp,
            followers_count: summary.participants.len() as i32,
            following_count: 0,
            statuses_count: 0,
            note: format!("Participants: {}", participant_names.join(", ")),
            url: String::new(),
            avatar: String::new(),
            avatar_static: String::new(),
            header: String::new(),
            header_static: String::new(),
            emojis: Vec::new(),
            fields: Vec::new(),
            bot: true,
            source: None,
            role: None,
            mute_expires_at: None,
        },
        content: summary.last_message_preview.clone(),
        visibility: StatusVisibility::Public,
        replies_count: 0,
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
