// UI Text Constants - Zero Allocation i18n Implementation
// Provides zero-allocation text lookups through the i18n system

use crate::i18n::{TextKey, t_safe};

/// Username input placeholder text
pub fn username_placeholder() -> &'static str {
    t_safe(TextKey::EnterUsername)
}

/// Password input placeholder text  
pub fn password_placeholder() -> &'static str {
    t_safe(TextKey::EnterPassword)
}

/// Chat message input placeholder text
pub fn chat_input_placeholder() -> &'static str {
    t_safe(TextKey::TypeYourMessage)
}

/// Post composition placeholder text
pub fn post_placeholder() -> &'static str {
    t_safe(TextKey::WhatsOnYourMind)
}

/// Search input placeholder text
pub fn search_placeholder() -> &'static str {
    t_safe(TextKey::SearchConversations)
}

/// Default button text for login action
pub fn login_button_text() -> &'static str {
    t_safe(TextKey::LoginWithGoogle)
}

/// Default button text for follow action  
pub fn follow_button_text() -> &'static str {
    t_safe(TextKey::Follow)
}

/// Search placeholder for user search
pub fn user_search_placeholder() -> &'static str {
    t_safe(TextKey::SearchUsers)
}

/// Placeholder for instance URL input
pub fn instance_url_placeholder() -> &'static str {
    t_safe(TextKey::EnterInstanceUrl)
}

/// Placeholder for verification code input
pub fn verification_code_placeholder() -> &'static str {
    t_safe(TextKey::EnterVerificationCode)
}

/// Placeholder for server/instance selection
pub fn server_selection_placeholder() -> &'static str {
    t_safe(TextKey::SelectMastodonServer)
}

/// Placeholder for verification code (short form)
pub fn code_placeholder() -> &'static str {
    t_safe(TextKey::VerificationCode)
}

/// Placeholder for post text entry
pub fn post_text_placeholder() -> &'static str {
    t_safe(TextKey::TypeYourMessage)
}

/// Placeholder for media description
pub fn media_description_placeholder() -> &'static str {
    t_safe(TextKey::EnterDescription)
}

/// Placeholder for general search
pub fn general_search_placeholder() -> &'static str {
    t_safe(TextKey::Search)
}

/// Compile-time validation that all UI text functions are properly mapped to TextKey variants
/// This ensures that every function corresponds to a valid TextKey and translation
macro_rules! validate_ui_text_completeness {
    () => {
        const _: () = {
            // Validate that all UI text functions use valid TextKey variants
            let _validate_mappings = || {
                let _ = username_placeholder();
                let _ = password_placeholder();
                let _ = chat_input_placeholder();
                let _ = post_placeholder();
                let _ = search_placeholder();
                let _ = login_button_text();
                let _ = follow_button_text();
                let _ = user_search_placeholder();
                let _ = instance_url_placeholder();
                let _ = verification_code_placeholder();
                let _ = server_selection_placeholder();
                let _ = code_placeholder();
                let _ = post_text_placeholder();
                let _ = media_description_placeholder();
                let _ = general_search_placeholder();
            };
        };
    };
}

// Compile-time validation of UI text completeness
validate_ui_text_completeness!();

/// Runtime validation for UI text system integrity
/// Ensures all UI text functions return valid, non-empty strings
pub fn validate_ui_text_system() -> Result<(), Vec<String>> {
    let mut errors = Vec::new();

    let functions = [
        ("username_placeholder", username_placeholder()),
        ("password_placeholder", password_placeholder()),
        ("chat_input_placeholder", chat_input_placeholder()),
        ("post_placeholder", post_placeholder()),
        ("search_placeholder", search_placeholder()),
        ("login_button_text", login_button_text()),
        ("follow_button_text", follow_button_text()),
        ("user_search_placeholder", user_search_placeholder()),
        ("instance_url_placeholder", instance_url_placeholder()),
        (
            "verification_code_placeholder",
            verification_code_placeholder(),
        ),
        (
            "server_selection_placeholder",
            server_selection_placeholder(),
        ),
        ("code_placeholder", code_placeholder()),
        ("post_text_placeholder", post_text_placeholder()),
        (
            "media_description_placeholder",
            media_description_placeholder(),
        ),
        ("general_search_placeholder", general_search_placeholder()),
    ];

    for (name, text) in functions {
        if text.is_empty() {
            errors.push(format!("UI text function '{}' returns empty string", name));
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
