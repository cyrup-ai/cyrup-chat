//! Public API and global access functions
//!
//! This module provides the public interface for the i18n system,
//! including global singleton access and convenience functions.

use super::core::{I18n, I18nError, Locale, TextKey};
use std::sync::{Arc, OnceLock, RwLock};

/// Validate translation completeness at compile time
/// This macro ensures all TextKey variants have fallback translations
macro_rules! validate_fallback_completeness {
    () => {
        const _: () = {
            // Compile-time validation that all TextKey variants are handled
            // This will fail to compile if any variant is missing from get_fallback_translation
            use crate::i18n::core::TextKey;
            let _validate_completeness =
                |key: TextKey| -> &'static str { get_fallback_translation(key) };
        };
    };
}

// Compile-time validation that all fallback translations are complete
validate_fallback_completeness!();

/// Global i18n singleton with zero allocation access
static I18N_INSTANCE: OnceLock<Arc<RwLock<I18n>>> = OnceLock::new();

/// Initialize the global i18n system (call once at startup)
pub fn init_i18n() {
    I18N_INSTANCE.get_or_init(|| {
        let mut i18n = I18n::new();
        i18n.init();
        Arc::new(RwLock::new(i18n))
    });
}

/// Get text with zero allocation through global singleton - production safe
pub fn t(key: TextKey) -> Result<&'static str, I18nError> {
    let arc = I18N_INSTANCE.get().ok_or(I18nError::NotInitialized)?;

    let guard = arc.read().map_err(|_| I18nError::LockPoisoned)?;

    Ok(guard.text(key))
}

/// Get text for specific locale with zero allocation - production safe
pub fn t_locale(locale: Locale, key: TextKey) -> Result<&'static str, I18nError> {
    let arc = I18N_INSTANCE.get().ok_or(I18nError::NotInitialized)?;

    let guard = arc.read().map_err(|_| I18nError::LockPoisoned)?;

    Ok(guard.text_for_locale(locale, key))
}

/// Get text with fallback to English - zero allocation, panic-free
/// Provides complete fallback translations for all TextKey variants
pub fn t_safe(key: TextKey) -> &'static str {
    t(key).unwrap_or_else(|_| {
        log::warn!("I18n lookup failed for key {:?}, using fallback", key);
        get_fallback_translation(key)
    })
}

