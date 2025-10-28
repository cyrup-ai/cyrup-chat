use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

#[derive(Clone, Serialize, Deserialize)]
pub struct OAuthResponse {
    #[serde(deserialize_with = "deserialize_zeroizing")]
    #[serde(serialize_with = "serialize_zeroizing")]
    pub access_token: Zeroizing<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    pub scope: String,
    pub token_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(deserialize_with = "deserialize_option_zeroizing")]
    #[serde(serialize_with = "serialize_option_zeroizing")]
    pub refresh_token: Option<Zeroizing<String>>,
}

impl std::fmt::Debug for OAuthResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OAuthResponse")
            .field("access_token", &"[REDACTED]")
            .field("expires_in", &self.expires_in)
            .field("scope", &self.scope)
            .field("token_type", &self.token_type)
            .field("refresh_token", &match &self.refresh_token {
                Some(_) => &Some("[REDACTED]"),
                None => &None::<&str>,
            })
            .finish()
    }
}

#[derive(Clone, Serialize, Deserialize)]
pub struct TokenResponse {
    #[serde(deserialize_with = "deserialize_zeroizing")]
    #[serde(serialize_with = "serialize_zeroizing")]
    pub access_token: Zeroizing<String>,
    pub expires_in: u64,
    pub scope: String,
    pub token_type: String,
}

impl std::fmt::Debug for TokenResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TokenResponse")
            .field("access_token", &"[REDACTED]")
            .field("expires_in", &self.expires_in)
            .field("scope", &self.scope)
            .field("token_type", &self.token_type)
            .finish()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserInfo {
    pub id: u64,
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub avatar_url: String,
    pub html_url: String,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub company: Option<String>,
    pub blog: Option<String>,
    pub public_repos: u32,
    pub public_gists: u32,
    pub followers: u32,
    pub following: u32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy)]
pub enum AccessType {
    Online,
    Offline,
}

impl AccessType {
    pub fn as_str(&self) -> &'static str {
        match self {
            AccessType::Online => "online",
            AccessType::Offline => "offline",
        }
    }
}

/// Callback handler for OAuth flow
#[derive(Debug)]
pub enum CallbackMode {
    /// HTTP server mode - starts a local server on the specified port
    Server { port: u16 },
}

impl CallbackMode {
    /// Create a server-based callback mode
    pub fn server(port: u16) -> Self {
        CallbackMode::Server { port }
    }
}

// Custom serde functions for Zeroizing<String> fields
fn deserialize_zeroizing<'de, D>(deserializer: D) -> Result<Zeroizing<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Zeroizing::new(s))
}

fn serialize_zeroizing<S>(value: &Zeroizing<String>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    value.as_str().serialize(serializer)
}

fn deserialize_option_zeroizing<'de, D>(
    deserializer: D,
) -> Result<Option<Zeroizing<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.map(Zeroizing::new))
}

fn serialize_option_zeroizing<S>(
    value: &Option<Zeroizing<String>>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match value {
        Some(v) => Some(v.as_str()).serialize(serializer),
        None => None::<&str>.serialize(serializer),
    }
}
