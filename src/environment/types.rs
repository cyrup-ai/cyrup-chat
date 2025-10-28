use chrono::{DateTime, Utc};
use enumset::EnumSet;
use serde::{Deserialize, Serialize};

// Repository Types

#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
    pub picture: String,
    pub username: Option<String>, // GitHub has usernames, Google doesn't
    pub provider: String,         // Store the provider type (Google/GitHub)
    pub last_login: DateTime<Utc>,
    pub instance_url: String,

    pub token_access_token: String,
    pub token_scope: String,
    pub token_created: u64,
    pub token_expires: Option<u64>,
    pub token_refresh_token: Option<String>,

    pub app_id: String,
    pub app_name: String,
    pub app_website: Option<String>,
    pub app_client_id: String,
    pub app_client_secret: String,
    pub app_auth_url: Option<String>,
    pub app_redirect_uri: String,
}

impl User {
    pub fn new(
        instance_url: String,
        account: megalodon::entities::Account,
        token: crate::environment::native::model::TokenData,
        data: crate::environment::native::model::AppData,
    ) -> Self {
        Self {
            id: account.id.clone(),
            name: if account.display_name.is_empty() {
                account.username.clone()
            } else {
                account.display_name.clone()
            },
            email: "".to_string(), // Megalodon Account doesn't have email field - will be populated from OAuth
            picture: account.avatar.clone(),
            username: Some(account.username.clone()),
            provider: "unknown".to_string(), // Will be set by caller based on provider
            last_login: Utc::now(),
            instance_url,
            token_access_token: token.access_token.clone(),
            token_scope: token.scope.clone().unwrap_or_default(),
            token_created: token.created_at.unwrap_or_default(),
            token_expires: token.expires_in,
            token_refresh_token: token.refresh_token.clone(),
            app_id: data.id.clone(),
            app_name: data.name.clone(),
            app_website: data.website.clone(),
            app_client_id: data.client_id.clone(),
            app_client_secret: data.client_secret.clone(),
            app_auth_url: data.url.clone(),
            app_redirect_uri: data.redirect_uri.clone().unwrap_or_default(),
        }
    }

