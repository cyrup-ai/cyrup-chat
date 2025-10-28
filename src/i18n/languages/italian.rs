//! Italian language translations
//!
//! Italian translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize Italian text mappings
    pub fn init_italian_texts(&mut self) {
        // Authentication texts
        self.add_text(
            Locale::Italian,
            TextKey::LoginWithGoogle,
            "Accedi con Google",
        );
        self.add_text(
            Locale::Italian,
            TextKey::LoginWithGitHub,
            "Accedi con GitHub",
        );
        self.add_text(Locale::Italian, TextKey::SkipLogin, "Salta accesso");
        self.add_text(Locale::Italian, TextKey::SignOut, "Esci");
        self.add_text(Locale::Italian, TextKey::WelcomeBack, "Bentornato!");
        self.add_text(
            Locale::Italian,
            TextKey::AuthenticatingPleaseWait,
            "Autenticazione in corso, attendere...",
        );
        self.add_text(
            Locale::Italian,
            TextKey::AuthenticationFailed,
            "Autenticazione fallita",
        );
        self.add_text(
            Locale::Italian,
            TextKey::RetryAuthentication,
            "Riprova autenticazione",
        );
    }
}
