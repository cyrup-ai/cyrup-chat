use crate::{error::OAuthError, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use sha2::{Digest, Sha256};
use zeroize::Zeroizing;

/// PKCE (Proof Key for Code Exchange) challenge and verifier pair
/// 
/// Implements RFC 7636 for OAuth 2.0 security enhancement.
/// The code verifier is a cryptographically random string using the 
/// unreserved characters [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"
/// with a minimum length of 43 characters and a maximum length of 128 characters.
pub struct PkceChallenge {
    /// The code verifier - a cryptographically random string (43-128 chars)
    pub code_verifier: Zeroizing<String>,
    /// The code challenge - SHA256 hash of verifier, base64url encoded
    pub code_challenge: String,
}

impl PkceChallenge {
    /// Generate a new PKCE challenge/verifier pair
    /// 
    /// Creates a cryptographically secure code verifier of 128 characters
    /// and generates the corresponding SHA256+base64url code challenge.
    /// 
    /// # Returns
    /// 
    /// Returns a `Result<PkceChallenge>` containing the challenge pair
    /// or an error if generation fails.
    /// 
    /// # Example
    /// 
    /// ```rust
    /// use google_oauth::PkceChallenge;
    /// 
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let challenge = PkceChallenge::new()?;
    /// println!("Challenge: {}", challenge.code_challenge);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Result<Self> {
        let code_verifier = generate_code_verifier()?;
        let code_challenge = generate_code_challenge(&code_verifier)?;
        
        Ok(Self {
            code_verifier,
            code_challenge,
        })
    }

    /// Create a PKCE challenge from an existing code verifier
    /// 
    /// # Arguments
    /// 
    /// * `verifier` - The code verifier string (must be 43-128 characters)
    /// 
    /// # Returns
    /// 
    /// Returns a `Result<PkceChallenge>` containing the challenge pair
    /// or an error if the verifier is invalid or challenge generation fails.
    pub fn from_verifier(verifier: String) -> Result<Self> {
        let secure_verifier = Zeroizing::new(verifier);
        validate_code_verifier(&secure_verifier)?;
        let code_challenge = generate_code_challenge(&secure_verifier)?;
        
        Ok(Self {
            code_verifier: secure_verifier,
            code_challenge,
        })
    }

    /// Get the code challenge method (always "S256" for this implementation)
    #[inline]
    pub fn challenge_method(&self) -> &'static str {
        "S256"
    }
}

impl std::fmt::Debug for PkceChallenge {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PkceChallenge")
            .field("code_verifier", &"[REDACTED]")
            .field("code_challenge", &self.code_challenge)
            .finish()
    }
}

impl Clone for PkceChallenge {
    fn clone(&self) -> Self {
        Self {
            code_verifier: Zeroizing::new(self.code_verifier.as_str().to_string()),
            code_challenge: self.code_challenge.clone(),
        }
    }
}

/// Generate a cryptographically secure code verifier
/// 
/// Creates a 128-character string using alphanumeric characters.
/// This is at the maximum length allowed by RFC 7636 for optimal security.
fn generate_code_verifier() -> Result<Zeroizing<String>> {
    // Use 128 characters for maximum security (RFC 7636 allows 43-128)
    const VERIFIER_LENGTH: usize = 128;
    
    // Generate random alphanumeric string
    // Using thread_rng() for cryptographic security
    let verifier: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(VERIFIER_LENGTH)
        .map(char::from)
        .collect();

    // Validate the generated verifier (should never fail with Alphanumeric)
    let verifier_zeroizing = Zeroizing::new(verifier);
    validate_code_verifier(&verifier_zeroizing)?;
    
    Ok(verifier_zeroizing)
}

/// Generate code challenge from code verifier using S256 method
/// 
/// # Arguments
/// 
/// * `verifier` - The code verifier to hash
/// 
/// # Returns
/// 
/// Returns the SHA256 hash of the verifier, base64url encoded without padding
fn generate_code_challenge(verifier: &Zeroizing<String>) -> Result<String> {
    // SHA256 hash the verifier
    let digest = Sha256::digest(verifier.as_str().as_bytes());
    
    // Base64url encode without padding per RFC 7636
    let challenge = URL_SAFE_NO_PAD.encode(digest);
    
    Ok(challenge)
}

/// Validate code verifier according to RFC 7636 requirements
/// 
/// # Arguments
/// 
/// * `verifier` - The code verifier to validate
/// 
/// # Returns
/// 
/// Returns `Ok(())` if valid, or `OAuthError::InvalidCodeChallenge` if invalid
fn validate_code_verifier(verifier: &Zeroizing<String>) -> Result<()> {
    let verifier_str = verifier.as_str();
    let len = verifier_str.len();
    
    // RFC 7636: code verifier must be 43-128 characters
    if len < 43 || len > 128 {
        return Err(OAuthError::InvalidCodeChallenge(format!(
            "Code verifier length {} is invalid (must be 43-128 characters)", 
            len
        )));
    }
    
    // RFC 7636: only unreserved characters allowed
    // [A-Z] / [a-z] / [0-9] / "-" / "." / "_" / "~"
    for ch in verifier_str.chars() {
        if !is_unreserved_char(ch) {
            return Err(OAuthError::InvalidCodeChallenge(format!(
                "Code verifier contains invalid character '{}' (only unreserved characters allowed)",
                ch
            )));
        }
    }
    
    Ok(())
}

/// Check if character is unreserved per RFC 3986
/// 
/// unreserved = ALPHA / DIGIT / "-" / "." / "_" / "~"
#[inline]
fn is_unreserved_char(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || matches!(ch, '-' | '.' | '_' | '~')
}

