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
//! library.add_directory("./.swissarmyhammer/prompts")?;
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

/// Prompt filtering functionality
pub mod prompt_filter;

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

/// Advanced search functionality
pub mod search_advanced;

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

/// Unified file system utilities for better error handling and testing
pub mod fs_utils;

// Re-export core types
pub use file_loader::FileSource;
pub use fs_utils::{FileSystem, FileSystemUtils};
pub use plugins::{CustomLiquidFilter, PluginRegistry, SwissArmyHammerPlugin};
pub use prompt_filter::PromptFilter;
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
pub mod error;

pub use error::{ErrorChainExt, ErrorContext, Result, SwissArmyHammerError};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        CustomLiquidFilter, FileSystem, FileSystemUtils, PluginRegistry, Prompt, PromptLibrary,
        PromptLoader, PromptStorage, Result, StorageBackend, SwissArmyHammerError,
        SwissArmyHammerPlugin, Template, TemplateEngine,
    };

    pub use crate::mcp::McpServer;
    pub use crate::search::{SearchEngine, SearchResult};
    pub use crate::search_advanced::{
        generate_excerpt, AdvancedSearchEngine, AdvancedSearchOptions, AdvancedSearchResult,
    };
    pub use crate::workflow::{
        State, StateId, Transition, Workflow, WorkflowName, WorkflowRun, WorkflowRunId,
        WorkflowRunStatus,
    };
}

/// Test utilities module for testing support
pub mod test_utils;
