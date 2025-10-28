//! Korean language translations
//!
//! Korean translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize Korean text mappings
    pub fn init_korean_texts(&mut self) {
        // Authentication texts
        self.add_text(Locale::Korean, TextKey::LoginWithGoogle, "Google로 로그인");
        self.add_text(Locale::Korean, TextKey::LoginWithGitHub, "GitHub로 로그인");
        self.add_text(Locale::Korean, TextKey::SkipLogin, "로그인 건너뛰기");
        self.add_text(Locale::Korean, TextKey::SignOut, "로그아웃");
        self.add_text(
            Locale::Korean,
            TextKey::WelcomeBack,
            "다시 오신 것을 환영합니다!",
        );
        self.add_text(
            Locale::Korean,
            TextKey::AuthenticatingPleaseWait,
            "인증 중입니다. 잠시 기다려 주세요...",
        );
        self.add_text(
            Locale::Korean,
            TextKey::AuthenticationFailed,
            "인증에 실패했습니다",
        );
        self.add_text(Locale::Korean, TextKey::RetryAuthentication, "인증 재시도");
    }
}
