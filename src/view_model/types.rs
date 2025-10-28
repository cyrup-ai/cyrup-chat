//! Core types and ID definitions for view models
//!
//! This module provides foundational types, enums, and ID wrappers used across
//! all view model components with zero-allocation patterns.

use megalodon::entities::status::StatusVisibility;
use serde::{Deserialize, Serialize};

/// Social media visibility options for posts and content
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Visibility {
    #[default]
    Public,
    Unlisted,
    Private,
    Direct,
    Local,
}

/// Zero-allocation conversion from StatusVisibility to Visibility
impl From<StatusVisibility> for Visibility {
    #[inline(always)]
    fn from(status_vis: StatusVisibility) -> Self {
        match status_vis {
            StatusVisibility::Public => Visibility::Public,
            StatusVisibility::Unlisted => Visibility::Unlisted,
            StatusVisibility::Private => Visibility::Private,
            StatusVisibility::Direct => Visibility::Direct,
            StatusVisibility::Local => Visibility::Unlisted, // Local visibility mapped to Unlisted
        }
    }
}

/// Account visibility settings for filtering content types
#[derive(enumset::EnumSetType, Debug, serde::Serialize, serde::Deserialize)]
pub enum AccountVisibility {
    Toots,
    Replies,
    Boosts,
}

/// Strongly-typed account identifier wrapper with zero-allocation patterns
#[derive(Debug, Eq, PartialEq, Hash, Clone, Default, Serialize, Deserialize)]
pub struct AccountId(pub String);

/// Strongly-typed status identifier wrapper with zero-allocation patterns
#[derive(Debug, Eq, PartialEq, Hash, Clone, Default, Serialize, Deserialize)]
pub struct StatusId(pub String);

impl StatusId {
    /// Generate DOM-compatible ID for status elements
    #[inline(always)]
    pub fn dom_id(&self) -> String {
        format!("status-{}", self.0)
    }
}

impl std::fmt::Display for StatusId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("StatusID:{}", self.0))
    }
}

// Zero-allocation conversion implementations for StatusId
impl From<String> for StatusId {
    #[inline(always)]
    fn from(s: String) -> Self {
        StatusId(s)
    }
}

impl From<StatusId> for String {
    #[inline(always)]
    fn from(id: StatusId) -> Self {
        id.0
    }
}

impl From<&str> for StatusId {
    #[inline(always)]
    fn from(s: &str) -> Self {
        StatusId(s.to_string())
    }
}

// Zero-allocation conversion implementations for AccountId
impl From<String> for AccountId {
    #[inline(always)]
    fn from(s: String) -> Self {
        AccountId(s)
    }
}

impl From<AccountId> for String {
    #[inline(always)]
    fn from(id: AccountId) -> Self {
        id.0
    }
}

impl From<&str> for AccountId {
    #[inline(always)]
    fn from(s: &str) -> Self {
        AccountId(s.to_string())
    }
}

/// List view model for social media list information
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ListViewModel {
    pub id: String,
    pub title: String,
    pub replies_policy: String,
    pub exclusive: bool,
}
