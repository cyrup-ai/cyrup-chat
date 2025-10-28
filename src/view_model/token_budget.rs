//! Token budget configuration for context window management
//!
//! Provides model-specific token budgets and message estimation heuristics.
//! Used by database layer to calculate dynamic message retrieval limits.

use super::agent::AgentModel;
use super::message::Message;

/// Token budget configuration for context window management
///
/// Provides model-specific token budgets and message estimation heuristics.
/// Used by database layer to calculate dynamic message retrieval limits.
#[derive(Debug, Clone)]
pub struct TokenBudgetConfig {
    /// Maximum tokens for input context window
    pub max_tokens: usize,

    /// Average characters per token for text estimation
    /// English: ~3.5, Code/JSON: ~2.5
    pub avg_chars_per_token: f64,

    /// Overhead characters per message (metadata, formatting)
    /// Accounts for: author name, timestamp, message type, IDs
    pub metadata_overhead_chars: usize,

    /// Overhead characters per attachment reference
    pub attachment_overhead_chars: usize,

    /// Safety margin multiplier (0.0-1.0)
    /// Applied to prevent context overflow
    pub safety_margin: f64,
}

impl TokenBudgetConfig {
    /// Create token budget for specific Claude model
    ///
    /// Applies model-specific context windows with safety margins:
    /// - Sonnet 3.5: 192k usable * 0.8 safety = ~153k tokens
    /// - Haiku 3: 196k usable * 0.8 safety = ~157k tokens  
    /// - Opus 3: 192k usable * 0.8 safety = ~153k tokens
    pub fn for_model(model: &AgentModel) -> Self {
        let (max_tokens, safety_margin) = match model {
            AgentModel::Sonnet => (192_000, 0.8),
            AgentModel::Haiku => (196_000, 0.8),
            AgentModel::Opus => (192_000, 0.8),
        };

        Self {
            max_tokens,
            avg_chars_per_token: 3.5,
            metadata_overhead_chars: 50,
            attachment_overhead_chars: 100,
            safety_margin,
        }
    }

    /// Estimate token count for a single message
    ///
    /// Accounts for:
    /// - Content length (unicode-aware)
    /// - Author name length
    /// - Metadata overhead (timestamp, IDs, types)
    /// - Attachment references
    pub fn estimate_message_tokens(&self, message: &Message) -> usize {
        // Content characters (unicode-aware)
        let content_chars = message.content.chars().count();

        // Author name characters
        let author_chars = message.author.chars().count();

        // Metadata overhead per message
        let metadata_chars = self.metadata_overhead_chars;

        // Attachment overhead
        let attachment_chars = message.attachments.len() * self.attachment_overhead_chars;

        // Total characters
        let total_chars = content_chars + author_chars + metadata_chars + attachment_chars;

        // Convert to tokens
        let estimated_tokens = (total_chars as f64) / self.avg_chars_per_token;

        estimated_tokens.ceil() as usize
    }

    /// Calculate maximum message count for retrieval
    ///
    /// Uses token budget to determine how many messages fit in context window.
    /// Assumes average message size for estimation (refined by actual usage).
    ///
    /// Algorithm:
    /// 1. Apply safety margin to max_tokens
    /// 2. Estimate chars from safe token budget
    /// 3. Calculate message limit using average message size
    /// 4. Clamp to reasonable bounds (10-1000)
    pub fn calculate_message_limit(&self) -> usize {
        // Apply safety margin
        let safe_token_budget = (self.max_tokens as f64) * self.safety_margin;

        // Convert to character budget
        let char_budget = safe_token_budget * self.avg_chars_per_token;

        // Average message size (content + metadata + attachments)
        let avg_message_chars = 500.0 + self.metadata_overhead_chars as f64;

        // Calculate message limit
        let message_limit = (char_budget / avg_message_chars) as usize;

        // Clamp to reasonable bounds
        message_limit.clamp(10, 1000)
    }

    /// Estimate total tokens for message collection
    ///
    /// More accurate than calculate_message_limit when actual messages available.
    pub fn estimate_total_tokens(&self, messages: &[Message]) -> usize {
        messages
            .iter()
            .map(|msg| self.estimate_message_tokens(msg))
            .sum()
    }
}

impl Default for TokenBudgetConfig {
    fn default() -> Self {
        Self {
            max_tokens: 32_000,
            avg_chars_per_token: 3.5,
            metadata_overhead_chars: 50,
            attachment_overhead_chars: 100,
            safety_margin: 0.8,
        }
    }
}
