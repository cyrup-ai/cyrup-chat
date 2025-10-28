//! Component stack action and message types

use crate::PublicAction;
use crate::environment::types::AppEvent;
use crate::view_model::{AccountViewModel, StatusId};

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone)]
#[allow(dead_code)] // Component stack actions - architectural scaffolding pending integration
pub enum Action {
    Initial,
    PushProfile(AccountViewModel),
    ResolvePushProfile(String),
    SelectConversation(StatusId),
    CloseConversation,
    AppEvent(AppEvent),
    CloseCurrent,
    PublicAction(PublicAction),
    Conversation(PublicAction),
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum Message {
    AppEvent(AppEvent),
    SelectConversation(StatusId),
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Component stack delegate messages - pending integration
pub enum DelegateMessage {
    PublicAction(PublicAction),
    ConversationAction(crate::PublicAction),
}
