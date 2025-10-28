//! German language translations
//!
//! German translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize German text mappings
    pub fn init_german_texts(&mut self) {
        // Authentication texts
        self.add_text(
            Locale::German,
            TextKey::LoginWithGoogle,
            "Mit Google anmelden",
        );
        self.add_text(
            Locale::German,
            TextKey::LoginWithGitHub,
            "Mit GitHub anmelden",
        );
        self.add_text(Locale::German, TextKey::SkipLogin, "Anmeldung überspringen");
        self.add_text(Locale::German, TextKey::SignOut, "Abmelden");
        self.add_text(Locale::German, TextKey::WelcomeBack, "Willkommen zurück!");
        self.add_text(
            Locale::German,
            TextKey::AuthenticatingPleaseWait,
            "Authentifizierung läuft, bitte warten...",
        );
        self.add_text(
            Locale::German,
            TextKey::AuthenticationFailed,
            "Authentifizierung fehlgeschlagen",
        );
        self.add_text(
            Locale::German,
            TextKey::RetryAuthentication,
            "Authentifizierung wiederholen",
        );
    }
}
