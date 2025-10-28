/// Security: Sanitize API error messages to prevent information disclosure
/// 
/// This function parses OAuth API error responses and returns sanitized error messages
/// that don't leak sensitive information to potential attackers.
/// 
/// # Arguments
/// * `error_text` - The raw error response text from the OAuth API
/// * `status_code` - The HTTP status code of the error response
/// 
/// # Returns
/// A sanitized error message safe for display to users
/// 
/// # Security Features
/// - Parses JSON errors to extract only safe information
/// - Maps specific OAuth error codes to user-friendly messages  
/// - Falls back to generic messages based on HTTP status codes
/// - Prevents information disclosure attacks
pub fn sanitize_api_error(error_text: &str, status_code: u16) -> String {
    // Parse JSON error if possible to extract safe information
    if let Ok(json) = serde_json::from_str::<serde_json::Value>(error_text) {
        if let Some(error_obj) = json.get("error") {
            // Check for error_description field first (more detailed)
            if let Some(error_desc) = error_obj.get("error_description") {
                if let Some(desc) = error_desc.as_str() {
                    return match desc {
                        desc if desc.contains("invalid_client") => {
                            "Invalid client credentials".to_string()
                        }
                        desc if desc.contains("invalid_grant") => {
                            "Invalid authorization code".to_string()
                        }
                        desc if desc.contains("invalid_request") => {
                            "Invalid request parameters".to_string()
                        }
                        desc if desc.contains("unsupported_grant_type") => {
                            "Unsupported grant type".to_string()
                        }
                        _ => "OAuth authentication failed".to_string(),
                    };
                }
            }
            
            // Fall back to error field if no error_description
            if let Some(error_type) = error_obj.as_str() {
                return match error_type {
                    "invalid_client" => "Invalid client credentials".to_string(),
                    "invalid_grant" => "Invalid authorization code".to_string(),
                    "invalid_request" => "Invalid request parameters".to_string(),
                    "unsupported_grant_type" => "Unsupported grant type".to_string(),
                    _ => "OAuth authentication failed".to_string(),
                };
            }
        }
    }

    // Fallback based on HTTP status code for non-JSON errors
    match status_code {
        400 => "Bad request - invalid OAuth parameters".to_string(),
        401 => "Unauthorized - invalid client credentials".to_string(),
        403 => "Forbidden - client not authorized".to_string(),
        429 => "Rate limit exceeded - please try again later".to_string(),
        500..=599 => "OAuth service temporarily unavailable".to_string(),
        _ => "OAuth authentication failed".to_string(),
    }
}

/// Security: Generate HTTP response with comprehensive security headers
/// 
/// Creates a complete HTTP response with security headers to protect against
/// various web vulnerabilities in the OAuth callback response.
/// 
/// # Arguments
/// * `html` - The HTML content to include in the response body
/// 
/// # Returns
/// A complete HTTP response string with security headers
/// 
/// # Security Headers Included
/// - X-Frame-Options: DENY (prevent clickjacking)
/// - X-Content-Type-Options: nosniff (prevent MIME sniffing)
/// - X-XSS-Protection: enable XSS filtering
/// - Content-Security-Policy: restrict resource loading
/// - Referrer-Policy: limit referrer information leakage
/// - Cache-Control: prevent sensitive data caching
/// - Pragma: additional cache prevention for older clients
/// - Expires: ensure immediate expiration
pub fn create_secure_http_response(html: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\n\
         Content-Type: text/html; charset=utf-8\r\n\
         X-Frame-Options: DENY\r\n\
         X-Content-Type-Options: nosniff\r\n\
         X-XSS-Protection: 1; mode=block\r\n\
         Content-Security-Policy: default-src 'self'; script-src 'unsafe-inline'; style-src 'unsafe-inline'\r\n\
         Referrer-Policy: strict-origin-when-cross-origin\r\n\
         Cache-Control: no-cache, no-store, must-revalidate\r\n\
         Pragma: no-cache\r\n\
         Expires: 0\r\n\
         \r\n{html}"
    )
}