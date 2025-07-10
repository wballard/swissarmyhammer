//! # SwissArmyHammer
//!
//! A flexible prompt management library for AI assistants.
//!
//! ## Features
//!
//! - **Prompt Management**: Load, store, and organize prompts from various sources
//! - **Template Engine**: Powerful Liquid-based template processing
//! - **Search**: Full-text search capabilities for finding prompts
//! - **MCP Support**: Model Context Protocol server integration
//! - **Async/Sync APIs**: Choose between async and sync interfaces
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use swissarmyhammer::{PromptLibrary, PromptStorage};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Create a new prompt library
//! let mut library = PromptLibrary::new();
//!
//! // Add prompts from a directory
//! library.add_directory("./prompts")?;
//!
//! // Get a prompt and render it
//! let prompt = library.get("code-review")?;
//! let args = vec![("language", "rust"), ("file", "main.rs")]
//!     .into_iter()
//!     .map(|(k, v)| (k.to_string(), v.to_string()))
//!     .collect();
//! let rendered = prompt.render(&args)?;
//!
//! println!("{}", rendered);
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

/// Prompt management and storage
pub mod prompts;

/// Prompt loading and resolution
pub mod prompt_resolver;

/// Template engine and rendering
pub mod template;

/// Model Context Protocol (MCP) server support
pub mod mcp;

/// Storage abstractions and implementations
pub mod storage;

/// Search functionality
pub mod search;

/// Plugin system for extensibility
pub mod plugins;

/// Workflow system for state-based execution
pub mod workflow;

/// Security utilities for path validation and resource limits
pub mod security;

/// File watching functionality for prompt directories
pub mod file_watcher;

/// Virtual file system for unified file loading
pub mod file_loader;

/// Directory traversal utilities
pub mod directory_utils;

// Re-export core types
pub use file_loader::FileSource;
pub use plugins::{CustomLiquidFilter, PluginRegistry, SwissArmyHammerPlugin};
pub use prompt_resolver::PromptResolver;
// Re-export FileSource as PromptSource for backward compatibility
pub use file_loader::FileSource as PromptSource;
pub use prompts::{ArgumentSpec, Prompt, PromptLibrary, PromptLoader};
pub use storage::{PromptStorage, StorageBackend};
pub use template::{Template, TemplateEngine};
pub use workflow::{
    State, StateId, Transition, Workflow, WorkflowName, WorkflowRun, WorkflowRunId,
    WorkflowRunStatus,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Error types used throughout the library
pub mod error {
    use thiserror::Error;

    /// Main error type for the library
    #[derive(Debug, Error)]
    pub enum SwissArmyHammerError {
        /// IO operation failed
        #[error("IO error: {0}")]
        Io(#[from] std::io::Error),

        /// Template parsing or rendering failed
        #[error("Template error: {0}")]
        Template(String),

        /// Prompt not found
        #[error("Prompt not found: {0}")]
        PromptNotFound(String),

        /// Invalid configuration
        #[error("Configuration error: {0}")]
        Config(String),

        /// Storage backend error
        #[error("Storage error: {0}")]
        Storage(String),

        /// Workflow not found
        #[error("Workflow not found: {0}")]
        WorkflowNotFound(String),

        /// Workflow run not found
        #[error("Workflow run not found: {0}")]
        WorkflowRunNotFound(String),

        /// Serialization/deserialization error
        #[error("Serialization error: {0}")]
        Serialization(#[from] serde_yaml::Error),

        /// JSON serialization/deserialization error
        #[error("JSON error: {0}")]
        Json(#[from] serde_json::Error),

        /// Other errors
        #[error("{0}")]
        Other(String),
    }

    /// Result type alias
    pub type Result<T> = std::result::Result<T, SwissArmyHammerError>;
}

pub use error::{Result, SwissArmyHammerError};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        CustomLiquidFilter, PluginRegistry, Prompt, PromptLibrary, PromptLoader, PromptStorage,
        Result, StorageBackend, SwissArmyHammerError, SwissArmyHammerPlugin, Template,
        TemplateEngine,
    };

    pub use crate::mcp::McpServer;
    pub use crate::search::{SearchEngine, SearchResult};
    pub use crate::workflow::{
        State, StateId, Transition, Workflow, WorkflowName, WorkflowRun, WorkflowRunId,
        WorkflowRunStatus,
    };
}

/// Test utilities module for testing support
#[doc(hidden)]
pub mod test_utils;
