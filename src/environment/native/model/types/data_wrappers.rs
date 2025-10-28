//! Data wrappers for megalodon types with Eq/PartialEq implementations

/// Wrapper for megalodon AppData with Eq/PartialEq
#[derive(Debug, Clone)]
pub struct AppData(megalodon::oauth::AppData);

impl PartialEq for AppData {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl Eq for AppData {}

impl From<megalodon::oauth::AppData> for AppData {
    fn from(value: megalodon::oauth::AppData) -> Self {
        AppData(value)
    }
}

impl std::ops::Deref for AppData {
    type Target = megalodon::oauth::AppData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Wrapper for megalodon TokenData with Eq/PartialEq
#[derive(Debug, Clone)]
pub struct TokenData(megalodon::oauth::TokenData);

impl PartialEq for TokenData {
    fn eq(&self, other: &Self) -> bool {
        self.0.access_token == other.0.access_token
    }
}

impl Eq for TokenData {}

impl From<megalodon::oauth::TokenData> for TokenData {
    fn from(value: megalodon::oauth::TokenData) -> Self {
        TokenData(value)
    }
}

impl std::ops::Deref for TokenData {
    type Target = megalodon::oauth::TokenData;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
