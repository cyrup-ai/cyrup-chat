use crate::{OAuthError, Result};
use url::Url;

pub fn extract_callback_code(callback_url: &str) -> Result<String> {
    let url = Url::parse(callback_url)?;

    // Check for error first
    if let Some(error) = url.query_pairs().find(|(key, _)| key == "error") {
        let error_desc = url
            .query_pairs()
            .find(|(key, _)| key == "error_description")
            .map(|(_, desc)| desc.into_owned())
            .unwrap_or_else(|| error.1.into_owned());
        return Err(OAuthError::Authorization(error_desc));
    }

    // Extract authorization code
    let code = url
        .query_pairs()
        .find(|(key, _)| key == "code")
        .map(|(_, value)| value.into_owned())
        .ok_or_else(|| OAuthError::Authorization("No authorization code found".to_string()))?;

    Ok(code)
}
