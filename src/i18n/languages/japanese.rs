//! Japanese language translations
//!
//! Japanese translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize Japanese text mappings
    pub fn init_japanese_texts(&mut self) {
        // Authentication texts
        self.add_text(
            Locale::Japanese,
            TextKey::LoginWithGoogle,
            "Googleでサインイン",
        );
        self.add_text(
            Locale::Japanese,
            TextKey::LoginWithGitHub,
            "GitHubでサインイン",
        );
        self.add_text(Locale::Japanese, TextKey::SkipLogin, "ログインをスキップ");
        self.add_text(Locale::Japanese, TextKey::SignOut, "サインアウト");
        self.add_text(Locale::Japanese, TextKey::WelcomeBack, "おかえりなさい！");
        self.add_text(
            Locale::Japanese,
            TextKey::AuthenticatingPleaseWait,
            "認証中です。お待ちください...",
        );
        self.add_text(
            Locale::Japanese,
            TextKey::AuthenticationFailed,
            "認証に失敗しました",
        );
        self.add_text(
            Locale::Japanese,
            TextKey::RetryAuthentication,
            "認証を再試行",
        );
    }
}
