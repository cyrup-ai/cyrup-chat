use crate::{error::OAuthError, Result};
use super::security::create_secure_http_response;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// Wait for OAuth callback from browser and extract authorization code
/// 
/// This function starts a local HTTP server to receive the OAuth callback,
/// validates the state parameter for CSRF protection, and returns the authorization code.
/// 
/// # Arguments
/// * `listener` - TCP listener bound to the callback port
/// * `expected_state` - Expected state parameter for CSRF validation
/// 
/// # Returns
/// The authorization code on success, or an error if validation fails
/// 
/// # Security
/// - Limits request size to prevent DoS attacks
/// - Validates CSRF state parameter
/// - Sends secure HTTP response headers
/// - Sanitizes error messages
pub async fn wait_for_callback(
    listener: tokio::net::TcpListener,
    expected_state: &str,
) -> Result<String> {
    log::debug!("Waiting for OAuth callback on listener");
    let (mut stream, _) = listener.accept().await?;
    log::debug!("Accepted connection from browser");

    // Security: Limit request size to prevent DoS attacks
    const MAX_REQUEST_SIZE: usize = 8192; // 8KB should be sufficient for OAuth callbacks
    let mut buffer = vec![0; MAX_REQUEST_SIZE];

    let n = match stream.read(&mut buffer).await {
        Ok(n) if n == MAX_REQUEST_SIZE => {
            // If we read exactly MAX_REQUEST_SIZE, the request might be larger
            return Err(OAuthError::Server("Request too large".to_string()));
        }
        Ok(n) => n,
        Err(e) => return Err(OAuthError::Io(e)),
    };

    let request = String::from_utf8_lossy(&buffer[..n]);

    // Extract the HTTP request line
    let request_line = request
        .lines()
        .next()
        .ok_or_else(|| OAuthError::Server("Invalid request".to_string()))?;

    // Parse the callback URL path from GET request
    let path = request_line
        .split_whitespace()
        .nth(1)
        .ok_or_else(|| OAuthError::Server("Invalid request path".to_string()))?;

    let callback_url = format!("http://localhost{path}");
    let url = url::Url::parse(&callback_url)?;

    // Extract OAuth parameters from query string
    let mut code = None;
    let mut state = None;

    for (key, value) in url.query_pairs() {
        match key.as_ref() {
            "code" => code = Some(value.into_owned()),
            "state" => state = Some(value.into_owned()),
            "error" => {
                // Handle OAuth error response
                let error_desc = url
                    .query_pairs()
                    .find(|(k, _)| k == "error_description")
                    .map(|(_, v)| v.into_owned())
                    .unwrap_or_else(|| value.into_owned());

                // Send error response to browser
                let html = crate::template::default_success_template(
                    &crate::template::TemplateContext::error(error_desc.clone()),
                )
                .into_string();
                let response = create_secure_http_response(&html);
                stream.write_all(response.as_bytes()).await?;
                stream.shutdown().await?;

                return Err(OAuthError::Authorization(error_desc));
            }
            _ => {}
        }
    }

    // Validate required parameters are present
    let code =
        code.ok_or_else(|| OAuthError::Authorization("No authorization code found".to_string()))?;
    let state =
        state.ok_or_else(|| OAuthError::Authorization("No state parameter found".to_string()))?;

    // Security: Verify state matches expected value (CSRF protection)
    if state != expected_state {
        // Send error response for invalid state
        let html =
            crate::template::default_success_template(&crate::template::TemplateContext::error(
                "Invalid state parameter - possible CSRF attack",
            ))
            .into_string();
        let response = create_secure_http_response(&html);
        stream.write_all(response.as_bytes()).await?;
        stream.shutdown().await?;

        return Err(OAuthError::InvalidState);
    }

    // Send success response to browser
    let html =
        crate::template::default_success_template(&crate::template::TemplateContext::success())
            .into_string();
    let response = create_secure_http_response(&html);
    log::debug!("Sending success response to browser");
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    
    // Give browser time to receive the response before closing connection
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    stream.shutdown().await?;
    log::info!("OAuth callback handled successfully");

    Ok(code)
}