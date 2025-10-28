// HTTP Header Parsing Utilities - Production Implementation
// RFC-compliant parsing replacing manual string manipulation

use http::HeaderValue;
use std::collections::HashMap;

/// Parse HTTP Link header according to RFC 5988
///
/// Extracts link relations and parameters from Link headers
/// commonly used in REST APIs for pagination
///
/// # Arguments
/// * `header_value` - The Link header value to parse
///
/// # Returns
/// * `Ok(Vec<LinkEntry>)` - Parsed link entries
/// * `Err(LinkParseError)` - Parsing error with details
///
/// # Examples
/// ```
/// # use cyrup::utils::http_parser::parse_link_header;
/// let header = r#"<https://api.example.com/users?page=2>; rel="next", <https://api.example.com/users?page=1>; rel="prev""#;
/// match parse_link_header(header) {
///     Ok(links) => {
///         assert_eq!(links.len(), 2);
///         // Process links...
///     }
///     Err(e) => {
///         log::error!("Failed to parse Link header: {}", e);
///     }
/// }
/// ```
#[inline]
pub fn parse_link_header(header_value: &str) -> Result<Vec<LinkEntry>, LinkParseError> {
    let mut links = Vec::new();

    // Split by comma to get individual link entries
    for link_str in header_value.split(',') {
        let link_str = link_str.trim();
        if link_str.is_empty() {
            continue;
        }

        let entry = parse_single_link(link_str)?;
        links.push(entry);
    }

    Ok(links)
}

/// Parse a single link entry from a Link header
///
/// # Arguments
/// * `link_str` - Single link entry string
///
/// # Returns
/// * `Ok(LinkEntry)` - Parsed link entry
/// * `Err(LinkParseError)` - Parsing error
fn parse_single_link(link_str: &str) -> Result<LinkEntry, LinkParseError> {
    // Find the URL part (between < and >)
    let url_start = link_str
        .find('<')
        .ok_or(LinkParseError::MissingUrlBrackets)?;
    let url_end = link_str
        .find('>')
        .ok_or(LinkParseError::MissingUrlBrackets)?;

    if url_end <= url_start {
        return Err(LinkParseError::InvalidUrlBrackets);
    }

    let url = link_str[(url_start + 1)..url_end].to_string();
    let params_str = &link_str[(url_end + 1)..];

    // Parse parameters
    let mut parameters = HashMap::new();

    for param_pair in params_str.split(';') {
        let param_pair = param_pair.trim();
        if param_pair.is_empty() {
            continue;
        }

        if let Some((key, value)) = parse_parameter(param_pair)? {
            parameters.insert(key, value);
        }
    }

    Ok(LinkEntry { url, parameters })
}

/// Parse a single parameter from link header
///
/// # Arguments
/// * `param_str` - Parameter string (e.g., 'rel="next"')
///
/// # Returns
/// * `Ok(Some((key, value)))` - Parsed parameter
/// * `Ok(None)` - Empty parameter
/// * `Err(LinkParseError)` - Parsing error
fn parse_parameter(param_str: &str) -> Result<Option<(String, String)>, LinkParseError> {
    if param_str.is_empty() {
        return Ok(None);
    }

    let eq_pos = param_str.find('=');

    match eq_pos {
        Some(pos) => {
            let key = param_str[..pos].trim().to_string();
            let value_part = param_str[(pos + 1)..].trim();

            // Remove quotes if present
            let value = if value_part.starts_with('"')
                && value_part.ends_with('"')
                && value_part.len() >= 2
            {
                value_part[1..(value_part.len() - 1)].to_string()
            } else {
                value_part.to_string()
            };

            Ok(Some((key, value)))
        }
        None => {
            // Parameter without value (e.g., just "nofollow")
            Ok(Some((param_str.trim().to_string(), String::new())))
        }
    }
}

/// Extract max_id parameter from Link header
///
/// Common helper for pagination in REST APIs
///
/// # Arguments
/// * `header_value` - Optional HeaderValue from HTTP response
///
/// # Returns
/// * `Ok(Some(String))` - Found max_id value
/// * `Ok(None)` - No max_id found
/// * `Err(LinkParseError)` - Parsing error
#[inline]
pub fn extract_max_id_from_link_header(
    header_value: Option<&HeaderValue>,
) -> Result<Option<String>, LinkParseError> {
    let header_str = match header_value {
        Some(hv) => hv.to_str().map_err(|_| LinkParseError::InvalidUtf8)?,
        None => return Ok(None),
    };

    let links = parse_link_header(header_str)?;

    // Look for max_id in any of the link URLs
    for link in links {
        if let Some(max_id) = extract_max_id_from_url(&link.url) {
            return Ok(Some(max_id));
        }
    }

    Ok(None)
}

/// Extract max_id parameter from a URL query string
///
/// # Arguments
/// * `url` - URL to parse
///
/// # Returns
/// * `Some(String)` - Found max_id value
/// * `None` - No max_id parameter found
#[inline]
fn extract_max_id_from_url(url: &str) -> Option<String> {
    // Find the query string part
    let query_start = url.find('?')?;
    let query = &url[(query_start + 1)..];

    // Parse query parameters
    for param in query.split('&') {
        if let Some((key, value)) = param.split_once('=')
            && key == "max_id"
        {
            return Some(value.to_string());
        }
    }

    None
}

/// Represents a single entry in a Link header
#[derive(Debug, Clone, PartialEq)]
pub struct LinkEntry {
    /// The URL of the link
    pub url: String,
    /// Parameters associated with the link (e.g., rel, type, etc.)
    pub parameters: HashMap<String, String>,
}

impl LinkEntry {
    /// Get the relation type of this link
    #[inline]
    pub fn rel(&self) -> Option<&str> {
        self.parameters.get("rel").map(|s| s.as_str())
    }

    /// Get the media type of this link
    #[inline]
    pub fn media_type(&self) -> Option<&str> {
        self.parameters.get("type").map(|s| s.as_str())
    }

    /// Check if this link has the specified relation
    #[inline]
    pub fn has_rel(&self, rel: &str) -> bool {
        self.rel() == Some(rel)
    }
}

/// Errors that can occur during Link header parsing
#[derive(Debug, thiserror::Error)]
pub enum LinkParseError {
    #[error("Missing URL brackets < >")]
    MissingUrlBrackets,

    #[error("Invalid URL bracket placement")]
    InvalidUrlBrackets,

    #[error("Invalid UTF-8 in header value")]
    InvalidUtf8,

    #[error("Invalid parameter format: {param}")]
    InvalidParameter { param: String },
}

// Tests have been moved to tests/unit/utils/http_parser_test.rs
