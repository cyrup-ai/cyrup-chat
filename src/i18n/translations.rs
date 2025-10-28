//! Translation data and initialization
//!
//! This module contains the initialization logic for the i18n system.
//! Individual language translations are organized in the languages/ subdirectory.

use super::core::I18n;

impl I18n {
    /// Initialize the i18n system with all text mappings
    pub fn init(&mut self) {
        self.init_english_texts();
        self.init_spanish_texts();
        self.init_french_texts();
        self.init_german_texts();
        self.init_japanese_texts();
        self.init_chinese_texts();
        self.init_korean_texts();
        self.init_portuguese_texts();
        self.init_italian_texts();
        self.init_russian_texts();
    }
}
