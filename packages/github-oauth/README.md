# github-oauth

A pure Rust implementation of GitHub OAuth 2.0 browser flow with a beautiful, chainable API and generic traits for use alongside other OAuth providers.

## Features

- 🌐 OAuth 2.0 browser-based authentication flow for GitHub
- 🔐 PKCE (Proof Key for Code Exchange) support for enhanced security (2025)
- 👤 User info retrieval from GitHub API
- 🚀 Zero error handling in the chain - errors surface at `.await`
- ⛓️ Beautiful method chaining with single await  
- 🎯 Type-safe builder pattern
- 🔧 Generic traits for interoperability with other OAuth providers
- 🛡️ Production-ready security features (CSRF protection, request limits, sanitized errors)

## Installation

```toml
[dependencies]
github-oauth = "0.1.0"
```

## Usage

### Login Flow

```rust
use github_oauth::Login;

// Enhanced security with PKCE (Proof Key for Code Exchange) - Always enabled in 2025
let token = Login::from_env()
    .scopes(["user:email", "repo"])?  // PKCE automatically enabled
    .state("my-csrf-token")
    .login()
    .await?;

// Login with environment variables - PKCE always included
let token = Login::from_env()
    .scopes(["user:email", "repo"])?  // PKCE automatically enabled
    .state("my-csrf-token")
    .login()
    .await?;

// Login with explicit credentials
let token = Login::client_id("your-github-client-id")
    .client_secret("your-github-client-secret")
    .scopes(["user:email"])?  // PKCE automatically enabled
    .port(8080)
    .login()
    .await?;
```

### Refresh Token

```rust
use github_oauth::Refresh;

// Note: GitHub does not support refresh tokens
// GitHub access tokens do not expire, so refresh is not needed
// If you need fresh permissions, re-authenticate with the user

// For compatibility with generic OAuth traits:
let result = Refresh::from_env()
    .token("unused")
    .refresh()
    .await; // Will return an error explaining GitHub doesn't support refresh
```

### User Info

```rust
use github_oauth::UserInfoBuilder;

// Get user information
let user = UserInfoBuilder::token("gho_xxxxxxxxxxxxxxxxxxxx")
    .get_info()
    .await?;

println!("Username: {}", user.login);
println!("Name: {:?}", user.name);
println!("Email: {:?}", user.email);
println!("Avatar: {}", user.avatar_url);
println!("Followers: {}", user.followers);
```

## PKCE (Proof Key for Code Exchange) Support

This library supports PKCE as specified in RFC 7636 for enhanced OAuth 2.0 security. PKCE protects against authorization code interception attacks and is mandatory for all OAuth flows in 2025.

### PKCE Features

- **Always Enabled**: PKCE is mandatory in all OAuth flows (2025 security standard)
- **Automatic Generation**: Generate cryptographically secure code verifiers (128 characters)
- **S256 Challenge Method**: Uses SHA256 + base64url encoding per GitHub's requirements
- **Production Ready**: Comprehensive error handling and validation

### PKCE Usage

```rust
use github_oauth::{Login, AuthFlow, PkceChallenge};

// Method 1: PKCE is always automatically enabled (2025 standard)
let token = Login::from_env()
    .scopes(["user:email"])?  // PKCE challenge automatically generated
    .login()
    .await?;

// Method 2: Custom PKCE challenge (for testing or manual control)
let pkce_challenge = PkceChallenge::new()?;
let token = Login::from_env()
    .scopes(["user:email"])?
    .with_pkce_challenge(pkce_challenge)
    .login()
    .await?;

// Method 3: Using AuthFlow (PKCE automatically included)
let auth_flow = AuthFlow::new("client_id", "client_secret", "redirect_uri")?  // PKCE auto-generated
    .with_scopes(vec!["user:email".to_string()]);

let auth_url = auth_flow.auth_url();
// ... redirect user to auth_url and get callback ...
let token = auth_flow.handle_callback(&callback_url).await?;

// Method 4: Traditional flow without PKCE (still supported)
let token = Login::from_env()
    .scopes(["user:email"])
    // No .enable_pkce() call
    .login()
    .await?;
```

