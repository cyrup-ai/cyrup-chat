//! StatusViewModel module with decomposed functionality
//!
//! This module coordinates the StatusViewModel implementation across multiple
//! specialized submodules for better maintainability and code organization.

mod constructor;
mod conversion;
mod core;
mod state_management;
mod thread_analyzer;

// Re-export the main StatusViewModel struct and its functionality
pub use core::StatusViewModel;

// Re-export thread analyzer components for external use
pub use thread_analyzer::{
    ConversationThreadAnalyzer, SurrealThreadAnalyzer, ThreadAnalysisError, ThreadAnalysisResult,
    ThreadAnalyzerFactory, ThreadNode, ThreadRelationType, ThreadStream,
};

// Re-export thread analyzer utility functions
pub use thread_analyzer::utils::{extract_reply_id_from_context, get_thread_stats, is_thread_root};

// The implementation methods are automatically available through the impl blocks
// in the respective modules due to Rust's module system
