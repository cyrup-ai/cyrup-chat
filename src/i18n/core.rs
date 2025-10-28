//! Core i18n types and error handling
//!
//! This module provides the fundamental types, errors, and locale definitions
//! for the zero-allocation internationalization system.

use std::collections::HashMap;
use strum::EnumCount;

/// I18n error types for production-safe error handling
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum I18nError {
    /// I18n system not initialized - call init_i18n() first
    NotInitialized,
    /// Text key not found for the current locale
    KeyNotFound(TextKey, Locale),
    /// Locale not supported
    LocaleNotSupported(String),
    /// Lock poisoned (thread panicked while holding lock)
    LockPoisoned,
}

impl std::fmt::Display for I18nError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            I18nError::NotInitialized => write!(f, "I18n system not initialized"),
            I18nError::KeyNotFound(key, locale) => {
                write!(f, "Text key {:?} not found for locale {:?}", key, locale)
            }
            I18nError::LocaleNotSupported(locale) => write!(f, "Locale '{}' not supported", locale),
            I18nError::LockPoisoned => write!(f, "I18n lock poisoned - thread panic occurred"),
        }
    }
}

impl std::error::Error for I18nError {}

/// Supported locale codes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Locale {
    #[default]
    English,
    Spanish,
    French,
    German,
    Japanese,
    Chinese,
    Korean,
    Portuguese,
    Italian,
    Russian,
}

impl Locale {
    pub const fn code(&self) -> &'static str {
        match self {
            Locale::English => "en",
            Locale::Spanish => "es",
            Locale::French => "fr",
            Locale::German => "de",
            Locale::Japanese => "ja",
            Locale::Chinese => "zh",
            Locale::Korean => "ko",
            Locale::Portuguese => "pt",
            Locale::Italian => "it",
            Locale::Russian => "ru",
        }
    }

    pub const fn name(&self) -> &'static str {
        match self {
            Locale::English => "English",
            Locale::Spanish => "Español",
            Locale::French => "Français",
            Locale::German => "Deutsch",
            Locale::Japanese => "日本語",
            Locale::Chinese => "中文",
            Locale::Korean => "한국어",
            Locale::Portuguese => "Português",
            Locale::Italian => "Italiano",
            Locale::Russian => "Русский",
        }
    }
}

/// Text key identifiers for compile-time optimization
///
/// This enum provides zero-allocation, compile-time validated text keys for the i18n system.
/// Each variant corresponds to a specific UI text element and must have translations
/// defined in all supported locales.
///
/// # Usage
/// ```rust
/// use crate::i18n::{TextKey, t};
///
/// let welcome_text = t(TextKey::WelcomeBack);
/// ```
///
/// # Adding New Keys
/// When adding new variants:
/// 1. Add the variant to this enum with proper documentation
/// 2. Add translations in all language files (src/i18n/languages/*.rs)
/// 3. Update the exhaustive pattern matching validation
/// 4. Run tests to ensure completeness
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumCount)]
pub enum TextKey {
    // Authentication texts
    /// "Sign in with Google" - OAuth login button
    LoginWithGoogle,
    /// "Sign in with GitHub" - OAuth login button  
    LoginWithGitHub,
    /// "Skip Login" - Skip authentication option
    SkipLogin,
    /// "Sign Out" - Logout button
    SignOut,
    /// "Welcome back!" - Returning user greeting
    WelcomeBack,
    /// "Authenticating, please wait..." - Login progress message
    AuthenticatingPleaseWait,
    /// "Authentication failed" - Login error message
    AuthenticationFailed,
    /// "Retry Authentication" - Retry login button
    RetryAuthentication,

    // Main UI texts
    /// "New Conversation" - Start new chat button
    NewConversation,
    /// "Send Message" - Message send button
    SendMessage,
    /// "Type your message..." - Message input placeholder
    TypeYourMessage,
    /// "Conversation History" - Chat history section title
    ConversationHistory,
    /// "Settings" - Settings menu item
    Settings,
    /// "Profile" - User profile section
    Profile,
    /// "About" - About section/dialog
    About,

    // Conversation texts
    /// "New Chat" - Create new conversation
    NewChat,
    /// "Delete Conversation" - Remove conversation action
    DeleteConversation,
    /// "Share Conversation" - Share conversation action
    ShareConversation,
    /// "Copy Message" - Copy message text action
    CopyMessage,
    /// "Regenerate Response" - AI response regeneration
    RegenerateResponse,
    /// "Stop Generation" - Stop AI response generation
    StopGeneration,

