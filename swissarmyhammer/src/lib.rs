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

/// Template engine and rendering
pub mod template;

/// Model Context Protocol (MCP) server support
#[cfg(feature = "mcp")]
pub mod mcp;

/// Storage abstractions and implementations
pub mod storage;

/// Search functionality
#[cfg(feature = "search")]
pub mod search;

/// Plugin system for extensibility
pub mod plugins;

// Re-export core types
pub use plugins::{CustomLiquidFilter, PluginRegistry, SwissArmyHammerPlugin};
pub use prompts::{ArgumentSpec, Prompt, PromptLibrary, PromptLoader};
pub use storage::{PromptStorage, StorageBackend};
pub use template::{Template, TemplateEngine};

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

        /// Serialization/deserialization error
        #[error("Serialization error: {0}")]
        Serialization(#[from] serde_yaml::Error),

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

    #[cfg(feature = "mcp")]
    pub use crate::mcp::McpServer;

    #[cfg(feature = "search")]
    pub use crate::search::{SearchEngine, SearchResult};
}
