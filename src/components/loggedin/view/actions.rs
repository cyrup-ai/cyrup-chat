//! Action conversion utilities for status interactions

use super::super::reducer::Action;
use crate::components::post::PostKind;
use crate::view_model::StatusViewModel;
use crate::widgets::StatusAction;
use crate::{PublicAction, StatusMutation};

impl From<(StatusAction, &StatusViewModel)> for Action {
    fn from(value: (StatusAction, &StatusViewModel)) -> Self {
        let (action, status) = value;
        match action {
            StatusAction::Clicked => Action::SelectConversation(status.id.clone()),
            StatusAction::Boost(s) => Action::Public(PublicAction::StatusMutation(
                StatusMutation::Boost(s),
                status.clone(),
            )),
            StatusAction::Reply => Action::Post(PostKind::Reply(status.clone())),
            StatusAction::Favorite(s) => Action::Public(PublicAction::StatusMutation(
                StatusMutation::Favourite(s),
                status.clone(),
            )),
            StatusAction::Bookmark(s) => Action::Public(PublicAction::StatusMutation(
                StatusMutation::Bookmark(s),
                status.clone(),
            )),
            StatusAction::OpenAccount(a) => Action::Public(PublicAction::OpenLink(a)),
            StatusAction::OpenLink(a) => Action::Public(PublicAction::OpenLink(a)),
            StatusAction::OpenTag(a) => Action::Public(PublicAction::OpenTag(a)),
            StatusAction::OpenImage(a) => Action::Public(PublicAction::OpenImage(a)),
            StatusAction::OpenVideo(a) => Action::Public(PublicAction::OpenVideo(a)),
            StatusAction::Copy(a) => Action::Public(PublicAction::Copy(a)),
        }
    }
}
