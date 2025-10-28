mod any;
pub use any::AnyTimelineProvider;

mod bookmarks;
pub use bookmarks::BookmarkTimelineProvider;

mod conversation_list;
pub use conversation_list::ConversationListProvider;

mod room_list;
pub use room_list::RoomListProvider;

use std::pin::Pin;

use futures_util::Future;

use crate::environment::types::TimelineDirection;

pub trait TimelineProvider: std::fmt::Debug {
    type Id;
    type Element;
    type ViewModel;
    fn should_auto_reload(&self) -> bool;
    fn identifier(&self) -> &str;
    // Reload was triggered
    fn reset(&self);
    // This provider enforces a direction
    fn forced_direction(&self) -> Option<TimelineDirection>;
    #[allow(clippy::type_complexity)]
    fn request_data(
        &self,
        after: Option<Self::Id>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<Self::Element>, String>> + Send>>;
    fn process_new_data(
        &self,
        updates: &[Self::Element],
        direction: TimelineDirection,
        is_reload: bool,
    ) -> bool;
    fn data(&self, direction: TimelineDirection) -> Vec<Self::ViewModel>;
    fn scroll_to_item(&self, updates: &[Self::Element]) -> Option<Self::Id>;
}
