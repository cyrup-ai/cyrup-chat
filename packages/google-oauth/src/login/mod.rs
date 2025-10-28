//! OAuth Login Flow Module
//! 
//! This module provides a comprehensive OAuth 2.0 login implementation for Google services
//! with PKCE (Proof Key for Code Exchange) security, secure callback handling, and
//! production-ready error handling.
//! 
//! # Features
//! 
//! - Zero-allocation, non-blocking async implementation  
//! - PKCE enabled by default for enhanced security
//! - Secure memory handling with automatic token cleanup
//! - Comprehensive security headers and CSRF protection
//! - Production-ready error sanitization
//! - Flexible callback handling modes
//! 
//! # Architecture
//! 
//! The module is decomposed into focused submodules:
//! - `builders` - Login builder pattern implementation
//! - `execution` - OAuth flow execution logic
//! - `callback` - HTTP server and callback handling
//! - `security` - Error sanitization and secure response generation
//! 
//! # Usage
//! 
//! ```rust,no_run  
//! use google_oauth::Login;
//! 
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let response = Login::from_env()
//!     .scopes(["userinfo.email", "userinfo.profile"])?
//!     .access_type(google_oauth::AccessType::Offline)
//!     .timeout(std::time::Duration::from_secs(120))
//!     .login()
//!     .await?;
//! 
//! println!("Access token: {}", &*response.access_token);
//! # Ok(())
//! # }
//! ```

pub mod builders;
pub mod callback;
pub mod execution;
pub mod security;

// Re-export the main types for ergonomic public API
pub use builders::{Login, LoginClientSecretBuilder, LoginConfigBuilder, LoginScopesBuilder};