// Internationalization tests extracted from src/i18n/mod.rs
// These tests verify localization functionality and text retrieval

use cyrup_chat::i18n::{init_i18n, t, t_locale, A11y, I18n, Locale, TextKey};

#[test]
fn test_locale_codes() {
    assert_eq!(Locale::English.code(), "en");
    assert_eq!(Locale::Spanish.code(), "es");
    assert_eq!(Locale::Japanese.code(), "ja");
}

#[test]
fn test_locale_names() {
    assert_eq!(Locale::English.name(), "English");
    assert_eq!(Locale::Spanish.name(), "Español");
    assert_eq!(Locale::Japanese.name(), "日本語");
}

#[test]
fn test_i18n_init_and_text_lookup() {
    init_i18n();
    
    // Test English texts
    assert_eq!(t(TextKey::LoginWithGoogle), "Sign in with Google");
    assert_eq!(t(TextKey::NewConversation), "New Conversation");
    
    // Test Spanish fallback through direct function
    assert_eq!(t_locale(Locale::Spanish, TextKey::LoginWithGoogle), "Iniciar sesión con Google");
}

#[test]
fn test_fallback_to_english() {
    init_i18n();
    
    // Test that incomplete translations fall back to English
    assert_eq!(t_locale(Locale::French, TextKey::LoginWithGoogle), "Sign in with Google");
}

#[test]
fn test_available_locales() {
    let locales = I18n::available_locales();
    assert_eq!(locales.len(), 10);
    assert!(locales.contains(&Locale::English));
    assert!(locales.contains(&Locale::Spanish));
    assert!(locales.contains(&Locale::Japanese));
}

#[test]
fn test_accessibility_helpers() {
    init_i18n();
    
    assert_eq!(A11y::aria_label(TextKey::OpenMenu), "Open menu");
    assert_eq!(A11y::button_name(TextKey::SendMessage), "Send Message");
    assert_eq!(A11y::input_label(TextKey::TypeYourMessage), "Type your message...");
}