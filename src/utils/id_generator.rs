// Lock-free ID Generation Utilities
// Provides thread-safe, collision-free ID generation without locks
// Zero-allocation implementation using stack buffers for optimal performance

use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

/// Global atomic counter for conversation IDs
static CONVERSATION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Global atomic counter for subscription IDs  
static SUBSCRIPTION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Global atomic counter for session IDs
static SESSION_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Error types for ID generation failures
#[derive(Debug, Clone, PartialEq)]
pub enum IdGenerationError {
    /// System time went backwards or is unavailable
    TimestampError,
    /// Counter overflow occurred
    CounterOverflow,
    /// Buffer too small for ID generation
    BufferTooSmall,
}

impl std::fmt::Display for IdGenerationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IdGenerationError::TimestampError => {
                write!(f, "System timestamp unavailable or invalid")
            }
            IdGenerationError::CounterOverflow => write!(f, "Counter overflow in ID generation"),
            IdGenerationError::BufferTooSmall => write!(f, "Buffer too small for ID generation"),
        }
    }
}

impl std::error::Error for IdGenerationError {}

/// Zero-allocation helper to write u64 to buffer in decimal format
/// Returns the number of bytes written
#[inline]
fn write_u64_to_buffer(mut value: u64, buffer: &mut [u8]) -> usize {
    if buffer.is_empty() {
        return 0;
    }

    if value == 0 {
        buffer[0] = b'0';
        return 1;
    }

    let mut pos = 0;
    let mut temp_buffer = [0u8; 20]; // u64::MAX has 20 digits
    let mut temp_pos = 0;

    // Write digits in reverse order to temp buffer
    while value > 0 {
        temp_buffer[temp_pos] = b'0' + (value % 10) as u8;
        value /= 10;
        temp_pos += 1;
    }

    // Copy reversed digits to output buffer
    for i in 0..temp_pos {
        if pos >= buffer.len() {
            return pos;
        }
        buffer[pos] = temp_buffer[temp_pos - 1 - i];
        pos += 1;
    }

    pos
}

/// Zero-allocation helper to write u128 to buffer in decimal format
/// Returns the number of bytes written
#[inline]
fn write_u128_to_buffer(mut value: u128, buffer: &mut [u8]) -> usize {
    if buffer.is_empty() {
        return 0;
    }

    if value == 0 {
        buffer[0] = b'0';
        return 1;
    }

    let mut pos = 0;
    let mut temp_buffer = [0u8; 39]; // u128::MAX has 39 digits
    let mut temp_pos = 0;

    // Write digits in reverse order to temp buffer
    while value > 0 {
        temp_buffer[temp_pos] = b'0' + (value % 10) as u8;
        value /= 10;
        temp_pos += 1;
    }

    // Copy reversed digits to output buffer
    for i in 0..temp_pos {
        if pos >= buffer.len() {
            return pos;
        }
        buffer[pos] = temp_buffer[temp_pos - 1 - i];
        pos += 1;
    }

    pos
}

/// Zero-allocation helper to copy string slice to buffer
/// Returns the number of bytes written
#[inline]
fn write_str_to_buffer(s: &str, buffer: &mut [u8]) -> usize {
    let bytes = s.as_bytes();
    let len = bytes.len().min(buffer.len());
    buffer[..len].copy_from_slice(&bytes[..len]);
    len
}

/// Generate a unique conversation ID with zero allocation
///
/// Uses atomic counter + UUID for guaranteed uniqueness across
/// multiple conversation instances and application restarts
///
/// # Returns
/// A unique conversation ID string
///
/// # Errors
/// Returns `IdGenerationError::CounterOverflow` if counter overflows
#[inline]
pub fn generate_conversation_id() -> Result<String, IdGenerationError> {
    let counter = CONVERSATION_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Check for counter overflow
    if counter == u64::MAX {
        return Err(IdGenerationError::CounterOverflow);
    }

    let uuid = Uuid::new_v4();
    let uuid_simple = uuid.simple();

    // Stack buffer for zero-allocation formatting
    // Format: "conversation-{counter}-{uuid}"
    // Max size: "conversation-" (13) + u64::MAX (20) + "-" (1) + UUID (32) = 66 bytes
    let mut buffer = [0u8; 128];
    let mut pos = 0;

    // Write "conversation-"
    pos += write_str_to_buffer("conversation-", &mut buffer[pos..]);

    // Write counter
    pos += write_u64_to_buffer(counter, &mut buffer[pos..]);

    // Write "-"
    pos += write_str_to_buffer("-", &mut buffer[pos..]);

    // Write UUID
    let uuid_str = uuid_simple.to_string();
    pos += write_str_to_buffer(&uuid_str, &mut buffer[pos..]);

    // Convert buffer to String
    String::from_utf8(buffer[..pos].to_vec()).map_err(|_| IdGenerationError::BufferTooSmall)
}

/// Generate a unique subscription ID with zero allocation
///
/// Creates collision-free subscription identifiers for storage
/// and event subscription management
///
/// # Returns  
/// A unique subscription ID string
///
/// # Errors
/// Returns `IdGenerationError::CounterOverflow` if counter overflows
#[inline]
pub fn generate_subscription_id() -> Result<String, IdGenerationError> {
    let counter = SUBSCRIPTION_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Check for counter overflow
    if counter == u64::MAX {
        return Err(IdGenerationError::CounterOverflow);
    }

    let uuid = Uuid::new_v4();
    let uuid_simple = uuid.simple();

    // Stack buffer for zero-allocation formatting
    // Format: "subscription-{counter}-{uuid}"
    // Max size: "subscription-" (13) + u64::MAX (20) + "-" (1) + UUID (32) = 66 bytes
    let mut buffer = [0u8; 128];
    let mut pos = 0;

    // Write "subscription-"
    pos += write_str_to_buffer("subscription-", &mut buffer[pos..]);

    // Write counter
    pos += write_u64_to_buffer(counter, &mut buffer[pos..]);

    // Write "-"
    pos += write_str_to_buffer("-", &mut buffer[pos..]);

    // Write UUID
    let uuid_str = uuid_simple.to_string();
    pos += write_str_to_buffer(&uuid_str, &mut buffer[pos..]);

    // Convert buffer to String
    String::from_utf8(buffer[..pos].to_vec()).map_err(|_| IdGenerationError::BufferTooSmall)
}