### Security Considerations

- **PKCE Recommended**: While optional, PKCE is strongly recommended for all OAuth flows
- **No Client Secret Exposure**: PKCE reduces the impact of client secret exposure
- **Protection Against Interception**: Guards against authorization code interception attacks
- **GitHub Support**: GitHub supports PKCE as of July 2025

### PKCE Implementation Details

- **Code Verifier**: 128-character cryptographically random string
- **Code Challenge**: SHA256 hash of verifier, base64url encoded
- **Challenge Method**: Always "S256" (only method supported by GitHub)
- **Error Handling**: Proper error propagation if PKCE generation fails (no insecure fallbacks)

## Generic OAuth Traits

This library implements common traits that allow for generic OAuth usage alongside other providers (like Google OAuth):

```rust
use github_oauth::{GitHubProvider, Login, traits::{OAuthProvider, OAuthLogin}};

// Generic function that works with any OAuth provider
async fn authenticate_user<P: OAuthProvider>() -> Result<String> {
    println!("Authenticating with {}", P::provider_name());
    
    let token = Login::from_env()
        .scopes(P::default_scopes().iter().map(|s| s.to_string()))
        .login()
        .await?;
    
    Ok(token.access_token)
}

// Usage
let github_token = authenticate_user::<GitHubProvider>().await?;
```

This allows applications like cyrup-chat to support multiple OAuth providers with the same code:

```rust
// In cyrup-chat
match provider {
    "github" => authenticate_user::<GitHubProvider>().await?,
    "google" => authenticate_user::<GoogleProvider>().await?, // When google-oauth crate is included
    _ => return Err("Unsupported provider"),
}
```

## Builder API

The API uses a polymorphic builder pattern where each method transitions to the appropriate builder type:

### Login Entry Points
- `Login::from_env()` → `LoginScopesBuilder`
- `Login::client_id(id)` → `LoginClientSecretBuilder`

### LoginScopesBuilder Methods
- `.scopes(scopes)` → `LoginConfigBuilder`
- `.add_scope(scope)` → `LoginConfigBuilder`

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
    .scopes(["tasks"])
    .login()
    .await;  // Error surfaces here

match result {
    Ok(token) => {
        println!("Access token: {}", token.access_token);
        println!("Refresh token: {:?}", token.refresh_token);
    },
    Err(e) => eprintln!("OAuth failed: {}", e),
}
```

## Examples

### Complete Authentication Flow

```rust
use github_oauth::{Login, UserInfo};

// Initial login (PKCE automatically enabled in 2025)
let oauth_response = Login::from_env()
    .scopes(["user:email", "repo"])?  // PKCE automatically enabled
    .state("random-csrf-token")
    .login()
    .await?;

// Save tokens (GitHub tokens don't expire, but store securely)
save_tokens(&oauth_response)?;

// Get user info
let user = UserInfo::token(&oauth_response.access_token)
    .get_info()
    .await?;

println!("Authenticated as: {} ({})", user.name.as_ref().unwrap_or(&user.login), user.login);
```

### Custom Configuration

```rust
let token = Login::client_id(client_id)
    .client_secret(client_secret)
    .scopes(["user:email", "repo", "gist"])?  // PKCE automatically enabled
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
    .add_scope("https://www.googleapis.com/auth/tasks.readonly")
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
    .scopes(["tasks"])
    .access_type(AccessType::Offline)
    .login()
    .await?;

// Store securely (example uses file, consider using OS keychain)
let stored = StoredTokens {
    access_token: tokens.access_token,
    refresh_token: tokens.refresh_token,
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
        
        new_tokens.access_token
    } else {
        return Err(anyhow::anyhow!("No refresh token available"));
    }
} else {
    stored.access_token
};
```

## Implementation Notes

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