# OAuth Credential Vault System

## Overview

The CYRUP Chat application now includes a secure credential vault system for storing OAuth tokens and user credentials. This system replaces the previous plain JSON file storage with native OS credential stores for enhanced security.

## Architecture

### Components

1. **CredentialVault** (`src/auth/vault.rs`) - Core vault implementation using keyring-rs
2. **AuthState Integration** (`src/auth/mod.rs`) - Seamless integration with existing auth system
3. **Provider Updates** (`src/auth/providers.rs`) - OAuth providers now save to vault
4. **Migration Support** - Automatic migration from legacy file storage

### Security Features

- **Native OS Integration**: Uses platform-specific secure stores
  - macOS: Keychain
  - Windows: Credential Manager
  - Linux: Secret Service (GNOME Keyring, KWallet, etc.)
- **Encrypted Storage**: Credentials are encrypted by the OS
- **Access Control**: OS-level access control and user authentication
- **Automatic Migration**: Seamlessly migrates from plain JSON storage

## Usage

### Basic Operations

```rust
use crate::auth::{CredentialVault, CredentialType};

// Create vault instance
let vault = CredentialVault::new();

// Store OAuth access token
vault.store_credential(
    "google",                           // provider
    CredentialType::AccessToken,        // credential type
    "ya29.a0ARrdaM...",                 // token value
    Some(expires_at),                   // expiration (optional)
    None,                               // metadata (optional)
)?;

// Retrieve credential
if let Some(credential) = vault.get_credential("google", &CredentialType::AccessToken)? {
    println!("Token: {}", credential.value);
    println!("Stored at: {}", credential.stored_at);
}

// Delete credential
vault.delete_credential("google", &CredentialType::AccessToken)?;
```

### Credential Types

- `AccessToken` - OAuth access tokens
- `RefreshToken` - OAuth refresh tokens  
- `UserInfo` - User profile information (stored as JSON)
- `ClientCredentials` - OAuth client credentials

### AuthState Integration

The vault system is transparently integrated with the existing `AuthState`:

```rust
// Save auth state (automatically uses vault)
let auth_state = AuthState { /* ... */ };
auth_state.save().await?;

// Load auth state (tries vault first, falls back to file)
if let Some(auth_state) = AuthState::load().await? {
    println!("Loaded from secure vault");
}

// Delete all credentials
AuthState::delete().await?;
```

## Migration

### Automatic Migration

When `AuthState::load()` is called:

1. **Vault Check**: First attempts to load from secure vault
2. **File Fallback**: If no vault data, checks legacy JSON file
3. **Migration**: If file exists, migrates data to vault and cleans up file
4. **Seamless**: No user intervention required

### Manual Migration

For advanced use cases:

```rust
// Load from file only
if let Some(auth_state) = AuthState::load_from_file().await? {
    // Migrate to vault
    auth_state.save_to_vault().await?;
    
    // Clean up file
    AuthState::delete_file().await?;
}
```

## Configuration

### Keyring Features

The vault uses keyring-rs with these features:
- `apple-native` - macOS Keychain support
- `windows-native` - Windows Credential Manager support  
- `sync-secret-service` - Linux Secret Service support

### Service Name

Credentials are stored under the service name: `ai.cyrup.chat`

### Credential Keys

Keys follow the pattern: `{provider}_{credential_type}`
- `google_access_token`
- `google_refresh_token`
- `github_access_token`
- etc.

## Error Handling

The vault system provides comprehensive error handling:

```rust
match vault.get_credential("google", &CredentialType::AccessToken) {
    Ok(Some(credential)) => {
        // Credential found and valid
    }
    Ok(None) => {
        // No credential stored
    }
    Err(e) => {
        // Handle vault errors (platform issues, access denied, etc.)
        log::error!("Vault error: {}", e);
    }
}
```

### Common Error Scenarios

- **Platform Failure**: OS credential store unavailable
- **Access Denied**: User denied access to credential store
- **No Entry**: Credential not found
- **Invalid Data**: Corrupted credential data

## Testing

The vault system includes comprehensive tests:

```bash
# Run vault tests
cargo test --lib auth::vault

# Test specific functionality
cargo test test_store_and_retrieve_credential
cargo test test_credential_expiration_check
cargo test test_delete_provider_credentials
```

### Test Isolation

Tests use a separate service name (`ai.cyrup.chat.test`) to avoid conflicts with production credentials.

## Security Considerations

### Best Practices

1. **OS Updates**: Keep OS updated for latest security patches
2. **User Authentication**: Vault access requires user to be logged in
3. **Encryption**: All data encrypted by OS credential store
4. **Access Logging**: OS may log credential access attempts

### Limitations

- **Enumeration**: Cannot list all stored credentials (OS limitation)
- **Cross-User**: Credentials are per-user, not system-wide
- **Backup**: Credential backup depends on OS backup systems

## Troubleshooting

### Common Issues

1. **Vault Unavailable**
   ```
   Error: Platform credential store failed
   Solution: Check OS credential service is running
   ```

2. **Access Denied**
   ```
   Error: User denied access to credential store
   Solution: Grant permission when prompted by OS
   ```

3. **Migration Failed**
   ```
   Warning: Failed to migrate auth state to vault
   Solution: Check vault permissions, may need manual cleanup
   ```

### Debug Logging

Enable debug logging to troubleshoot vault operations:

```bash
RUST_LOG=debug cargo run
```

## Future Enhancements

### Planned Features

1. **Client Credential Storage**: Store OAuth client IDs/secrets in vault
2. **Multiple Accounts**: Support multiple accounts per provider
3. **Credential Rotation**: Automatic token rotation and cleanup
4. **Backup/Restore**: Export/import encrypted credential backups

### Platform Extensions

- **iOS/Android**: Mobile keychain integration
- **Web**: Browser credential manager integration
- **Enterprise**: Integration with enterprise credential stores

## API Reference

### CredentialVault

```rust
impl CredentialVault {
    pub fn new() -> Self
    pub fn with_service(service: String) -> Self
    
    pub fn store_credential(
        &self,
        provider: &str,
        credential_type: CredentialType,
        value: &str,
        expires_at: Option<DateTime<Utc>>,
        metadata: Option<HashMap<String, String>>,
    ) -> Result<()>
    
    pub fn get_credential(
        &self,
        provider: &str,
        credential_type: &CredentialType,
    ) -> Result<Option<StoredCredential>>
    
    pub fn delete_credential(
        &self,
        provider: &str,
        credential_type: &CredentialType,
    ) -> Result<bool>
    
    pub fn delete_provider_credentials(&self, provider: &str) -> Result<u32>
    pub fn is_credential_valid(&self, provider: &str, credential_type: &CredentialType) -> Result<bool>
    pub fn list_providers(&self) -> Result<Vec<String>>
}
```

### StoredCredential

```rust
pub struct StoredCredential {
    pub value: String,
    pub stored_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, String>,
}
```

### CredentialType

```rust
pub enum CredentialType {
    AccessToken,
    RefreshToken,
    UserInfo,
    ClientCredentials,
}
```
