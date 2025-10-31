//! Conversation summarization service with rate limiting
//!
//! Provides SummarizerService that generates rolling summaries and titles
//! using ephemeral Claude agents with 60-second rate limiting per conversation.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use surrealdb_types::{RecordId, ToSql};

use crate::database::Database;

/// Rate limiter for conversation summarization
///
/// Enforces minimum 60-second interval between summarization runs
/// to prevent excessive API usage and costs.
struct RateLimiter {
    /// Last time summarization ran for this conversation
    last_run: Option<Instant>,
    /// Whether summarization is currently in progress
    in_progress: bool,
}

impl RateLimiter {
    /// Create new rate limiter (never run before)
    fn new() -> Self {
        Self {
            last_run: None,
            in_progress: false,
        }
    }

    /// Check if summarization can run now
    ///
    /// Returns true if:
    /// - Never run before, OR
    /// - Last run was >60 seconds ago AND not currently running
    fn can_run(&self) -> bool {
        if self.in_progress {
            return false;
        }

        match self.last_run {
            None => true, // Never run before
            Some(last) => last.elapsed() >= Duration::from_secs(60),
        }
    }

    /// Mark summarization as started
    fn mark_started(&mut self) {
        self.in_progress = true;
    }

    /// Mark summarization as finished
    fn mark_finished(&mut self) {
        self.last_run = Some(Instant::now());
        self.in_progress = false;
    }
}

/// Summarizer output containing title and summary
#[derive(Debug, Clone)]
pub struct SummarizerOutput {
    /// Generated conversation title (max 50 chars)
    pub title: String,
    /// Generated conversation summary (max 8000 tokens)
    pub summary: String,
}

/// Summarization service with rate limiting
///
/// Generates conversation titles and rolling summaries using ephemeral
/// Claude agents with 60-second rate limiting per conversation.
pub struct SummarizerService {
    /// Database connection
    db: Arc<Database>,
    /// Rate limiters per conversation_id (using string for HashMap key)
    rate_limiters: Mutex<HashMap<String, RateLimiter>>,
}

impl SummarizerService {
    /// Create new summarizer service
    ///
    /// # Arguments
    /// * `db` - Shared database connection
    pub fn new(db: Arc<Database>) -> Self {
        Self {
            db,
            rate_limiters: Mutex::new(HashMap::new()),
        }
    }

    /// Maybe run summarization if rate limit allows
    ///
    /// # Arguments
    /// * `conversation_id` - Conversation to summarize
    ///
    /// # Returns
    /// * `Ok(Some(output))` - Summarization ran successfully
    /// * `Ok(None)` - Skipped due to rate limit
    /// * `Err(msg)` - Summarization failed
    ///
    /// # Rate Limiting
    /// Enforces 60-second minimum interval. If called within cooldown period,
    /// returns Ok(None) immediately without spawning agent.
    pub async fn maybe_summarize(
        &self,
        conversation_id: &RecordId,
    ) -> Result<Option<SummarizerOutput>, String> {
        let conversation_id_str = conversation_id.to_sql();
        
        // Check rate limit
        let mut limiters = self.rate_limiters.lock().await;
        let limiter = limiters
            .entry(conversation_id_str.clone())
            .or_insert_with(RateLimiter::new);

        if !limiter.can_run() {
            return Ok(None); // Skip this run
        }

        limiter.mark_started();
        drop(limiters); // Release lock before async work

        // Run summarization
        let result = self.run_summarizer(conversation_id).await;

        // Mark finished (even if failed)
        let mut limiters = self.rate_limiters.lock().await;
        if let Some(limiter) = limiters.get_mut(&conversation_id_str) {
            limiter.mark_finished();
        }

        result.map(Some)
    }

