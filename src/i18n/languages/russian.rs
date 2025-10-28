//! Russian language translations
//!
//! Russian translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize Russian text mappings
    pub fn init_russian_texts(&mut self) {
        // Authentication texts
        self.add_text(
            Locale::Russian,
            TextKey::LoginWithGoogle,
            "Войти через Google",
        );
        self.add_text(
            Locale::Russian,
            TextKey::LoginWithGitHub,
            "Войти через GitHub",
        );
        self.add_text(Locale::Russian, TextKey::SkipLogin, "Пропустить вход");
        self.add_text(Locale::Russian, TextKey::SignOut, "Выйти");
        self.add_text(
            Locale::Russian,
            TextKey::WelcomeBack,
            "Добро пожаловать обратно!",
        );
        self.add_text(
            Locale::Russian,
            TextKey::AuthenticatingPleaseWait,
            "Аутентификация, пожалуйста подождите...",
        );
        self.add_text(
            Locale::Russian,
            TextKey::AuthenticationFailed,
            "Ошибка аутентификации",
        );
        self.add_text(
            Locale::Russian,
            TextKey::RetryAuthentication,
            "Повторить аутентификацию",
        );
    }
}
