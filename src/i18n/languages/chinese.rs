//! Chinese language translations
//!
//! Chinese translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize Chinese text mappings
    pub fn init_chinese_texts(&mut self) {
        // Authentication texts
        self.add_text(Locale::Chinese, TextKey::LoginWithGoogle, "使用Google登录");
        self.add_text(Locale::Chinese, TextKey::LoginWithGitHub, "使用GitHub登录");
        self.add_text(Locale::Chinese, TextKey::SkipLogin, "跳过登录");
        self.add_text(Locale::Chinese, TextKey::SignOut, "退出登录");
        self.add_text(Locale::Chinese, TextKey::WelcomeBack, "欢迎回来！");
        self.add_text(
            Locale::Chinese,
            TextKey::AuthenticatingPleaseWait,
            "正在验证，请稍候...",
        );
        self.add_text(Locale::Chinese, TextKey::AuthenticationFailed, "验证失败");
        self.add_text(Locale::Chinese, TextKey::RetryAuthentication, "重试验证");
    }
}
