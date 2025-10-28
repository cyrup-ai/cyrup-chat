//! Core types and state definitions for login functionality

use crate::auth::{AuthState, Provider};
use crate::environment::{
    model::{Account, AppData, TokenData},
    types::Instance,
};
use dioxus::prelude::*;
use std::cell::RefCell;

// Modern Dioxus signal-based state management
pub type LoginSignal = Signal<LoginState>;

#[allow(dead_code)] // Follow user ID constant - pending integration
pub const FOLLOW_USER_ID: &str = "109325706684051157";

#[allow(dead_code)] // Login reducer - architectural scaffolding pending integration
pub struct LoginReducer;

#[derive(Debug, Clone)]
#[allow(dead_code)] // Login selection states - pending integration
pub enum Selection {
    /// The instance
    Instance(Instance),
    /// The Host url
    Host(String),
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // Login actions - architectural scaffolding pending integration
pub enum LoginAction {
    // OAuth provider selection and login
    SelectProvider(Provider),
    StartOAuth(Provider),
    OAuthCompleted(Result<AuthState, String>),

    // Modern Mastodon support with enhanced error handling
    Load,
    LoadedInstances(Vec<Instance>),
    SelectInstance(Selection),
    ChosenInstance,
    RetrieveUrl(super::model::ModelContainer, Box<Result<AppData, String>>),
    EnteredCode(String),
    ValidatedCode(Result<TokenData, String>),
    RetrievedUser(Box<Result<Account, String>>),
    SaveCredentials,
    CloseLogin,

    ActionRegister,
    ActionFollow,
    ActionFollowDone(Result<bool, String>),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub struct LoginState {
    // OAuth state
    pub is_loading: bool,
    pub error_message: Option<String>,
    pub selected_provider: Option<Provider>,
    pub auth_state: Option<AuthState>,
    pub done: bool,
    pub close: bool,

    // Modern Mastodon support with enhanced state management
    pub instances: Vec<Instance>,
    pub selected_instance: Option<Instance>,
    pub selected_instance_url: Option<String>,
    pub app_data: Option<AppData>,
    pub code: Option<String>,
    pub verification_code: Option<String>,
    pub access_token: Option<TokenData>,
    pub model: Option<super::model::ModelContainer>,
    pub account: Option<Account>,
    pub did_follow: bool,
    pub send_model: RefCell<Option<super::model::ModelContainer>>,
}
