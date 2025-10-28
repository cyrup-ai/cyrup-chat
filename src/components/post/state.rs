#[allow(dead_code)] // Post state management - architectural scaffolding pending integration
use std::path::PathBuf;
use std::str::FromStr;

use crate::environment::model::StatusVisibility;
use crate::environment::types::UiConfig;
use crate::view_model::AccountViewModel;
use crate::view_model::{AttachmentMedia, StatusViewModel};

#[derive(Clone, Debug, PartialEq)]
pub enum PostKind {
    Post,
    Reply(StatusViewModel),
    ReplyPrivate(StatusViewModel),
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub account: AccountViewModel,
    pub kind: PostKind,
    pub is_window: bool,
    pub image_paths: Vec<PathBuf>,
    pub images: Vec<AttachmentMedia>,
    pub posting: bool,
    pub dropping_file: bool,
    pub error_message: Option<String>,
    pub visibility: Option<Visibility>,
    pub text: String,
    pub validity: (bool, u32, u32),
    pub config: UiConfig,
}

impl Default for State {
    fn default() -> Self {
        Self {
            account: AccountViewModel::default(),
            kind: PostKind::Post,
            is_window: false,
            image_paths: Vec::new(),
            images: Vec::new(),
            posting: false,
            dropping_file: false,
            error_message: None,
            visibility: None,
            text: String::new(),
            validity: (false, 0, 0),
            config: UiConfig::default(),
        }
    }
}

impl State {
    #[allow(dead_code)] // Post state constructor - used in loggedin/view.rs:152 but compiler doesn't detect it
    pub fn new(
        account: AccountViewModel,
        kind: PostKind,
        is_window: bool,
        paths: Vec<PathBuf>,
    ) -> Self {
        Self {
            account,
            kind,
            is_window,
            images: Default::default(),
            image_paths: paths,
            posting: Default::default(),
            dropping_file: Default::default(),
            error_message: Default::default(),
            visibility: Default::default(),
            text: Default::default(),
            validity: Default::default(),
            config: Default::default(),
        }
    }

    /// Create State from a StatusViewModel for timeline post display
    /// This is used when rendering posts in timeline components
    pub fn from_status(status: crate::view_model::StatusViewModel) -> Self {
        Self {
            account: status.account.clone(),
            kind: PostKind::Post,
            is_window: false,
            images: Default::default(),
            image_paths: Default::default(),
            posting: false,
            dropping_file: false,
            error_message: None,
            visibility: Some(match status.visibility {
                crate::view_model::Visibility::Public => Visibility::Public,
                crate::view_model::Visibility::Unlisted => Visibility::Unlisted,
                crate::view_model::Visibility::Private => Visibility::Private,
                crate::view_model::Visibility::Direct => Visibility::Direct,
                crate::view_model::Visibility::Local => Visibility::Local,
            }),
            text: status
                .content
                .iter()
                .map(|item| item.to_string())
                .collect::<String>(),
            validity: (
                true,
                status
                    .content
                    .iter()
                    .map(|item| item.to_string())
                    .collect::<String>()
                    .len() as u32,
                500,
            ),
            config: Default::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Visibility {
    Public,
    Unlisted,
    Private,
    Direct,
    Local,
}

impl FromStr for Visibility {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "public" => Ok(Visibility::Public),
            "unlisted" => Ok(Visibility::Unlisted),
            "private" => Ok(Visibility::Private),
            "direct" => Ok(Visibility::Direct),
            _ => Err("INVALID".to_string()),
        }
    }
}

impl From<&Visibility> for StatusVisibility {
    fn from(value: &Visibility) -> Self {
        match value {
            Visibility::Public => StatusVisibility::Public,
            Visibility::Unlisted => StatusVisibility::Unlisted,
            Visibility::Private => StatusVisibility::Private,
            Visibility::Direct => StatusVisibility::Direct,
            Visibility::Local => StatusVisibility::Unlisted, // Map Local to Unlisted (closest equivalent)
        }
    }
}