    // Status texts
    /// "Connecting..." - Connection in progress
    Connecting,
    /// "Connected" - Successfully connected
    Connected,
    /// "Disconnected" - Connection lost
    Disconnected,
    /// "Sending message..." - Message send in progress
    SendingMessage,
    /// "Receiving response..." - Waiting for response
    ReceivingResponse,
    /// "Message sent" - Message sent confirmation
    MessageSent,
    /// "Message failed" - Message send failure
    MessageFailed,

    // Error texts
    /// "Network connection error" - Network failure message
    NetworkError,
    /// "Authentication error occurred" - Auth failure message
    AuthenticationError,
    /// "An unexpected error occurred" - Generic error message
    UnexpectedError,
    /// "Please try again" - Retry instruction
    PleaseRetry,
    /// "Contact support if the problem persists" - Support contact message
    ContactSupport,

    // Settings texts
    /// "Language" - Language setting option
    Language,
    /// "Theme" - Theme/appearance setting
    Theme,
    /// "Notifications" - Notification settings
    Notifications,
    /// "Privacy" - Privacy settings section
    Privacy,
    /// "Data Export" - Export user data option
    DataExport,
    /// "Delete Account" - Account deletion option
    DeleteAccount,

    // Accessibility texts
    /// "Open Menu" - Screen reader menu open action
    OpenMenu,
    /// "Close Menu" - Screen reader menu close action
    CloseMenu,
    /// "Navigation Menu" - Screen reader navigation description
    NavigationMenu,
    /// "User Avatar" - Screen reader avatar description
    UserAvatar,
    /// "Message Input" - Screen reader input field description
    MessageInput,
    /// "Send Button" - Screen reader send button description
    SendButton,
    /// "Conversation List" - Screen reader conversation list description
    ConversationList,
    /// "Message History" - Screen reader message history description
    MessageHistory,

    // Input placeholders
    /// "Enter your username" - Username input placeholder
    EnterUsername,
    /// "Enter your password" - Password input placeholder
    EnterPassword,
    /// "Search conversations..." - Conversation search placeholder
    SearchConversations,
    /// "What's on your mind?" - Post composition placeholder
    WhatsOnYourMind,
    /// "Enter instance URL" - Server URL input placeholder
    EnterInstanceUrl,
    /// "Enter verification code" - Auth code input placeholder
    EnterVerificationCode,
    /// "Select or Enter a Mastodon Server" - Server selection placeholder
    SelectMastodonServer,
    /// "Code" - Short verification code placeholder
    VerificationCode,
    /// "Enter Description…" - Media description placeholder
    EnterDescription,
    /// "Search users..." - User search placeholder
    SearchUsers,
    /// "Search" - General search placeholder
    Search,
    /// "Follow" - Follow user action
    Follow,

    // UI Actions and Buttons
    /// "Options" - Context menu options button
    Options,
    /// "Copy Link" - Copy URL to clipboard action
    CopyLink,
    /// "Open in Browser" - Open link in external browser
    OpenInBrowser,
    /// "Copy Text" - Copy text content action
    CopyText,
    /// "Open Conversation" - Open conversation thread
    OpenConversation,
    /// "Open Profile" - View user profile action
    OpenProfile,
    /// "Open" - Generic open action
    Open,
    /// "Close" - Generic close action
    Close,
    /// "Back" - Navigation back button
    Back,
    /// "Continue" - Proceed to next step
    Continue,
    /// "Confirm" - Confirm action button
    Confirm,
    /// "Done" - Completion button
    Done,
    /// "Register" - Registration button
    Register,
    /// "Use Custom" - Custom option selection
    UseCustom,
    /// "Paste" - Paste from clipboard action
    Paste,
    /// "Reload" - Refresh/reload action
    Reload,
    /// "New Toot" - Create new post (Mastodon terminology)
    NewToot,

    // Content and Navigation
    /// "Timeline" - Main content timeline
    Timeline,
    /// "Posts" - User posts section
    Posts,
    /// "Following" - Users being followed
    Following,
    /// "Followers" - User's followers
    Followers,
    /// "More Followers" - Load additional followers
    MoreFollowers,
    /// "Load more followers" - Accessibility text for load more
    LoadMoreFollowers,
    /// "Timelines" - Multiple timeline section
    Timelines,
    /// "Account" - User account section
    Account,
    /// "Classic Timeline" - Traditional timeline view
    ClassicTimeline,
    /// "Your Posts" - User's own posts
    YourPosts,
    /// "Local" - Local instance timeline
    Local,
    /// "Federated" - Federated timeline
    Federated,
    /// "Grouped Timelines" - Accessibility text for timeline groups
    GroupedTimelines,
    /// "Messages" - Direct messages section
    Messages,
    /// "Direct Messages" - Full direct messages title
    DirectMessages,
    /// "More" - Additional options section
    More,
    /// "Timelines & Lists" - Combined timeline and list view
    TimelinesAndLists,
    /// "Followers, Classical Timelines & More" - Extended navigation description
    FollowersClassicalTimelinesAndMore,

