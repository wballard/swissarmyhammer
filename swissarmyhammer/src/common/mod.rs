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
pub use error_context::{io_error_with_context, io_error_with_message, other_error, IoResultExt};
pub use env_loader::{load_env_string, load_env_parsed, load_env_optional, EnvLoader};
pub use file_types::{is_prompt_file, is_any_prompt_file, extract_base_name, ExtensionMatcher, PROMPT_EXTENSIONS};
pub use mcp_errors::{ToSwissArmyHammerError, McpResultExt, mcp};
pub use validation_builders::{ValidationErrorBuilder, ValidationChain, ValidationResult, quick};