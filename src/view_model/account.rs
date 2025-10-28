//! Account view models and related functionality
//!
//! This module provides account-related view models including user profiles,
//! account fields, and account updates with zero-allocation patterns.

use super::media::replace_emoji;
use super::types::AccountId;
use crate::helper::{HtmlItem, clean_html, format_number};
use chrono::{DateTime, Utc};
use megalodon::entities::Account;
use serde::{Deserialize, Serialize};

/// Comprehensive account view model with formatted display fields
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct AccountViewModel {
    pub id: AccountId,
    pub image: String,
    pub image_header: String,
    pub username: String,
    pub display_name: String,
    pub display_name_html: String,
    pub acct: String,
    pub note_plain: String,
    pub note_html: Vec<HtmlItem>,
    pub joined_human: String,
    pub joined_full: String,
    pub joined: DateTime<Utc>,
    pub url: String,
    pub followers: u32,
    pub followers_str: String,
    pub following: u32,
    pub following_str: String,
    pub statuses: u32,
    pub statuses_str: String,
    pub header: String,
    pub fields: Vec<AccountField>,
    pub locked: bool,
    pub bot: bool,
}

impl PartialEq for AccountViewModel {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for AccountViewModel {}

impl AccountViewModel {
    /// Create AccountViewModel from Account with optimized processing
    ///
    /// # Arguments
    /// * `account` - Raw account data from the API
    ///
    /// # Returns
    /// * `AccountViewModel` - Processed view model with formatted fields
    ///
    /// # Performance
    /// Uses efficient string processing and conditional emoji replacement
    #[inline(always)]
    pub fn new(account: &Account) -> Self {
        let (h, f) = crate::environment::platform::format_datetime(&account.created_at);
        let (mut plain, html) = match replace_emoji(&account.note, &account.emojis) {
            Some(n) => clean_html(&n),
            None => clean_html(&account.note),
        };

        // Efficient text truncation with ellipsis
        if plain.len() > 140 {
            plain = plain.chars().take(140).collect();
            plain.push('…');
        }

        let fields: Vec<_> = account
            .fields
            .iter()
            .map(|f| AccountField::new(&f.name, &f.value, f.verified_at))
            .collect();

        let display_name_html = match replace_emoji(&account.display_name, &account.emojis) {
            Some(n) => n,
            None => account.display_name.clone(),
        };

        Self {
            id: AccountId(account.id.clone()),
            image: account.avatar_static.clone(),
            image_header: account.header_static.clone(),
            username: account.username.clone(),
            display_name: account.display_name.clone(),
            display_name_html,
            acct: account.acct.clone(),
            note_plain: plain,
            note_html: html,
            joined_human: h,
            joined_full: f,
            joined: account.created_at,
            url: account.url.clone(),
            followers: account.followers_count as u32,
            followers_str: format_number(account.followers_count as i64),
            following: account.following_count,
            following_str: format_number(account.following_count as i64),
            statuses: account.statuses_count,
            statuses_str: format_number(account.statuses_count as i64),
            header: account.header_static.clone(),
            fields,
            locked: account.locked,
            bot: account.bot,
        }
    }
}

/// Account profile field with link parsing and verification status
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct AccountField {
    /// The original name
    pub name: String,
    /// The original value
    pub value: String,
    /// A parsed value obtained by stripping HTML (or value)
    pub value_parsed: String,
    /// If the field value contains a link, this is the parsed link
    pub link: Option<url::Url>,
    pub verified_at: Option<DateTime<Utc>>,
}

impl AccountField {
    /// Parse a Field with intelligent link extraction
    ///
    /// # Arguments
    /// * `name` - Field name
    /// * `value` - Field value (may contain HTML)
    /// * `verified_at` - Optional verification timestamp
    ///
    /// # Returns
    /// * `AccountField` - Processed field with extracted links and clean text
    ///
    /// # Performance
    /// Uses iterator chaining and early returns for efficient link extraction
    #[inline(always)]
    pub fn new(name: &str, value: &str, verified_at: Option<DateTime<Utc>>) -> AccountField {
        let cleaned = clean_html(value);
        let parsed = cleaned
            .1
            .into_iter()
            .filter_map(|e| match e {
                HtmlItem::Link { url, .. } => url::Url::parse(&url)
                    .ok()
                    .map(|url| (url.host_str().unwrap_or("Link").to_string(), url)),
                HtmlItem::Mention { url, name } => {
                    url::Url::parse(&url).ok().map(|url| (name, url))
                }
                _ => None,
            })
            .next();
        let (value_parsed, link) = parsed.unzip();
        AccountField {
            name: name.to_string(),
            value: value.to_string(),
            value_parsed: value_parsed.unwrap_or_else(|| value.to_string()),
            link,
            verified_at,
        }
    }
}

/// Account update view model for timeline display
#[derive(Default, Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct AccountUpdateViewModel {
    pub id: AccountId,
    pub favorited: bool,
    pub account: AccountViewModel,
    pub content: String,
    pub last_updated_human: String,
    pub last_updated_full: String,
    pub last_updated: DateTime<Utc>,
}