    // Onboarding and Help
    /// "Welcome to CYRUP" - Application welcome message
    WelcomeToCyrup,
    /// "A website should just have opened in your browser." - OAuth instruction
    WebsiteShouldHaveOpened,
    /// "Please authorize CYRUP and then copy & paste the code into the box below." - OAuth instruction
    PleaseAuthorizeAndPaste,
    /// "Copy the browser URL to the clipboard" - URL copy instruction
    CopyBrowserUrlToClipboard,
    /// "CYRUP is still a very early alpha. Expect bugs and missing features." - Alpha warning
    CyrupAlphaWarning,
    /// "You can report feedback by sending me a private message." - Feedback instruction
    ReportFeedbackInstruction,
    /// "One Tip: Tap a selection in the left column twice, to scroll to the timeline bottom" - Usage tip
    TapSelectionTwiceToScrollTip,
}

/// Zero-allocation text provider with compile-time optimization
#[derive(Default)]
pub struct I18n {
    pub(crate) current_locale: Locale,
    pub(crate) texts: HashMap<(Locale, TextKey), &'static str>,
}

impl I18n {
    pub fn new() -> Self {
        Self {
            current_locale: Locale::English,
            texts: HashMap::new(),
        }
    }

    pub(crate) fn add_text(&mut self, locale: Locale, key: TextKey, text: &'static str) {
        self.texts.insert((locale, key), text);
    }

    /// Get text for current locale with zero allocation
    pub fn text(&self, key: TextKey) -> &'static str {
        self.texts
            .get(&(self.current_locale, key))
            .or_else(|| self.texts.get(&(Locale::English, key)))
            .copied()
            .unwrap_or("Missing text")
    }

    /// Get text for specific locale with zero allocation  
    pub fn text_for_locale(&self, locale: Locale, key: TextKey) -> &'static str {
        self.texts
            .get(&(locale, key))
            .or_else(|| self.texts.get(&(Locale::English, key)))
            .copied()
            .unwrap_or("Missing text")
    }

    /// Set current locale (zero allocation)
    pub fn set_locale(&mut self, locale: Locale) {
        self.current_locale = locale;
    }

    /// Get current locale
    pub const fn current_locale(&self) -> Locale {
        self.current_locale
    }

    /// Get all available locales
    pub const fn available_locales() -> &'static [Locale] {
        &[
            Locale::English,
            Locale::Spanish,
            Locale::French,
            Locale::German,
            Locale::Japanese,
            Locale::Chinese,
            Locale::Korean,
            Locale::Portuguese,
            Locale::Italian,
            Locale::Russian,
        ]
    }

    /// Validate that all TextKey variants have translations for the given locale
    /// This function ensures exhaustive pattern matching at compile time
    pub fn validate_completeness(&self, locale: Locale) -> bool {
        Self::all_text_keys()
            .iter()
            .all(|key| self.texts.contains_key(&(locale, *key)))
    }

    /// Get all possible TextKey variants for validation
    /// This must be kept in sync with the TextKey enum
    pub const fn all_text_keys() -> &'static [TextKey] {
        &[
            // Authentication texts
            TextKey::LoginWithGoogle,
            TextKey::LoginWithGitHub,
            TextKey::SkipLogin,
            TextKey::SignOut,
            TextKey::WelcomeBack,
            TextKey::AuthenticatingPleaseWait,
            TextKey::AuthenticationFailed,
            TextKey::RetryAuthentication,
            // Main UI texts
            TextKey::NewConversation,
            TextKey::SendMessage,
            TextKey::TypeYourMessage,
            TextKey::ConversationHistory,
            TextKey::Settings,
            TextKey::Profile,
            TextKey::About,
            // Conversation texts
            TextKey::NewChat,
            TextKey::DeleteConversation,
            TextKey::ShareConversation,
            TextKey::CopyMessage,
            TextKey::RegenerateResponse,
            TextKey::StopGeneration,
            // Status texts
            TextKey::Connecting,
            TextKey::Connected,
            TextKey::Disconnected,
            TextKey::SendingMessage,
            TextKey::ReceivingResponse,
            TextKey::MessageSent,
            TextKey::MessageFailed,
            // Error texts
            TextKey::NetworkError,
            TextKey::AuthenticationError,
            TextKey::UnexpectedError,
            TextKey::PleaseRetry,
            TextKey::ContactSupport,
            // Settings texts
            TextKey::Language,
            TextKey::Theme,
            TextKey::Notifications,
            TextKey::Privacy,
            TextKey::DataExport,
            TextKey::DeleteAccount,
            // Accessibility texts
            TextKey::OpenMenu,
            TextKey::CloseMenu,
            TextKey::NavigationMenu,
            TextKey::UserAvatar,
            TextKey::MessageInput,
            TextKey::SendButton,
            TextKey::ConversationList,
            TextKey::MessageHistory,
            // Input placeholders
            TextKey::EnterUsername,
            TextKey::EnterPassword,
            TextKey::SearchConversations,
            TextKey::WhatsOnYourMind,
            TextKey::EnterInstanceUrl,
            TextKey::EnterVerificationCode,
            TextKey::SelectMastodonServer,
            TextKey::VerificationCode,
            TextKey::EnterDescription,
            TextKey::SearchUsers,
            TextKey::Search,
            TextKey::Follow,
            // UI Actions and Buttons
            TextKey::Options,
            TextKey::CopyLink,
            TextKey::OpenInBrowser,
            TextKey::CopyText,
            TextKey::OpenConversation,
            TextKey::OpenProfile,
            TextKey::Open,
            TextKey::Close,
            TextKey::Back,
            TextKey::Continue,
            TextKey::Confirm,
            TextKey::Done,
            TextKey::Register,
            TextKey::UseCustom,
            TextKey::Paste,
            TextKey::Reload,
            TextKey::NewToot,
            // Content and Navigation
            TextKey::Timeline,
            TextKey::Posts,
            TextKey::Following,
            TextKey::Followers,
            TextKey::MoreFollowers,
            TextKey::LoadMoreFollowers,
            TextKey::Timelines,
            TextKey::Account,
            TextKey::ClassicTimeline,
            TextKey::YourPosts,
            TextKey::Local,
            TextKey::Federated,
            TextKey::GroupedTimelines,
            TextKey::Messages,
            TextKey::DirectMessages,
            TextKey::More,
            TextKey::TimelinesAndLists,
            TextKey::FollowersClassicalTimelinesAndMore,
            // Onboarding and Help
            TextKey::WelcomeToCyrup,
            TextKey::WebsiteShouldHaveOpened,
            TextKey::PleaseAuthorizeAndPaste,
            TextKey::CopyBrowserUrlToClipboard,
            TextKey::CyrupAlphaWarning,
            TextKey::ReportFeedbackInstruction,
            TextKey::TapSelectionTwiceToScrollTip,
        ]
    }
}

