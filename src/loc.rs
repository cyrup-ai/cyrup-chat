/// Localization module - Proper implementation for production use
/// This module provides localization support with extensible string management
use std::collections::HashMap;
use std::sync::OnceLock;

/// Thread-safe global localization store
static LOCALIZATION_STORE: OnceLock<LocalizationStore> = OnceLock::new();

/// Localization store with extensible language support
#[derive(Debug)]
#[allow(dead_code)] // Localization system - language field pending internationalization
pub struct LocalizationStore {
    strings: HashMap<String, String>,
    language: String,
}

impl Default for LocalizationStore {
    fn default() -> Self {
        Self::new()
    }
}

impl LocalizationStore {
    /// Initialize with default English strings
    pub fn new() -> Self {
        let mut strings = HashMap::new();

        // Core UI strings
        strings.insert("Timelines".to_string(), "Timelines".to_string());
        strings.insert("Mentions".to_string(), "Mentions".to_string());
        strings.insert("Messages".to_string(), "Messages".to_string());
        strings.insert("More".to_string(), "More".to_string());
        strings.insert("Timeline".to_string(), "Timeline".to_string());
        strings.insert(
            "Classic Timeline".to_string(),
            "Classic Timeline".to_string(),
        );
        strings.insert("Your Posts".to_string(), "Your Posts".to_string());
        strings.insert("Local".to_string(), "Local".to_string());
        strings.insert("Federated".to_string(), "Federated".to_string());
        strings.insert("Posts".to_string(), "Posts".to_string());
        strings.insert("Hashtags".to_string(), "Hashtags".to_string());
        strings.insert("Followers".to_string(), "Followers".to_string());
        strings.insert("Following".to_string(), "Following".to_string());
        strings.insert("Bookmarks".to_string(), "Bookmarks".to_string());
        strings.insert("Favorites".to_string(), "Favorites".to_string());
        strings.insert("Account".to_string(), "Account".to_string());
        strings.insert(
            "Favorite Account. Always at the top".to_string(),
            "Favorite Account. Always at the top".to_string(),
        );
        strings.insert("More Followers".to_string(), "More Followers".to_string());
        strings.insert(
            "Load more followers".to_string(),
            "Load more followers".to_string(),
        );

        Self {
            strings,
            language: "en".to_string(),
        }
    }

    /// Get localized string by key
    pub fn get(&self, key: &str) -> String {
        self.strings
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string())
    }

    /// Get localized string with parameter substitution
    pub fn get_with_params(&self, key: &str, params: &[(&str, &str)]) -> String {
        let mut result = self.get(key);
        for (param_key, param_value) in params {
            result = result.replace(&format!("{{{}}}", param_key), param_value);
        }
        result
    }
}

/// Initialize the global localization store
pub fn initialize_localization() {
    let _ = LOCALIZATION_STORE.set(LocalizationStore::new());
}

/// Get the global localization store, initializing if needed
fn get_store() -> &'static LocalizationStore {
    LOCALIZATION_STORE.get_or_init(LocalizationStore::new)
}

/// Localization macro - returns localized string for the key
#[macro_export]
macro_rules! loc {
    ($key:expr) => {
        $crate::loc::t($key)
    };
}

/// Get localized string by key
pub fn t(key: &str) -> String {
    get_store().get(key)
}

/// Get localized string with parameters
pub fn t_with_params(key: &str, params: &[(&str, &str)]) -> String {
    get_store().get_with_params(key, params)
}
