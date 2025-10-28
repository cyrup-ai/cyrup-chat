//! French language translations
//!
//! French translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize French text mappings
    pub fn init_french_texts(&mut self) {
        // Authentication texts
        self.add_text(
            Locale::French,
            TextKey::LoginWithGoogle,
            "Se connecter avec Google",
        );
        self.add_text(
            Locale::French,
            TextKey::LoginWithGitHub,
            "Se connecter avec GitHub",
        );
        self.add_text(Locale::French, TextKey::SkipLogin, "Ignorer la connexion");
        self.add_text(Locale::French, TextKey::SignOut, "Se déconnecter");
        self.add_text(
            Locale::French,
            TextKey::WelcomeBack,
            "Content de vous revoir !",
        );
        self.add_text(
            Locale::French,
            TextKey::AuthenticatingPleaseWait,
            "Authentification en cours, veuillez patienter...",
        );
        self.add_text(
            Locale::French,
            TextKey::AuthenticationFailed,
            "Échec de l'authentification",
        );
        self.add_text(
            Locale::French,
            TextKey::RetryAuthentication,
            "Réessayer l'authentification",
        );
    }
}