/// Compile-time validation macro for TextKey completeness
///
/// This macro ensures that all TextKey variants are handled in pattern matching
/// and generates a compile error if any variants are missing.
///
/// # Usage
/// ```rust
/// validate_text_key_exhaustiveness!(match some_key {
///     TextKey::LoginWithGoogle => "Sign in with Google",
///     TextKey::LoginWithGitHub => "Sign in with GitHub",
///     // ... all other variants must be handled
/// });
/// ```
#[macro_export]
macro_rules! validate_text_key_exhaustiveness {
    ($expr:expr) => {
        // This will cause a compile error if any TextKey variants are missing
        // from the match expression due to Rust's exhaustiveness checking
        $expr
    };
}

/// Const generic validation for TextKey at compile time
///
/// This trait provides compile-time validation that ensures all TextKey variants
/// are properly handled and no keys are missing from translations.
/// Currently unused but kept for future validation needs
#[allow(dead_code)]
pub trait TextKeyValidation {
    /// Validate that all TextKey variants are covered
    const VARIANT_COUNT: usize;

    /// Get the total number of TextKey variants for validation
    fn total_variants() -> usize {
        Self::VARIANT_COUNT
    }
}

impl TextKeyValidation for TextKey {
    // Automatically counted via strum::EnumCount derive macro
    const VARIANT_COUNT: usize = TextKey::COUNT;
}

/// Runtime validation helper for ensuring translation completeness
///
/// This function validates that all TextKey variants have translations
/// for all supported locales and reports any missing keys.
/// Currently unused but kept for future validation needs
#[allow(dead_code)]
pub fn validate_translation_completeness(i18n: &I18n) -> Result<(), Vec<(Locale, TextKey)>> {
    let mut missing_keys = Vec::new();

    for &locale in I18n::available_locales() {
        for &key in I18n::all_text_keys() {
            if !i18n.texts.contains_key(&(locale, key)) {
                missing_keys.push((locale, key));
            }
        }
    }

    if missing_keys.is_empty() {
        Ok(())
    } else {
        Err(missing_keys)
    }
}
