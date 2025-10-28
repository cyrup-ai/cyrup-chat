//! Portuguese language translations
//!
//! Portuguese translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize Portuguese text mappings
    pub fn init_portuguese_texts(&mut self) {
        // Authentication texts
        self.add_text(
            Locale::Portuguese,
            TextKey::LoginWithGoogle,
            "Entrar com Google",
        );
        self.add_text(
            Locale::Portuguese,
            TextKey::LoginWithGitHub,
            "Entrar com GitHub",
        );
        self.add_text(Locale::Portuguese, TextKey::SkipLogin, "Pular login");
        self.add_text(Locale::Portuguese, TextKey::SignOut, "Sair");
        self.add_text(
            Locale::Portuguese,
            TextKey::WelcomeBack,
            "Bem-vindo de volta!",
        );
        self.add_text(
            Locale::Portuguese,
            TextKey::AuthenticatingPleaseWait,
            "Autenticando, por favor aguarde...",
        );
        self.add_text(
            Locale::Portuguese,
            TextKey::AuthenticationFailed,
            "Falha na autenticação",
        );
        self.add_text(
            Locale::Portuguese,
            TextKey::RetryAuthentication,
            "Tentar autenticação novamente",
        );
    }
}
