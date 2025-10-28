pub fn format_number(number: i64) -> String {
    use numfmt::*;
    let mut formatter = Formatter::default()
        .scales(Scales::metric())
        .precision(Precision::Decimals(0));
    formatter.fmt2(number as f64).to_string()
}

/// Parse a mastodon URL into a mastodon ID with comprehensive error handling
///
/// Converts URLs like "https://mstdn.social/@briannawu"
/// into mastodon IDs like "briannawu@mstdn.social"
///
/// # Arguments
/// * `url` - The URL to parse
///
/// # Returns
/// * `Ok(String)` - The parsed mastodon ID
/// * `Err(UrlParseError)` - Detailed error information
///
/// # Examples
/// ```
/// # use cyrup::helper::parse_user_url;
/// let result = parse_user_url("https://mstdn.social/@briannawu");
/// assert!(result.is_ok());
/// ```
#[inline]
pub fn parse_user_url(url: &str) -> Result<String, UrlParseError> {
    use url::Url;
    // Validate input
    if url.trim().is_empty() {
        return Err(UrlParseError::InvalidFormat(
            "URL cannot be empty".to_string(),
        ));
    }

    // Parse URL with detailed error context
    let parsed = Url::parse(url)
        .map_err(|e| UrlParseError::InvalidFormat(format!("Invalid URL format: {}", e)))?;

    // Extract host
    let host = parsed.host().ok_or(UrlParseError::MissingHost)?;

    // Extract path segments
    let user = parsed
        .path_segments()
        .and_then(|mut segments| segments.next())
        .ok_or(UrlParseError::InvalidUsername)?;

    // Validate username format
    if !user.starts_with('@') {
        return Err(UrlParseError::InvalidUsername);
    }

    // Extract username without @ prefix
    let username = &user[1..];

    // Validate username is not empty
    if username.is_empty() {
        return Err(UrlParseError::InvalidUsername);
    }

    // Zero-allocation formatting where possible
    Ok(format!("{username}@{host}"))
}

/// Errors that can occur during URL parsing
#[derive(Debug, thiserror::Error)]
pub enum UrlParseError {
    #[error("Invalid URL format: {0}")]
    InvalidFormat(String),

    #[error("Missing host in URL")]
    MissingHost,

    #[error("Invalid username format")]
    InvalidUsername,
}

mod clean_html_content {
    use html5gum::{HtmlString, Token, Tokenizer};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    pub enum HtmlItem {
        Mention { url: String, name: String },
        Hashtag { name: String },
        Link { url: String, name: String },
        Text { content: String },
        Image { url: String },
        Break,
    }

    impl std::fmt::Display for HtmlItem {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                HtmlItem::Mention { name, .. } => write!(f, "@{name}"),
                HtmlItem::Hashtag { name } => write!(f, "#{name}"),
                HtmlItem::Link { name, .. } => write!(f, "{name}"),
                HtmlItem::Text { content } => write!(f, "{content}"),
                HtmlItem::Image { .. } => write!(f, "[Image]"),
                HtmlItem::Break => writeln!(f),
            }
        }
    }

    pub fn clean_html(html: &str) -> (String, Vec<HtmlItem>) {
        let mut text = String::new();

        let mut collected = Vec::new();

        let mut last_text = String::new();
        let mut last_url: Option<String> = None;
        let mut is_href = false;

        let attr_href = HtmlString("href".as_bytes().to_owned());
        let attr_src = HtmlString("src".as_bytes().to_owned());

        for token in Tokenizer::new(html) {
            match token {
                Ok(Token::StartTag(tag)) => {
                    let name = std::str::from_utf8(&tag.name.0);
                    match name {
                        Ok("a") => {
                            is_href = true;
                            last_text.clear();
                            if let Some(href) = tag
                                .attributes
                                .get(&attr_href)
                                .and_then(|e| std::str::from_utf8(&e.0).ok())
                            {
                                last_url = Some(href.to_string());
                            }
                        }
                        Ok("br") => {
                            collected.push(HtmlItem::Break);
                            text.push(' ');
                            last_text.push(' ');
                        }
                        Ok("img") => {
                            if let Some(url) = tag
                                .attributes
                                .get(&attr_src)
                                .and_then(|e| std::str::from_utf8(&e.0).ok())
                            {
                                collected.push(HtmlItem::Image {
                                    url: url.to_string(),
                                });
                            }
                            text.push(' ');
                            last_text.push(' ');
                        }
                        _ => (),
                    }
                }
                Ok(Token::EndTag(tag)) => {
                    let name = std::str::from_utf8(&tag.name.0);
                    match name {
                        Ok("a") => {
                            let Some(url) = last_url.take() else { continue };
                            let name = last_text.clone();
                            match last_text.chars().next() {
                                Some('@') => collected.push(HtmlItem::Mention { url, name }),
                                Some('#') => collected.push(HtmlItem::Hashtag { name }),
                                _ => collected.push(HtmlItem::Link { url, name }),
                            }

                            last_text.clear();
                            is_href = false;
                        }
                        Ok("p") => {
                            collected.push(HtmlItem::Break);
                            collected.push(HtmlItem::Break);
                            text.push(' ');
                            last_text.push(' ');
                        }
                        _ => (),
                    }
                }
                Ok(Token::String(s)) => {
                    let sx = std::str::from_utf8(&s.0).unwrap_or_default();
                    text.push_str(sx);
                    last_text.push_str(sx);
                    if !is_href {
                        collected.push(HtmlItem::Text {
                            content: sx.to_string(),
                        })
                    }
                }
                Ok(Token::Comment(_)) => (),
                Ok(Token::Doctype(_)) => (),
                Ok(Token::Error(_)) => (),
                Err(_) => (), // Handle tokenization errors
            }
        }

        // if we have > 0 brs at the end, remove them
        let mut brs = 0;
        for n in collected.iter().rev() {
            if matches!(n, HtmlItem::Break) {
                brs += 1;
            } else {
                break;
            }
        }
        for _ in 0..brs {
            collected.remove(collected.len() - 1);
        }

        (text, collected)
    }
}

pub use clean_html_content::{HtmlItem, clean_html};
