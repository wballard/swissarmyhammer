//! Common utilities module
//!
//! This module provides shared utilities to eliminate code duplication
//! throughout the SwissArmyHammer codebase.

/// Error handling utilities and context helpers
pub mod error_context;

/// Environment variable loading utilities  
pub mod env_loader;

/// File type detection and extension handling
pub mod file_types;

/// MCP error conversion utilities
pub mod mcp_errors;

/// Validation builders and error construction
pub mod validation_builders;

// Re-export commonly used items
pub use env_loader::{load_env_optional, load_env_parsed, load_env_string, EnvLoader};
pub use error_context::{io_error_with_context, io_error_with_message, other_error, IoResultExt};
pub use file_types::{
    extract_base_name, is_any_prompt_file, is_prompt_file, ExtensionMatcher, PROMPT_EXTENSIONS,
};
pub use mcp_errors::{mcp, McpResultExt, ToSwissArmyHammerError};
pub use validation_builders::{quick, ValidationChain, ValidationErrorBuilder, ValidationResult};
