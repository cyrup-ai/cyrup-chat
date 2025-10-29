//! Comprehensive datetime utilities for SurrealDB integration
//! 
//! Provides timezone-safe datetime handling, serialization helpers,
//! and SurrealDB-specific datetime operations.

use chrono::{DateTime, Utc, Duration};
use thiserror::Error;

/// Error types for datetime operations
#[derive(Error, Debug)]
pub enum DateTimeError {
    #[error("Invalid datetime format: {0}")]
    ParseError(String),
    #[error("Non-UTC timezone detected: {0}")]
    NonUtcTimezone(String),
    #[error("SurrealDB conversion failed: {0}")]
    SurrealConversionError(String),
    #[error("Datetime validation failed: {0}")]
    ValidationError(String),
}

pub type DateTimeResult<T> = Result<T, DateTimeError>;

/// Get validated current UTC datetime with microsecond precision
pub fn now_utc() -> DateTime<Utc> {
    Utc::now()
}

/// Convert chrono DateTime to SurrealDB-compatible RFC3339 format
/// 
/// SurrealDB expects ISO 8601 format with timezone suffix
pub fn to_surreal_format(dt: &DateTime<Utc>) -> String {
    dt.to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

/// Parse SurrealDB datetime string to chrono DateTime
/// 
/// Performs validation to ensure UTC timezone after parsing
pub fn from_surreal_format(s: &str) -> DateTimeResult<DateTime<Utc>> {
    if s.is_empty() {
        return Err(DateTimeError::ParseError("Empty datetime string".to_string()));
    }
    
    DateTime::parse_from_rfc3339(s)
        .map_err(|e| DateTimeError::ParseError(format!("RFC3339 parsing failed: {}", e)))
        .and_then(|dt| {
            let utc_dt = dt.with_timezone(&Utc);
            validate_utc(utc_dt)
        })
}

/// Validate that datetime is in UTC timezone
/// 
/// Additional validation beyond timezone to ensure reasonable datetime values
pub fn validate_utc(dt: DateTime<Utc>) -> DateTimeResult<DateTime<Utc>> {
    let now = Utc::now();
    let max_future = now + Duration::hours(24); // Allow up to 24 hours in future for edge cases
    let min_past = now - Duration::days(365 * 10); // Allow back 10 years
    
    if dt > max_future {
        return Err(DateTimeError::ValidationError(format!(
            "Datetime {} is too far in the future (max: {})", 
            dt, max_future
        )));
    }
    
    if dt < min_past {
        return Err(DateTimeError::ValidationError(format!(
            "Datetime {} is too old (min: {})", 
            dt, min_past
        )));
    }
    
    Ok(dt)
}

/// Format datetime for human-readable display
/// 
/// Returns localized string representation suitable for UI display
pub fn format_for_display(dt: &DateTime<Utc>) -> String {
    // Format as "2024-01-15 14:30:45 UTC"
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Calculate relative time description (e.g., "2 minutes ago")
/// 
/// Returns human-readable relative time from now
pub fn format_relative_time(dt: &DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now - *dt;
    
    if duration.num_seconds() < 60 {
        format!("{} seconds ago", duration.num_seconds())
    } else if duration.num_minutes() < 60 {
        format!("{} minutes ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{} hours ago", duration.num_hours())
    } else {
        format!("{} days ago", duration.num_days())
    }
}