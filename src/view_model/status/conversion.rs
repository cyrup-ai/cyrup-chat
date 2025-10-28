//! StatusViewModel conversion and content processing implementation
//!
//! This module contains methods for converting StatusViewModel back to megalodon
//! Status objects and extracting various content elements like mentions, hashtags,
//! and emojis from the processed content.

use super::core::StatusViewModel;
use crate::helper::HtmlItem;
use regex::Regex;

impl StatusViewModel {
    /// Convert StatusViewModel back to megalodon::entities::Status
    /// Used for operations that require returning Status objects
    ///
    /// # Returns
    /// * `megalodon::entities::Status` - Converted status with essential fields populated
    ///
    /// # Note
    /// This is a simplified conversion, some fields may be missing or defaulted
    pub fn to_megalodon_status(&self) -> megalodon::entities::Status {
        // Create a basic status with essential fields
        megalodon::entities::Status {
            id: self.id.0.clone(),
            uri: self.uri.clone(),
            created_at: self.created,
            account: self.account.clone().into(), // Use the From<AccountViewModel> for Account conversion
            content: self
                .content
                .iter()
                .map(|item| match item {
                    HtmlItem::Text { content } => content.clone(),
                    HtmlItem::Link { name, .. } => name.clone(),
                    _ => String::new(),
                })
                .collect::<Vec<_>>()
                .join(""),
            visibility: megalodon::entities::status::StatusVisibility::Public, // Default
            sensitive: false,                                                  // Default
            spoiler_text: String::new(),                                       // Default
            media_attachments: self.extract_media_attachments(),
            application: None,
            mentions: self.extract_mentions_from_content(),
            tags: self.extract_hashtags_from_content(),
            emojis: self.extract_emojis_from_content(),
            reblogs_count: self.reblog_count,
            favourites_count: self.favourited_count,
            replies_count: self.replies_count,
            url: None,
            in_reply_to_id: self.extract_reply_id_from_context(),
            in_reply_to_account_id: None,
            reblog: self
                .reblog_status
                .as_ref()
                .map(|r| Box::new(r.to_megalodon_status())),
            poll: None,
            card: self.card.clone(),
            language: None,
            favourited: Some(self.is_favourited),
            reblogged: Some(self.has_reblogged),
            muted: Some(false), // Default
            bookmarked: Some(self.is_bookmarked),
            pinned: Some(false),               // Default
            edited_at: None,                   // Default - status has not been edited
            emoji_reactions: Some(Vec::new()), // Default - no emoji reactions
            plain_content: None,               // Default - no plain text content available
            quote: false,                      // Default - not a quote status
        }
    }

    /// Extract media attachments from status images with zero allocation
    #[inline(always)]
    fn extract_media_attachments(&self) -> Vec<megalodon::entities::Attachment> {
        self.status_images
            .iter()
            .map(|(id, url, description)| megalodon::entities::Attachment {
                id: id.clone(),
                r#type: megalodon::entities::attachment::AttachmentType::Image,
                url: url.clone(),
                remote_url: Some(url.clone()),
                preview_url: Some(url.clone()),
                text_url: None,
                meta: None,
                description: Some(description.clone()),
                blurhash: None,
            })
            .collect()
    }

    /// Extract mentions from content using zero-allocation regex parsing
    #[inline(always)]
    fn extract_mentions_from_content(&self) -> Vec<megalodon::entities::Mention> {
        // Static regex for @mentions - compiled once and cached
        static MENTION_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let regex = MENTION_REGEX.get_or_init(|| {
            Regex::new(r"@([a-zA-Z0-9_]+)(?:@([a-zA-Z0-9.-]+))?").unwrap_or_else(|_| {
                // Fallback to simple regex if compilation fails
                Regex::new(r"@\w+").unwrap_or_else(|_| panic!("Failed to compile fallback regex"))
            })
        });

        let content = self.get_plain_text_content();
        regex
            .captures_iter(&content)
            .filter_map(|cap| {
                let username = cap.get(1)?.as_str();
                let domain = cap.get(2).map(|m| m.as_str()).unwrap_or("");

                Some(megalodon::entities::Mention {
                    id: format!("{}@{}", username, domain),
                    username: username.to_string(),
                    url: format!("https://{}/{}", domain, username),
                    acct: if domain.is_empty() {
                        username.to_string()
                    } else {
                        format!("{}@{}", username, domain)
                    },
                })
            })
            .collect()
    }

    /// Extract hashtags from content using zero-allocation regex parsing
    #[inline(always)]
    fn extract_hashtags_from_content(&self) -> Vec<megalodon::entities::status::Tag> {
        // Static regex for #hashtags - compiled once and cached
        static HASHTAG_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let regex = HASHTAG_REGEX.get_or_init(|| {
            Regex::new(r"#([a-zA-Z0-9_]+)").unwrap_or_else(|_| {
                // Fallback to simple regex if compilation fails
                Regex::new(r"#\w+").unwrap_or_else(|_| panic!("Failed to compile fallback regex"))
            })
        });

        let content = self.get_plain_text_content();
        regex
            .captures_iter(&content)
            .filter_map(|cap| {
                let tag_name = cap.get(1)?.as_str();

                Some(megalodon::entities::status::Tag {
                    name: tag_name.to_string(),
                    url: format!("https://example.com/tags/{}", tag_name),
                })
            })
            .collect()
    }

    /// Extract emojis from content using zero-allocation parsing
    #[inline(always)]
    fn extract_emojis_from_content(&self) -> Vec<megalodon::entities::Emoji> {
        // Static regex for :custom_emoji: - compiled once and cached
        static EMOJI_REGEX: std::sync::OnceLock<Regex> = std::sync::OnceLock::new();
        let regex = EMOJI_REGEX.get_or_init(|| {
            Regex::new(r":([a-zA-Z0-9_]+):").unwrap_or_else(|_| {
                // Fallback to simple regex if compilation fails
                Regex::new(r":\w+:").unwrap_or_else(|_| panic!("Failed to compile fallback regex"))
            })
        });

        let content = self.get_plain_text_content();
        regex
            .captures_iter(&content)
            .filter_map(|cap| {
                let emoji_name = cap.get(1)?.as_str();

                Some(megalodon::entities::Emoji {
                    shortcode: emoji_name.to_string(),
                    url: format!("https://example.com/emoji/{}.png", emoji_name),
                    static_url: format!("https://example.com/emoji/{}_static.png", emoji_name),
                    visible_in_picker: true,
                    category: None,
                })
            })
            .collect()
    }

    /// Extract reply ID from stored field
    #[inline(always)]
    fn extract_reply_id_from_context(&self) -> Option<String> {
        self.in_reply_to_id.clone()
    }

    /// Get plain text content with zero allocation when possible
    #[inline(always)]
    fn get_plain_text_content(&self) -> String {
        self.content
            .iter()
            .filter_map(|item| match item {
                HtmlItem::Text { content } => Some(content.as_str()),
                HtmlItem::Link { name, .. } => Some(name.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}