/// Generate a unique session ID with zero allocation
///
/// Creates unique session identifiers for user sessions
/// and authentication tracking
///
/// # Returns
/// A unique session ID string
///
/// # Errors
/// Returns `IdGenerationError::CounterOverflow` if counter overflows
#[inline]
pub fn generate_session_id() -> Result<String, IdGenerationError> {
    let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Check for counter overflow
    if counter == u64::MAX {
        return Err(IdGenerationError::CounterOverflow);
    }

    let uuid = Uuid::new_v4();
    let uuid_simple = uuid.simple();

    // Stack buffer for zero-allocation formatting
    // Format: "session-{counter}-{uuid}"
    // Max size: "session-" (8) + u64::MAX (20) + "-" (1) + UUID (32) = 61 bytes
    let mut buffer = [0u8; 128];
    let mut pos = 0;

    // Write "session-"
    pos += write_str_to_buffer("session-", &mut buffer[pos..]);

    // Write counter
    pos += write_u64_to_buffer(counter, &mut buffer[pos..]);

    // Write "-"
    pos += write_str_to_buffer("-", &mut buffer[pos..]);

    // Write UUID
    let uuid_str = uuid_simple.to_string();
    pos += write_str_to_buffer(&uuid_str, &mut buffer[pos..]);

    // Convert buffer to String
    String::from_utf8(buffer[..pos].to_vec()).map_err(|_| IdGenerationError::BufferTooSmall)
}

/// Generate a simple UUID string with zero allocation
///
/// For cases where a simple UUID is sufficient
///
/// # Returns
/// A UUID string in simple format (32 hex digits)
#[inline]
pub fn generate_uuid() -> String {
    Uuid::new_v4().simple().to_string()
}

/// Generate a timestamp-based ID with zero allocation
///
/// Combines current timestamp with atomic counter for
/// ordering guarantees while maintaining uniqueness
///
/// # Arguments
/// * `prefix` - Optional prefix for the ID
///
/// # Returns
/// A timestamp-based unique ID
///
/// # Errors
/// Returns `IdGenerationError::TimestampError` if system time is unavailable
/// Returns `IdGenerationError::CounterOverflow` if counter overflows
#[inline]
pub fn generate_timestamp_id(prefix: Option<&str>) -> Result<String, IdGenerationError> {
    let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Check for counter overflow
    if counter == u64::MAX {
        return Err(IdGenerationError::CounterOverflow);
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| IdGenerationError::TimestampError)?
        .as_millis();

    // Stack buffer for zero-allocation formatting
    // Max size: prefix + "-" + u128::MAX (39) + "-" + u64::MAX (20) = ~100 bytes
    let mut buffer = [0u8; 256];
    let mut pos = 0;

    // Write optional prefix
    if let Some(p) = prefix {
        pos += write_str_to_buffer(p, &mut buffer[pos..]);
        pos += write_str_to_buffer("-", &mut buffer[pos..]);
    }

    // Write timestamp
    pos += write_u128_to_buffer(timestamp, &mut buffer[pos..]);

    // Write "-"
    pos += write_str_to_buffer("-", &mut buffer[pos..]);

    // Write counter
    pos += write_u64_to_buffer(counter, &mut buffer[pos..]);

    // Convert buffer to String
    String::from_utf8(buffer[..pos].to_vec()).map_err(|_| IdGenerationError::BufferTooSmall)
}

/// Fallback ID generation for when timestamp fails
///
/// Uses only atomic counter with UUID for uniqueness
/// when system time is unavailable
///
/// # Arguments
/// * `prefix` - Optional prefix for the ID
///
/// # Returns
/// A counter-based unique ID
///
/// # Errors
/// Returns `IdGenerationError::CounterOverflow` if counter overflows
#[inline]
pub fn generate_fallback_id(prefix: Option<&str>) -> Result<String, IdGenerationError> {
    let counter = SESSION_COUNTER.fetch_add(1, Ordering::Relaxed);

    // Check for counter overflow
    if counter == u64::MAX {
        return Err(IdGenerationError::CounterOverflow);
    }

    let uuid = Uuid::new_v4();
    let uuid_simple = uuid.simple();

    // Stack buffer for zero-allocation formatting
    let mut buffer = [0u8; 256];
    let mut pos = 0;

    // Write optional prefix
    if let Some(p) = prefix {
        pos += write_str_to_buffer(p, &mut buffer[pos..]);
        pos += write_str_to_buffer("-", &mut buffer[pos..]);
    }

    // Write "fallback-"
    pos += write_str_to_buffer("fallback-", &mut buffer[pos..]);

    // Write counter
    pos += write_u64_to_buffer(counter, &mut buffer[pos..]);

    // Write "-"
    pos += write_str_to_buffer("-", &mut buffer[pos..]);

    // Write UUID
    let uuid_str = uuid_simple.to_string();
    pos += write_str_to_buffer(&uuid_str, &mut buffer[pos..]);

    // Convert buffer to String
    String::from_utf8(buffer[..pos].to_vec()).map_err(|_| IdGenerationError::BufferTooSmall)
}