impl AccountUpdateViewModel {
    /// Create AccountUpdateViewModel from StatusViewModel with content truncation
    ///
    /// # Arguments
    /// * `status` - The status to create an account update from
    ///
    /// # Returns
    /// * `AccountUpdateViewModel` - Account update with formatted content
    ///
    /// # Performance
    /// Uses efficient content processing with conditional boost detection
    #[inline(always)]
    pub fn new(status: &super::StatusViewModel) -> Self {
        let account = status.account.clone();
        let mut content = if let Some(ref boosted_content) = status.reblog_status {
            format!("{} boosted: {}", account.username, boosted_content.text)
        } else {
            status.text.clone()
        };

        // Efficient content truncation
        if content.len() > 140 {
            content = content.chars().take(140).collect();
            content.push('…');
        }

        Self {
            id: account.id.clone(),
            favorited: false,
            account,
            content,
            last_updated_human: status.created_human.clone(),
            last_updated_full: status.created_full.clone(),
            last_updated: status.created,
        }
    }
}

/// Relationship status for user interactions
#[derive(Default, Clone, Debug, Serialize, Deserialize)]
pub struct Relationship {
    pub following: bool,
    pub followed_by: bool,
    pub blocked: bool,
    pub muting: bool,
    pub requested: bool,
}

impl Relationship {
    #[inline(always)]
    pub fn new() -> Self {
        Self::default()
    }
}

/// Convert AccountViewModel back to megalodon::entities::Account
///
/// Used for operations that require returning Account objects to the API
impl From<AccountViewModel> for megalodon::entities::Account {
    #[inline(always)]
    fn from(account_vm: AccountViewModel) -> Self {
        megalodon::entities::Account {
            id: account_vm.id.0,
            username: account_vm.username,
            acct: account_vm.acct,
            display_name: account_vm.display_name,
            locked: account_vm.locked,
            discoverable: None,
            group: None,
            noindex: None,
            moved: None,
            suspended: None,
            limited: None,
            created_at: account_vm.joined,
            followers_count: account_vm.followers as i32,
            following_count: account_vm.following,
            statuses_count: account_vm.statuses,
            note: account_vm.note_plain,
            url: account_vm.url,
            avatar: account_vm.image.clone(),
            avatar_static: account_vm.image,
            header: account_vm.header.clone(),
            header_static: account_vm.header,
            emojis: vec![], // Emojis are processed during HTML rendering and not stored in AccountViewModel
            fields: account_vm
                .fields
                .into_iter()
                .map(|field| megalodon::entities::Field {
                    name: field.name,
                    value: field.value,
                    verified_at: field.verified_at,
                    verified: field.verified_at.map(|_| true),
                })
                .collect(),
            bot: account_vm.bot,
            source: None,
            role: None,
            mute_expires_at: None,
        }
    }
}
