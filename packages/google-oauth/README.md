# google-oauth

A pure Rust implementation of Google OAuth 2.0 browser flow with a beautiful, chainable API.

## Features

- 🌐 OAuth 2.0 browser-based authentication flow
- 🔄 Token refresh support
- 👤 User info retrieval
- 🔐 **PKCE** (Proof Key for Code Exchange) enabled by default for enhanced security
- 🛡️ **Secure memory handling** - all sensitive tokens automatically cleared from memory on drop
- 🚀 Zero error handling in the chain - errors surface at `.await`
- ⛓️ Beautiful method chaining with single await
- 🎯 Type-safe builder pattern

## Installation

```toml
[dependencies]
google-oauth = "0.1.0"
```

## Usage

### Login Flow

```rust
use google_oauth::Login;

// Login with environment variables (GOOGLE_CLIENT_ID, GOOGLE_CLIENT_SECRET)
let token = Login::from_env()
    .scopes(["tasks", "drive", "calendar"])?
    .state("my-csrf-token")
    .access_type(AccessType::Offline)
    .login()
    .await?;

// Login with explicit credentials
let token = Login::client_id("your-client-id.apps.googleusercontent.com")
    .client_secret("your-client-secret")
    .scopes(["tasks"])?
    .port(8080)
    .login()
    .await?;
```

### Refresh Token

```rust
use google_oauth::Refresh;

// Refresh with environment variables
let new_token = Refresh::from_env()
    .token("1//0gLu...")
    .refresh()
    .await?;

// Refresh with explicit credentials
let new_token = Refresh::client_id("your-client-id.apps.googleusercontent.com")
    .client_secret("your-client-secret")
    .token("1//0gLu...")
    .refresh()
    .await?;
```

### User Info

```rust
use google_oauth::UserInfo;

// Get user information
let user = UserInfo::token("ya29.a0...")
    .get_info()
    .await?;

println!("Email: {}", user.email);
println!("Name: {}", user.name);
println!("Picture: {}", user.picture);
```

## Builder API

The API uses a polymorphic builder pattern where each method transitions to the appropriate builder type:

### Login Entry Points
- `Login::from_env()` → `LoginScopesBuilder`
- `Login::client_id(id)` → `LoginClientSecretBuilder`

### LoginScopesBuilder Methods
- `.scopes(scopes)` → `Result<LoginConfigBuilder>`
- `.add_scope(scope)` → `Result<LoginConfigBuilder>`

### LoginConfigBuilder Methods
- `.port(port)` → `LoginConfigBuilder`
- `.redirect_uri(uri)` → `LoginConfigBuilder`
- `.state(state)` → `LoginConfigBuilder`
- `.access_type(type)` → `LoginConfigBuilder`
- `.timeout(duration)` → `LoginConfigBuilder`
- `.login()` → `WrappedFuture<Result<OAuthResponse>>`

### Refresh Entry Points
- `Refresh::from_env()` → `RefreshTokenBuilder`
- `Refresh::client_id(id)` → `RefreshClientSecretBuilder`

### RefreshTokenBuilder Methods
- `.token(refresh_token)` → `RefreshExecuteBuilder`
- `.refresh()` → `WrappedFuture<Result<TokenResponse>>`

### UserInfo Entry Points
- `UserInfo::token(access_token)` → `UserInfoExecuteBuilder`
- `.get_info()` → `WrappedFuture<Result<UserInfo>>`

## Error Handling

All errors are deferred until the final `.await`:

```rust
// No '?' needed in the chain!
let result = Login::client_id("maybe-invalid")
    .client_secret("maybe-invalid")
    .scopes(["tasks"])?
    .login()
    .await;  // Error surfaces here

match result {
    Ok(token) => {
        println!("Access token: {}", &*token.access_token);
        println!("Refresh token: {:?}", token.refresh_token.as_ref().map(|t| &**t));
    },
    Err(e) => eprintln!("OAuth failed: {}", e),
}
```

## Examples

### Complete Authentication Flow

```rust
use google_oauth::{Login, Refresh, UserInfo};

// Initial login
let oauth_response = Login::from_env()
    .scopes(["tasks", "userinfo.email"])?
    .state("random-csrf-token")
    .access_type(AccessType::Offline)  // Get refresh token
    .login()
    .await?;

// Save tokens
save_tokens(&oauth_response)?;

// Later, refresh the token
if let Some(refresh_token) = &oauth_response.refresh_token {
    let new_tokens = Refresh::from_env()
        .token(refresh_token)
        .refresh()
        .await?;
    
    // Get user info
    let user = UserInfo::token(&*new_tokens.access_token)
        .get_info()
        .await?;
}
```

### Custom Configuration

```rust
let token = Login::client_id(client_id)
    .client_secret(client_secret)
    .scopes(["tasks", "drive", "calendar"])?
    .port(3000)  // Custom callback port
    .redirect_uri("http://localhost:3000/oauth/callback")
    .timeout(Duration::from_secs(120))  // 2 minute timeout
    .state(generate_csrf_token())
    .login()
    .await?;
```

### Minimal Scope Access

```rust
// Request only what you need
let token = Login::from_env()
    .add_scope("https://www.googleapis.com/auth/tasks.readonly")?
    .login()
    .await?;
```

## Token Storage

The library returns tokens but doesn't handle storage. Here's a simple example:

```rust
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct StoredTokens {
    access_token: String,
    refresh_token: Option<String>,
    expires_at: chrono::DateTime<chrono::Utc>,
}

// After login
let tokens = Login::from_env()
    .scopes(["tasks"])?
    .access_type(AccessType::Offline)
    .login()
    .await?;

// Store securely (example uses file, consider using OS keychain)
let stored = StoredTokens {
    access_token: tokens.access_token.to_string(),
    refresh_token: tokens.refresh_token.map(|t| t.to_string()),
    expires_at: chrono::Utc::now() + chrono::Duration::seconds(tokens.expires_in as i64),
};

std::fs::write("tokens.json", serde_json::to_string(&stored)?)?;

// Later, check if refresh needed
let stored: StoredTokens = serde_json::from_str(&std::fs::read_to_string("tokens.json")?)?;

let access_token = if stored.expires_at <= chrono::Utc::now() {
    // Refresh needed
    if let Some(refresh_token) = &stored.refresh_token {
        let new_tokens = Refresh::from_env()
            .token(refresh_token)
            .refresh()
            .await?;
        
        new_tokens.access_token.to_string()
    } else {
        return Err(anyhow::anyhow!("No refresh token available"));
    }
} else {
    stored.access_token
};
```

## Implementation Notes

### Security Features

- **PKCE (RFC 7636)**: All OAuth flows automatically use Proof Key for Code Exchange for enhanced security
- **Secure Memory**: Sensitive data (tokens, client secrets) uses `Zeroizing<String>` and is automatically cleared from memory when dropped
- **Debug Redaction**: Sensitive fields show `[REDACTED]` in debug output to prevent accidental logging
- **Error Sanitization**: API error messages are sanitized to prevent information disclosure

### Builder Pattern

The builders use `#[doc(hidden)]` on their `new()` methods to guide users toward the semantic entry points:

```rust
impl Login {
    #[doc(hidden)]
    pub fn new() -> Self { /* ... */ }
    
    pub fn from_env() -> LoginScopesBuilder { /* ... */ }
    pub fn client_id(id: impl Into<String>) -> LoginClientSecretBuilder { /* ... */ }
}
```

## License

MIT