# GitHub OAuth Examples

This directory contains examples showing how to use the GitHub OAuth library both directly and generically.

## Environment Setup

Set these environment variables for the examples to work:

```bash
export GITHUB_CLIENT_ID="your_github_client_id"
export GITHUB_CLIENT_SECRET="your_github_client_secret"
```

You can get these from your GitHub App settings at: https://github.com/settings/applications/new

## Running Examples

```bash
# Run the generic OAuth example
cargo run --example generic_oauth
```

## Usage for cyrup-chat

The `generic_oauth.rs` example shows how cyrup-chat can use both Google and GitHub OAuth libraries interchangeably through common traits:

```rust
use github_oauth::{GitHubProvider, traits::OAuthProvider};
use google_oauth::{GoogleProvider, traits::OAuthProvider}; // hypothetical

// Generic function that works with any OAuth provider
async fn authenticate<P: OAuthProvider>() -> Result<UserInfo> {
    let scopes = P::default_scopes();
    // ... authentication logic
}

// Usage in cyrup-chat
match provider_type {
    "github" => authenticate::<GitHubProvider>().await?,
    "google" => authenticate::<GoogleProvider>().await?,
    _ => return Err("Unsupported provider"),
}
```

## Key Features for Generic Usage

1. **Common Traits**: `OAuthProvider`, `OAuthLogin`, `OAuthConfigBuilder`, `OAuthUserInfo`, `OAuthRefresh`
2. **Consistent API**: Same method names and patterns across providers
3. **Provider-specific defaults**: Each provider has appropriate default scopes and endpoints
4. **Type safety**: Compile-time guarantees that implementations are correct