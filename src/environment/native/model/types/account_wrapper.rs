//! Account wrapper with pagination metadata

/// Account wrapper with pagination metadata
#[derive(Debug, Clone)]
pub struct Account {
    /// The underlying megalodon account
    pub account: megalodon::entities::Account,
    /// Pagination token for loading more accounts
    pub next: Option<String>,
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.account.id == other.account.id && self.next == other.next
    }
}

impl Eq for Account {}

impl Account {
    /// Create a new Account wrapper with pagination metadata
    #[inline(always)]
    pub fn new(account: megalodon::entities::Account, next: Option<String>) -> Self {
        Self { account, next }
    }
}

impl Default for Account {
    fn default() -> Self {
        Self {
            account: megalodon::entities::Account {
                id: String::new(),
                username: String::new(),
                acct: String::new(),
                display_name: String::new(),
                locked: false,
                discoverable: None,
                group: None,
                noindex: None,
                moved: None,
                suspended: None,
                limited: None,
                created_at: chrono::Utc::now(),
                followers_count: 0,
                following_count: 0,
                statuses_count: 0,
                note: String::new(),
                url: String::new(),
                avatar: String::new(),
                avatar_static: String::new(),
                header: String::new(),
                header_static: String::new(),
                emojis: Vec::new(),
                fields: Vec::new(),
                bot: false,
                source: None,
                role: None,
                mute_expires_at: None,
            },
            next: None,
        }
    }
}

impl From<megalodon::entities::Account> for Account {
    fn from(account: megalodon::entities::Account) -> Self {
        Self::new(account, None)
    }
}

impl std::ops::Deref for Account {
    type Target = megalodon::entities::Account;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl std::ops::DerefMut for Account {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}
