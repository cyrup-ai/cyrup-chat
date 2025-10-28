//! Parse @mentions from messages for multi-agent room routing
//!
//! Extracts agent names from @mention patterns in user messages.
//! Used for routing messages to specific agents in multi-agent rooms (Phase 7).
//!
//! # Pattern
//! Matches `@agent-name` where agent-name can contain:
//! - Letters (a-z, A-Z)
//! - Numbers (0-9)
//! - Underscores (_)
//! - Hyphens (-)
//!
//! # Example
//! ```
//! use cyrup::services::mention_parser::parse_mentions;
//!
//! let content = "Hey @agent-a and @agent-b, can you help?";
//! let mentions = parse_mentions(content);
//! assert_eq!(mentions, vec!["agent-a", "agent-b"]);
//! ```

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    /// Compiled regex for @mention pattern
    ///
    /// Pattern: @([a-zA-Z0-9_-]+)
    /// - @ = literal @ symbol
    /// - () = capture group for agent name
    /// - [a-zA-Z0-9_-]+ = one or more alphanumeric, underscore, or hyphen
    static ref MENTION_REGEX: Regex = Regex::new(r"@([a-zA-Z0-9_-]+)")
        .expect("Failed to compile mention regex - pattern should be valid");
}

/// Parse @mentions from message content
///
/// Extracts all @mention patterns and returns agent names without @ prefix.
/// Used for routing messages to specific agents in multi-agent rooms.
///
/// # Arguments
/// * `content` - Message text to parse
///
/// # Returns
/// * `Vec<String>` - List of mentioned agent names (without @ prefix)
///
/// # Example
/// ```
/// let content = "Hey @agent-a and @agent-b, can you help?";
/// let mentions = parse_mentions(content);
/// // mentions = vec!["agent-a", "agent-b"]
/// ```
///
/// # Performance
/// Regex is compiled once at program start via lazy_static,
/// so subsequent calls are very fast (no recompilation).
pub fn parse_mentions(content: &str) -> Vec<String> {
    MENTION_REGEX
        .captures_iter(content)
        .filter_map(|cap| cap.get(1).map(|m| m.as_str().to_string()))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_single_mention() {
        let mentions = parse_mentions("Hello @agent-a");
        assert_eq!(mentions, vec!["agent-a"]);
    }

    #[test]
    fn test_multiple_mentions() {
        let mentions = parse_mentions("Hey @agent-a and @agent-b, help!");
        assert_eq!(mentions, vec!["agent-a", "agent-b"]);
    }

    #[test]
    fn test_with_underscores() {
        let mentions = parse_mentions("Ask @my_agent_123");
        assert_eq!(mentions, vec!["my_agent_123"]);
    }

    #[test]
    fn test_no_mentions() {
        let mentions = parse_mentions("Hello world");
        assert!(mentions.is_empty());
    }

    #[test]
    fn test_empty_string() {
        let mentions = parse_mentions("");
        assert!(mentions.is_empty());
    }
}
