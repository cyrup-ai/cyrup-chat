use google_oauth::PkceChallenge;

#[test]
fn test_pkce_challenge_new() {
    let challenge = PkceChallenge::new().expect("PKCE challenge generation should work in tests");
    
    // Verify code verifier properties
    assert_eq!(challenge.code_verifier.as_str().len(), 128);
    assert!(!challenge.code_challenge.is_empty());
    assert_eq!(challenge.challenge_method(), "S256");
    
    // Verify code verifier uses only allowed characters (unreserved chars per RFC 7636)
    for ch in challenge.code_verifier.as_str().chars() {
        assert!(ch.is_ascii_alphanumeric(), "Invalid character: {}", ch);
    }
}

#[test] 
fn test_from_verifier_valid() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk".to_string();
    let challenge = PkceChallenge::from_verifier(verifier.clone()).expect("PKCE from verifier should work in tests");
    
    assert_eq!(challenge.code_verifier.as_str(), &verifier);
    assert_eq!(challenge.code_challenge, "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM");
    assert_eq!(challenge.challenge_method(), "S256");
}

#[test]
fn test_from_verifier_too_short() {
    let verifier = "short".to_string();
    let result = PkceChallenge::from_verifier(verifier);
    assert!(result.is_err());
}

#[test]
fn test_from_verifier_invalid_char() {
    let verifier = "dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk!".to_string();
    let result = PkceChallenge::from_verifier(verifier);
    assert!(result.is_err());
}

#[test]
fn test_challenge_method() {
    let challenge = PkceChallenge::new().expect("PKCE challenge generation should work in tests");
    assert_eq!(challenge.challenge_method(), "S256");
}

#[test]
fn test_debug_redaction() {
    let challenge = PkceChallenge::new().expect("PKCE challenge generation should work in tests");
    let debug_output = format!("{:?}", challenge);
    
    // Verify that debug output shows redacted verifier
    assert!(debug_output.contains("[REDACTED]"));
    assert!(!debug_output.contains(challenge.code_verifier.as_str()));
    
    // Verify that challenge is still visible
    assert!(debug_output.contains(&challenge.code_challenge));
}

#[test]
fn test_clone() {
    let challenge = PkceChallenge::new().expect("PKCE challenge generation should work in tests");
    let cloned = challenge.clone();
    
    assert_eq!(challenge.code_verifier.as_str(), cloned.code_verifier.as_str());
    assert_eq!(challenge.code_challenge, cloned.code_challenge);
    assert_eq!(challenge.challenge_method(), cloned.challenge_method());
}