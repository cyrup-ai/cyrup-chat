//! Spanish language translations
//!
//! Spanish translations for the i18n system.

use super::super::core::{I18n, Locale, TextKey};

impl I18n {
    /// Initialize Spanish text mappings
    pub fn init_spanish_texts(&mut self) {
        // Authentication texts
        self.add_text(
            Locale::Spanish,
            TextKey::LoginWithGoogle,
            "Iniciar sesión con Google",
        );
        self.add_text(
            Locale::Spanish,
            TextKey::LoginWithGitHub,
            "Iniciar sesión con GitHub",
        );
        self.add_text(Locale::Spanish, TextKey::SkipLogin, "Omitir inicio");
        self.add_text(Locale::Spanish, TextKey::SignOut, "Cerrar sesión");
        self.add_text(
            Locale::Spanish,
            TextKey::WelcomeBack,
            "¡Bienvenido de vuelta!",
        );
        self.add_text(
            Locale::Spanish,
            TextKey::AuthenticatingPleaseWait,
            "Autenticando, por favor espere...",
        );
        self.add_text(
            Locale::Spanish,
            TextKey::AuthenticationFailed,
            "Error de autenticación",
        );
        self.add_text(
            Locale::Spanish,
            TextKey::RetryAuthentication,
            "Reintentar autenticación",
        );

        // Main UI texts
        self.add_text(
            Locale::Spanish,
            TextKey::NewConversation,
            "Nueva conversación",
        );
        self.add_text(Locale::Spanish, TextKey::SendMessage, "Enviar mensaje");
        self.add_text(
            Locale::Spanish,
            TextKey::TypeYourMessage,
            "Escribe tu mensaje...",
        );
        self.add_text(
            Locale::Spanish,
            TextKey::ConversationHistory,
            "Historial de conversaciones",
        );
        self.add_text(Locale::Spanish, TextKey::Settings, "Configuración");
        self.add_text(Locale::Spanish, TextKey::Profile, "Perfil");
        self.add_text(Locale::Spanish, TextKey::About, "Acerca de");
    }
}
