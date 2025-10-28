use serde::{Deserialize, Deserializer, Serialize, Serializer};
use zeroize::Zeroizing;

#[derive(Clone, Serialize, Deserialize)]
pub struct OAuthResponse {
    #[serde(deserialize_with = "deserialize_zeroizing")]
    #[serde(serialize_with = "serialize_zeroizing")]
    pub access_token: Zeroizing<String>,
    pub expires_in: u64,
    pub scope: String,
    pub token_type: String,
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
            .field("refresh_token", &if self.refresh_token.is_some() { "[REDACTED]" } else { "None" })
            .finish()
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
    pub id: String,
    pub email: String,
    pub verified_email: bool,
    pub name: Option<String>,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub picture: Option<String>,
    pub locale: Option<String>,
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

/// Custom serde functions for secure memory handling
fn serialize_zeroizing<S>(value: &Zeroizing<String>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(value.as_str())
}

fn deserialize_zeroizing<'de, D>(deserializer: D) -> std::result::Result<Zeroizing<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Zeroizing::new(s))
}

fn serialize_option_zeroizing<S>(value: &Option<Zeroizing<String>>, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match value {
        Some(ref v) => serializer.serialize_some(v.as_str()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_option_zeroizing<'de, D>(deserializer: D) -> std::result::Result<Option<Zeroizing<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;
    Ok(opt.map(Zeroizing::new))
}