    /// Create User from AuthState for proper OAuth integration
    pub fn from_auth_state(auth_state: &crate::auth::AuthState, instance_url: String) -> Self {
        Self {
            id: auth_state.user.id.clone(),
            name: auth_state.user.name.clone(),
            email: auth_state.user.email.clone(),
            picture: auth_state.user.picture.clone(),
            username: auth_state.user.username.clone(),
            provider: format!("{:?}", auth_state.provider), // Store provider as string
            last_login: Utc::now(),
            instance_url,
            token_access_token: "".to_string(), // Tokens now stored in vault
            token_scope: "read write follow".to_string(), // Default scope
            token_created: Utc::now().timestamp() as u64,
            token_expires: None,       // Tokens now managed by vault
            token_refresh_token: None, // Tokens now stored in vault
            // Default app data - these would come from OAuth app registration
            app_id: "cyrup-chat".to_string(),
            app_name: "CYRUP Chat".to_string(),
            app_website: Some("https://cyrup.ai".to_string()),
            app_client_id: "client-id".to_string(), // Placeholder - real ID from OAuth
            app_client_secret: "client-secret".to_string(), // Placeholder - real secret from OAuth
            app_auth_url: None,
            app_redirect_uri: "ai.cyrup.chat://oauth/callback".to_string(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Marker {
    /// When was the marker set
    pub set: DateTime<Utc>,
    /// The last / highest status the user saw
    pub id: String,
    /// The id for which this marker is saved
    pub marker_id: String,
}

#[derive(Default, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct UiConfig {
    pub visibility:
        std::collections::HashMap<String, EnumSet<crate::view_model::AccountVisibility>>,
    pub last_notification_id: Option<StatusId>,
    pub zoom: UiZoom,
    #[serde(default)]
    pub direction: TimelineDirection,
    #[serde(default)]
    pub post_window_inline: bool,
    // Missing fields that were causing compilation errors
    #[serde(default)]
    pub sidebar_visible: bool,
    #[serde(default = "default_text_size")]
    pub text_size: f32,
    /// Timeline of status view models for efficient UI updates
    #[serde(default)]
    pub timeline: Option<Vec<crate::view_model::StatusViewModel>>,
}

fn default_text_size() -> f32 {
    14.0
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Default)]
pub enum TimelineDirection {
    NewestBottom,
    #[default]
    NewestTop,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum ScrollDirection {
    Up,
    Down,
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize, Eq, PartialEq, FromRepr)]
#[repr(u8)]
#[derive(Default)]
pub enum UiZoom {
    Z90 = 90,
    #[default]
    Z100 = 100,
    Z110 = 110,
    Z120 = 120,
    Z130 = 130,
    Z140 = 140,
    Z150 = 150,
}

impl UiZoom {
    pub fn css_class(&self) -> String {
        let value: u8 = *self as u8;
        format!("zoom{value}")
    }
}

impl UiZoom {
    const CHANGE: u8 = 10;

    pub fn increase(&self) -> Option<Self> {
        let mut v: u8 = *self as u8;
        v += Self::CHANGE;
        Self::from_repr(v)
    }

    pub fn decrease(&self) -> Option<Self> {
        let mut v: u8 = *self as u8;
        v -= Self::CHANGE;
        Self::from_repr(v)
    }
}

// Instance Types

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Instance {
    pub id: String,
    pub name: String,
    pub users: String,
    pub thumbnail: Option<String>,
}

impl Default for Instance {
    fn default() -> Self {
        Self {
            id: "cyrup-chat".to_string(),
            name: "CYRUP Chat".to_string(),
            users: "1".to_string(),
            thumbnail: None,
        }
    }
}

impl Instance {
    pub fn url(&self) -> String {
        format!("https://{}", self.name)
    }
}

// Menu

use strum_macros::Display;
use strum_macros::EnumIter;
use strum_macros::FromRepr;
use strum_macros::IntoStaticStr;

use crate::view_model::StatusId;

#[derive(IntoStaticStr, EnumIter, Display, Debug, Clone, Copy, Eq, PartialEq)]
pub enum MainMenuEvent {
    NewPost,
    Logout,
    Reload,
    ScrollUp,
    ScrollDown,
    TextSizeIncrease,
    TextSizeDecrease,
    TextSizeReset,
    Timeline,
    Mentions,
    Messages,
    More,
    PostWindowSubmit,
    PostWindowAttachFile,
    CYRUPHelp,
    Settings,
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
pub struct MainMenuConfig {
    pub logged_in: bool,
    pub enable_scroll: bool,
    pub enable_postwindow: bool,
}

pub trait ActionFromEvent {
    fn make_focus_event(focus: bool) -> Option<Self>
    where
        Self: Sized;
    fn make_menu_event(event: MainMenuEvent) -> Option<Self>
    where
        Self: Sized;
    fn make_close_window_event() -> Option<Self>
    where
        Self: Sized;
}

#[derive(Clone, Debug, PartialEq)]
pub enum FocusChange {
    Gained,
    Lost,
}

impl From<bool> for FocusChange {
    fn from(focused: bool) -> Self {
        if focused {
            FocusChange::Gained
        } else {
            FocusChange::Lost
        }
    }
}

#[derive(Clone, Debug)]
pub enum AppEvent {
    FocusChange(FocusChange),
    MenuEvent(crate::environment::types::MainMenuEvent),
    FileEvent(FileEvent),
    ClosingWindow,
}

impl ActionFromEvent for AppEvent {
    fn make_focus_event(focus: bool) -> Option<Self>
    where
        Self: Sized,
    {
        Some(AppEvent::FocusChange(FocusChange::from(focus)))
    }
    fn make_menu_event(event: MainMenuEvent) -> Option<Self>
    where
        Self: Sized,
    {
        Some(AppEvent::MenuEvent(event))
    }
    fn make_close_window_event() -> Option<Self>
    where
        Self: Sized,
    {
        Some(AppEvent::ClosingWindow)
    }
}

#[derive(Clone, Debug)]
pub enum FileEvent {
    Hovering(bool),
    Dropped(Vec<std::path::PathBuf>),
    Cancelled,
}
