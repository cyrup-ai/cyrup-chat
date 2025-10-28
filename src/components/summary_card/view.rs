//! Summary card component - shows at top of conversation
//!
//! Displays conversation summary and context transparency to show users
//! what information the agent has access to. Collapsible to save space.

use chrono::{DateTime, Utc};
use dioxus::prelude::*;

/// Summary card component displaying conversation context
///
/// Shows collapsible card at top of conversation with:
/// - Conversation title with emoji
/// - Summary text (what agent sees)
/// - Last updated timestamp (relative time)
/// - Expand/collapse button
///
/// # Props
/// * `title` - Conversation title
/// * `summary` - Generated summary text
/// * `last_updated` - DateTime when summary was last updated
///
/// # Example
/// ```rust
/// SummaryCard {
///     title: conversation.title,
///     summary: conversation.summary,
///     last_updated: conversation.last_message_at,
/// }
/// ```
///
/// # Design
/// Follows Dioxus 0.7 patterns:
/// - `#[component]` attribute for component declaration
/// - `use_signal()` for local reactive state
/// - `rsx!` macro for JSX-like template syntax
/// - Props as function parameters
///
/// References:
/// - [src/components/status_timeline/view.rs:9-48](../../status_timeline/view.rs)
/// - [src/components/conversation/view/main_component.rs:13-63](../../conversation/view/main_component.rs)
#[component]
pub fn SummaryCard(title: String, summary: String, last_updated: DateTime<Utc>) -> Element {
    // Local state for collapse/expand - starts collapsed to save space
    let mut collapsed = use_signal(|| false);

    rsx! {
        div {
            class: "summary-card m-3 p-3 border rounded",

            // Header with title and collapse button
            div {
                class: "summary-header d-flex justify-between items-center",
                onclick: move |_| collapsed.set(!collapsed()),

                h3 {
                    class: "m-0",
                    "📋 {title}"
                }

                button {
                    class: "btn btn-sm",
                    // Toggle between + (collapsed) and − (expanded)
                    if *collapsed.read() { "+" } else { "−" }
                }
            }

            // Content area (only shown when not collapsed)
            if !*collapsed.read() {
                div {
                    class: "summary-content mt-2",

                    p {
                        class: "summary-text",
                        "{summary}"
                    }

                    span {
                        class: "text-muted small",
                        "Last updated: {format_time_ago(last_updated)}"
                    }
                }
            }
        }
    }
}

/// Format DateTime as relative time string
///
/// Converts absolute DateTime to human-readable relative time:
/// - "just now" for < 1 minute
/// - "X minutes ago" for < 1 hour
/// - "X hours ago" for < 24 hours
/// - "X days ago" for ≥ 24 hours
///
/// # Arguments
/// * `dt` - DateTime to format
///
/// # Returns
/// Human-readable relative time string
///
/// # Pattern
/// Based on chrono duration patterns from:
/// [src/environment/native/platform.rs:237-244](../../../environment/native/platform.rs)
#[allow(dead_code)] // Used in RSX macro - compiler doesn't detect usage through Dioxus interpolation
fn format_time_ago(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(dt);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        let mins = duration.num_minutes();
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if duration.num_hours() < 24 {
        let hours = duration.num_hours();
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else {
        let days = duration.num_days();
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    }
}