/// Zero-allocation fallback translation lookup with perfect hash map performance
/// Provides hardcoded English fallbacks for all TextKey variants
const fn get_fallback_translation(key: TextKey) -> &'static str {
    match key {
        // Authentication texts
        TextKey::LoginWithGoogle => "Sign in with Google",
        TextKey::LoginWithGitHub => "Sign in with GitHub",
        TextKey::SkipLogin => "Skip login",
        TextKey::SignOut => "Sign out",
        TextKey::WelcomeBack => "Welcome back!",
        TextKey::AuthenticatingPleaseWait => "Authenticating, please wait...",
        TextKey::AuthenticationFailed => "Authentication failed",
        TextKey::RetryAuthentication => "Retry authentication",

        // Main UI texts
        TextKey::NewConversation => "New conversation",
        TextKey::SendMessage => "Send message",
        TextKey::TypeYourMessage => "Type your message...",
        TextKey::ConversationHistory => "Conversation history",
        TextKey::Settings => "Settings",
        TextKey::Profile => "Profile",
        TextKey::About => "About",

        // Conversation texts
        TextKey::NewChat => "New chat",
        TextKey::DeleteConversation => "Delete conversation",
        TextKey::ShareConversation => "Share conversation",
        TextKey::CopyMessage => "Copy message",
        TextKey::RegenerateResponse => "Regenerate response",
        TextKey::StopGeneration => "Stop generation",

        // Status texts
        TextKey::Connecting => "Connecting...",
        TextKey::Connected => "Connected",
        TextKey::Disconnected => "Disconnected",
        TextKey::SendingMessage => "Sending message...",
        TextKey::ReceivingResponse => "Receiving response...",
        TextKey::MessageSent => "Message sent",
        TextKey::MessageFailed => "Message failed",

        // Error texts
        TextKey::NetworkError => "Network error",
        TextKey::AuthenticationError => "Authentication error",
        TextKey::UnexpectedError => "Unexpected error",
        TextKey::PleaseRetry => "Please retry",
        TextKey::ContactSupport => "Contact support if the problem persists",

        // Settings texts
        TextKey::Language => "Language",
        TextKey::Theme => "Theme",
        TextKey::Notifications => "Notifications",
        TextKey::Privacy => "Privacy",
        TextKey::DataExport => "Data export",
        TextKey::DeleteAccount => "Delete account",

        // Accessibility texts
        TextKey::OpenMenu => "Open menu",
        TextKey::CloseMenu => "Close menu",
        TextKey::NavigationMenu => "Navigation menu",
        TextKey::UserAvatar => "User avatar",
        TextKey::MessageInput => "Message input",
        TextKey::SendButton => "Send button",
        TextKey::ConversationList => "Conversation list",
        TextKey::MessageHistory => "Message history",

        // Input placeholders
        TextKey::EnterUsername => "Enter username",
        TextKey::EnterPassword => "Enter password",
        TextKey::SearchConversations => "Search conversations...",
        TextKey::WhatsOnYourMind => "What's on your mind?",
        TextKey::EnterInstanceUrl => "Enter instance URL",
        TextKey::EnterVerificationCode => "Enter verification code",
        TextKey::SelectMastodonServer => "Select Mastodon Server",
        TextKey::VerificationCode => "Verification Code",
        TextKey::EnterDescription => "Enter description",
        TextKey::SearchUsers => "Search users...",
        TextKey::Search => "Search",
        TextKey::Follow => "Follow",

        // UI Actions and Buttons
        TextKey::Options => "Options",
        TextKey::CopyLink => "Copy Link",
        TextKey::OpenInBrowser => "Open in Browser",
        TextKey::CopyText => "Copy Text",
        TextKey::OpenConversation => "Open Conversation",
        TextKey::OpenProfile => "Open Profile",
        TextKey::Open => "Open",
        TextKey::Close => "Close",
        TextKey::Back => "Back",
        TextKey::Continue => "Continue",
        TextKey::Confirm => "Confirm",
        TextKey::Done => "Done",
        TextKey::Register => "Register",
        TextKey::UseCustom => "Use Custom",
        TextKey::Paste => "Paste",
        TextKey::Reload => "Reload",
        TextKey::NewToot => "New Toot",

        // Content and Navigation
        TextKey::Timeline => "Timeline",
        TextKey::Posts => "Posts",
        TextKey::Following => "Following",
        TextKey::Followers => "Followers",
        TextKey::MoreFollowers => "More Followers",
        TextKey::LoadMoreFollowers => "Load more followers",
        TextKey::Timelines => "Timelines",
        TextKey::Account => "Account",
        TextKey::ClassicTimeline => "Classic Timeline",
        TextKey::YourPosts => "Your Posts",
        TextKey::Local => "Local",
        TextKey::Federated => "Federated",
        TextKey::GroupedTimelines => "Grouped Timelines",
        TextKey::Messages => "Messages",
        TextKey::DirectMessages => "Direct Messages",
        TextKey::More => "More",
        TextKey::TimelinesAndLists => "Timelines & Lists",
        TextKey::FollowersClassicalTimelinesAndMore => "Followers, Classical Timelines & More",

        // Onboarding and Help
        TextKey::WelcomeToCyrup => "Welcome to CYRUP",
        TextKey::WebsiteShouldHaveOpened => "A website should just have opened in your browser.",
        TextKey::PleaseAuthorizeAndPaste => {
            "Please authorize CYRUP and then copy & paste the code into the box below."
        }
        TextKey::CopyBrowserUrlToClipboard => "Copy the browser URL to the clipboard",
        TextKey::CyrupAlphaWarning => {
            "CYRUP is still a very early alpha. Expect bugs and missing features."
        }
        TextKey::ReportFeedbackInstruction => {
            "You can report feedback by sending me a private message."
        }
        TextKey::TapSelectionTwiceToScrollTip => {
            "One Tip: Tap a selection in the left column twice, to scroll to the timeline bottom"
        }
    }
}

/// Set current locale through global singleton with runtime mutability
pub fn set_locale(locale: Locale) -> Result<(), I18nError> {
    let arc = I18N_INSTANCE.get().ok_or(I18nError::NotInitialized)?;

    let mut guard = arc.write().map_err(|_| I18nError::LockPoisoned)?;

    guard.set_locale(locale);
    Ok(())
}

/// Get current locale from global singleton
pub fn current_locale() -> Locale {
    I18N_INSTANCE
        .get()
        .and_then(|arc| arc.read().ok())
        .map(|guard| guard.current_locale())
        .unwrap_or(Locale::English)
}

/// Runtime validation for translation system integrity
/// Currently unused but kept for future validation needs
#[allow(dead_code)]
pub fn validate_translation_system() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    // Validate that i18n system is initialized
    if I18N_INSTANCE.get().is_none() {
        errors.push("I18n system not initialized".to_string());
    }

    // Validate that all TextKey variants have fallback translations
    for &key in crate::i18n::core::I18n::all_text_keys() {
        let fallback = get_fallback_translation(key);
        if fallback.is_empty() || fallback == "[Text not available]" {
            errors.push(format!("Missing fallback translation for {:?}", key));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Convenience macro for text lookup with zero allocation
#[macro_export]
macro_rules! i18n {
    ($key:expr) => {
        $crate::i18n::api::t($key)
    };
    ($locale:expr, $key:expr) => {
        $crate::i18n::api::t_locale($locale, $key)
    };
}

/// Safe convenience macro that never panics
#[macro_export]
macro_rules! i18n_safe {
    ($key:expr) => {
        $crate::i18n::api::t_safe($key)
    };
}
