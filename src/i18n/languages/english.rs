//! English language translations
//!
//! Base language translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize English text mappings (base language)
    pub fn init_english_texts(&mut self) {
        // Authentication texts
        self.add_text(
            Locale::English,
            TextKey::LoginWithGoogle,
            "Sign in with Google",
        );
        self.add_text(
            Locale::English,
            TextKey::LoginWithGitHub,
            "Sign in with GitHub",
        );
        self.add_text(Locale::English, TextKey::SkipLogin, "Skip Login");
        self.add_text(Locale::English, TextKey::SignOut, "Sign Out");
        self.add_text(Locale::English, TextKey::WelcomeBack, "Welcome back!");
        self.add_text(
            Locale::English,
            TextKey::AuthenticatingPleaseWait,
            "Authenticating, please wait...",
        );
        self.add_text(
            Locale::English,
            TextKey::AuthenticationFailed,
            "Authentication failed",
        );
        self.add_text(
            Locale::English,
            TextKey::RetryAuthentication,
            "Retry Authentication",
        );

        // Main UI texts
        self.add_text(
            Locale::English,
            TextKey::NewConversation,
            "New Conversation",
        );
        self.add_text(Locale::English, TextKey::SendMessage, "Send Message");
        self.add_text(
            Locale::English,
            TextKey::TypeYourMessage,
            "Type your message...",
        );
        self.add_text(
            Locale::English,
            TextKey::ConversationHistory,
            "Conversation History",
        );
        self.add_text(Locale::English, TextKey::Settings, "Settings");
        self.add_text(Locale::English, TextKey::Profile, "Profile");
        self.add_text(Locale::English, TextKey::About, "About");

        // Conversation texts
        self.add_text(Locale::English, TextKey::NewChat, "New Chat");
        self.add_text(
            Locale::English,
            TextKey::DeleteConversation,
            "Delete Conversation",
        );
        self.add_text(
            Locale::English,
            TextKey::ShareConversation,
            "Share Conversation",
        );
        self.add_text(Locale::English, TextKey::CopyMessage, "Copy Message");
        self.add_text(
            Locale::English,
            TextKey::RegenerateResponse,
            "Regenerate Response",
        );
        self.add_text(Locale::English, TextKey::StopGeneration, "Stop Generation");

        // Status texts
        self.add_text(Locale::English, TextKey::Connecting, "Connecting...");
        self.add_text(Locale::English, TextKey::Connected, "Connected");
        self.add_text(Locale::English, TextKey::Disconnected, "Disconnected");
        self.add_text(
            Locale::English,
            TextKey::SendingMessage,
            "Sending message...",
        );
        self.add_text(
            Locale::English,
            TextKey::ReceivingResponse,
            "Receiving response...",
        );
        self.add_text(Locale::English, TextKey::MessageSent, "Message sent");
        self.add_text(Locale::English, TextKey::MessageFailed, "Message failed");

        // Settings texts
        self.add_text(Locale::English, TextKey::Language, "Language");
        self.add_text(Locale::English, TextKey::Theme, "Theme");
        self.add_text(Locale::English, TextKey::Notifications, "Notifications");
        self.add_text(Locale::English, TextKey::Privacy, "Privacy");
        self.add_text(Locale::English, TextKey::DataExport, "Data Export");
        self.add_text(Locale::English, TextKey::DeleteAccount, "Delete Account");

        // Accessibility texts
        self.add_text(Locale::English, TextKey::OpenMenu, "Open Menu");
        self.add_text(Locale::English, TextKey::CloseMenu, "Close Menu");
        self.add_text(Locale::English, TextKey::NavigationMenu, "Navigation Menu");
        self.add_text(Locale::English, TextKey::UserAvatar, "User Avatar");
        self.add_text(Locale::English, TextKey::MessageInput, "Message Input");
        self.add_text(Locale::English, TextKey::SendButton, "Send Button");
        self.add_text(
            Locale::English,
            TextKey::ConversationList,
            "Conversation List",
        );
        self.add_text(Locale::English, TextKey::MessageHistory, "Message History");

        // Input placeholders
        self.add_text(
            Locale::English,
            TextKey::SearchConversations,
            "Search conversations...",
        );
        self.add_text(
            Locale::English,
            TextKey::WhatsOnYourMind,
            "What's on your mind?",
        );
        self.add_text(
            Locale::English,
            TextKey::EnterUsername,
            "Enter your username",
        );
        self.add_text(
            Locale::English,
            TextKey::EnterPassword,
            "Enter your password",
        );
        self.add_text(
            Locale::English,
            TextKey::EnterVerificationCode,
            "Enter verification code",
        );
        self.add_text(
            Locale::English,
            TextKey::SelectMastodonServer,
            "Select or Enter a Mastodon Server",
        );
        self.add_text(Locale::English, TextKey::VerificationCode, "Code");
        self.add_text(
            Locale::English,
            TextKey::EnterDescription,
            "Enter Description…",
        );
        self.add_text(Locale::English, TextKey::SearchUsers, "Search users...");
        self.add_text(Locale::English, TextKey::Search, "Search");
        self.add_text(Locale::English, TextKey::Follow, "Follow");
        self.add_text(
            Locale::English,
            TextKey::EnterInstanceUrl,
            "Enter instance URL",
        );

        // UI Actions and Buttons
        self.add_text(Locale::English, TextKey::Options, "Options");
        self.add_text(Locale::English, TextKey::CopyLink, "Copy Link");
        self.add_text(Locale::English, TextKey::OpenInBrowser, "Open in Browser");
        self.add_text(Locale::English, TextKey::CopyText, "Copy Text");
        self.add_text(
            Locale::English,
            TextKey::OpenConversation,
            "Open Conversation",
        );
        self.add_text(Locale::English, TextKey::OpenProfile, "Open Profile");
        self.add_text(Locale::English, TextKey::Open, "Open");
        self.add_text(Locale::English, TextKey::Close, "Close");
        self.add_text(Locale::English, TextKey::Back, "Back");
        self.add_text(Locale::English, TextKey::Continue, "Continue");
        self.add_text(Locale::English, TextKey::Confirm, "Confirm");
        self.add_text(Locale::English, TextKey::Done, "Done");
        self.add_text(Locale::English, TextKey::Register, "Register");
        self.add_text(Locale::English, TextKey::UseCustom, "Use Custom");
        self.add_text(Locale::English, TextKey::Paste, "Paste");
        self.add_text(Locale::English, TextKey::Reload, "Reload");
        self.add_text(Locale::English, TextKey::NewToot, "New Toot");

        // Content and Navigation
        self.add_text(Locale::English, TextKey::Timeline, "Timeline");
        self.add_text(Locale::English, TextKey::Posts, "Posts");
        self.add_text(Locale::English, TextKey::Following, "Following");
        self.add_text(Locale::English, TextKey::Followers, "Followers");
        self.add_text(Locale::English, TextKey::MoreFollowers, "More Followers");
        self.add_text(
            Locale::English,
            TextKey::LoadMoreFollowers,
            "Load more followers",
        );
        self.add_text(Locale::English, TextKey::Timelines, "Timelines");
        self.add_text(Locale::English, TextKey::Account, "Account");
        self.add_text(
            Locale::English,
            TextKey::ClassicTimeline,
            "Classic Timeline",
        );
        self.add_text(Locale::English, TextKey::YourPosts, "Your Posts");
        self.add_text(Locale::English, TextKey::Local, "Local");
        self.add_text(Locale::English, TextKey::Federated, "Federated");
        self.add_text(
            Locale::English,
            TextKey::GroupedTimelines,
            "Grouped Timelines",
        );
        self.add_text(Locale::English, TextKey::Messages, "Messages");
        self.add_text(Locale::English, TextKey::DirectMessages, "Direct Messages");
        self.add_text(Locale::English, TextKey::More, "More");
        self.add_text(
            Locale::English,
            TextKey::TimelinesAndLists,
            "Timelines & Lists",
        );
        self.add_text(
            Locale::English,
            TextKey::FollowersClassicalTimelinesAndMore,
            "Followers, Classical Timelines & More",
        );

        // Onboarding and Help
        self.add_text(Locale::English, TextKey::WelcomeToCyrup, "Welcome to CYRUP");
        self.add_text(
            Locale::English,
            TextKey::WebsiteShouldHaveOpened,
            "A website should just have opened in your browser.",
        );
        self.add_text(
            Locale::English,
            TextKey::PleaseAuthorizeAndPaste,
            "Please authorize CYRUP and then copy & paste the code into the box below.",
        );
        self.add_text(
            Locale::English,
            TextKey::CopyBrowserUrlToClipboard,
            "Copy the browser URL to the clipboard",
        );
        self.add_text(
            Locale::English,
            TextKey::CyrupAlphaWarning,
            "CYRUP is still a very early alpha. Expect bugs and missing features.",
        );
        self.add_text(
            Locale::English,
            TextKey::ReportFeedbackInstruction,
            "You can report feedback by sending me a private message.",
        );
        self.add_text(
            Locale::English,
            TextKey::TapSelectionTwiceToScrollTip,
            "One Tip: Tap a selection in the left column twice, to scroll to the timeline bottom",
        );

        // Error messages
        self.add_text(
            Locale::English,
            TextKey::NetworkError,
            "Network connection error",
        );
        self.add_text(
            Locale::English,
            TextKey::AuthenticationError,
            "Authentication error occurred",
        );
        self.add_text(
            Locale::English,
            TextKey::UnexpectedError,
            "An unexpected error occurred",
        );
        self.add_text(Locale::English, TextKey::PleaseRetry, "Please try again");
        self.add_text(
            Locale::English,
            TextKey::ContactSupport,
            "Contact support if the problem persists",
        );
    }

    /// Validate that all TextKey variants have English translations
    /// This ensures compile-time completeness validation
    pub fn validate_english_completeness(&self) -> Result<(), Vec<TextKey>> {
        let missing_keys: Vec<TextKey> = Self::all_text_keys()
            .iter()
            .filter(|&&key| !self.texts.contains_key(&(Locale::English, key)))
            .copied()
            .collect();

        if missing_keys.is_empty() {
            Ok(())
        } else {
            Err(missing_keys)
        }
    }
}