    /// Run summarizer agent and update database
    ///
    /// # Steps
    /// 1. Get conversation and messages from database
    /// 2. Build prompt with previous summary and new messages
    /// 3. Spawn ephemeral agent with query()
    /// 4. Parse response for TITLE and SUMMARY
    /// 5. Update database with new summary and title
    ///
    /// # Error Handling
    /// Returns error if:
    /// - Database queries fail
    /// - Agent spawn fails
    /// - Response parsing fails
    /// - Database update fails
    async fn run_summarizer(&self, conversation_id: &RecordId) -> Result<SummarizerOutput, String> {
        // Get conversation and messages
        let conversation = self
            .db
            .get_conversation(conversation_id)
            .await
            .map_err(|e| format!("Failed to get conversation: {}", e))?;

        let messages = self
            .db
            .get_recent_messages(conversation_id)
            .await
            .map_err(|e| format!("Failed to get messages: {}", e))?;

        // Build messages text for prompt
        let messages_text = messages
            .iter()
            .map(|m| format!("{}: {}", m.author, m.content))
            .collect::<Vec<_>>()
            .join("\n\n");

        // Build prompt
        let prompt = format!(
            r#"You are a conversation summarizer. Generate:
1. A concise title (max 50 chars)
2. A comprehensive summary (max 8000 tokens)

Previous summary: {}

Messages to summarize:
{}

Respond in this exact format:
TITLE: <title here>
SUMMARY: <summary here>"#,
            conversation.summary, messages_text
        );

        // Spawn ephemeral agent
        let options = kodegen_tools_claude_agent::ClaudeAgentOptions::builder()
            .system_prompt("You are a conversation summarizer. Extract key points and generate concise titles.")
            .max_turns(1)
            .build();

        let stream = kodegen_tools_claude_agent::query(&prompt, Some(options))
            .await
            .map_err(|e| format!("Failed to spawn agent: {}", e))?;

        let mut stream = Box::pin(stream);

        // Collect response from agent
        let mut response = String::new();
        use futures_util::StreamExt;

        while let Some(message) = stream.next().await {
            match message {
                Ok(kodegen_tools_claude_agent::Message::Assistant { message, .. }) => {
                    // Extract text from content blocks
                    for block in &message.content {
                        if let kodegen_tools_claude_agent::ContentBlock::Text { text } = block {
                            response.push_str(text);
                        }
                    }
                }
                Err(e) => return Err(format!("Agent error: {}", e)),
                _ => {} // Ignore other message types
            }
        }

        // Parse response
        let (title, summary) = parse_summarizer_output(&response)?;

        // Update database
        self.db
            .update_conversation_summary(conversation_id, &summary, &title)
            .await
            .map_err(|e| format!("Failed to update database: {}", e))?;

        Ok(SummarizerOutput { title, summary })
    }
}

/// Parse summarizer agent response
///
/// Expected format:
/// ```text
/// TITLE: Some Title Here
/// SUMMARY: Some summary text here
/// that can span multiple lines
/// ```
///
/// # Arguments
/// * `response` - Raw agent response text
///
/// # Returns
/// * `Ok((title, summary))` - Successfully parsed
/// * `Err(msg)` - Parse error with details
///
/// # Error Cases
/// - No "TITLE:" line found
/// - No "SUMMARY:" line found
/// - Empty title or summary
fn parse_summarizer_output(response: &str) -> Result<(String, String), String> {
    let lines: Vec<&str> = response.lines().collect();

    // Find TITLE line
    let title_line = lines
        .iter()
        .find(|l| l.starts_with("TITLE:"))
        .ok_or("No TITLE in response")?;

    // Find SUMMARY start position
    let summary_start = lines
        .iter()
        .position(|l| l.starts_with("SUMMARY:"))
        .ok_or("No SUMMARY in response")?;

    // Extract title (remove "TITLE:" prefix and trim)
    let title = title_line.trim_start_matches("TITLE:").trim().to_string();

    if title.is_empty() {
        return Err("Title is empty".to_string());
    }

    // Validate title length (max 50 chars)
    if title.len() > 50 {
        return Err(format!("Title too long: {} chars (max 50)", title.len()));
    }

    // Extract summary (all lines from SUMMARY onwards, joined)
    let summary = lines[summary_start..]
        .join("\n")
        .trim_start_matches("SUMMARY:")
        .trim()
        .to_string();

    if summary.is_empty() {
        return Err("Summary is empty".to_string());
    }

    Ok((title, summary))
}
