//! Unified error handling for the SwissArmyHammer library
//!
//! This module provides a comprehensive error type hierarchy that replaces
//! ad-hoc error handling throughout the codebase with typed, structured errors.

use std::fmt;
use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// The main error type for the SwissArmyHammer library
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum SwissArmyHammerError {
    /// IO operation failed
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

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

    /// Generic error with context
    #[error("{message}")]
    Context {
        message: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
}

/// Workflow-specific errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum WorkflowError {
    /// Workflow not found
    #[error("Workflow '{name}' not found")]
    NotFound { name: String },

    /// Invalid workflow definition
    #[error("Invalid workflow '{name}': {reason}")]
    Invalid { name: String, reason: String },

    /// Circular dependency detected
    #[error("Circular dependency detected: {cycle}")]
    CircularDependency { cycle: String },

    /// State not found in workflow
    #[error("State '{state}' not found in workflow '{workflow}'")]
    StateNotFound { state: String, workflow: String },

    /// Invalid transition
    #[error("Invalid transition from '{from}' to '{to}' in workflow '{workflow}'")]
    InvalidTransition {
        from: String,
        to: String,
        workflow: String,
    },

    /// Workflow execution error
    #[error("Workflow execution failed: {reason}")]
    ExecutionFailed { reason: String },

    /// Timeout during workflow execution
    #[error("Workflow execution timed out after {duration:?}")]
    Timeout { duration: std::time::Duration },
}

/// Action-specific errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ActionError {
    /// Action not found
    #[error("Action '{name}' not found")]
    NotFound { name: String },

    /// Invalid action configuration
    #[error("Invalid action configuration: {reason}")]
    InvalidConfig { reason: String },

    /// Action execution failed
    #[error("Action '{name}' failed: {reason}")]
    ExecutionFailed { name: String, reason: String },

    /// Variable not found in context
    #[error("Variable '{variable}' not found in context")]
    VariableNotFound { variable: String },

    /// Invalid variable name
    #[error("Invalid variable name '{name}': {reason}")]
    InvalidVariableName { name: String, reason: String },

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {message}. Retry after {retry_after:?}")]
    RateLimit {
        message: String,
        retry_after: std::time::Duration,
    },

    /// External command failed
    #[error("External command failed: {command}")]
    CommandFailed { command: String },
}

/// Parsing errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ParseError {
    /// Invalid syntax
    #[error("Invalid syntax at line {line}, column {column}: {message}")]
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },

    /// Missing required field
    #[error("Missing required field '{field}'")]
    MissingField { field: String },

    /// Invalid field value
    #[error("Invalid value for field '{field}': {reason}")]
    InvalidField { field: String, reason: String },

    /// Unsupported format
    #[error("Unsupported format: {format}")]
    UnsupportedFormat { format: String },
}

/// Validation errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ValidationError {
    /// Schema validation failed
    #[error("Schema validation failed: {reason}")]
    Schema { reason: String },

    /// Content validation failed
    #[error("Content validation failed in {file}: {reason}")]
    Content { file: PathBuf, reason: String },

    /// Structure validation failed
    #[error("Structure validation failed: {reason}")]
    Structure { reason: String },

    /// Security validation failed
    #[error("Security validation failed: {reason}")]
    Security { reason: String },
}

/// Storage-related errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum StorageError {
    /// Storage not found
    #[error("Storage '{name}' not found")]
    NotFound { name: String },

    /// Storage already exists
    #[error("Storage '{name}' already exists")]
    AlreadyExists { name: String },

    /// Storage operation failed
    #[error("Storage operation failed: {reason}")]
    OperationFailed { reason: String },

    /// Invalid storage path
    #[error("Invalid storage path: {path}")]
    InvalidPath { path: PathBuf },
}

/// MCP (Model Context Protocol) errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum McpError {
    /// Connection failed
    #[error("MCP connection failed: {reason}")]
    ConnectionFailed { reason: String },

    /// Protocol error
    #[error("MCP protocol error: {reason}")]
    Protocol { reason: String },

    /// Tool execution failed
    #[error("MCP tool '{tool}' failed: {reason}")]
    ToolFailed { tool: String, reason: String },

    /// Resource not found
    #[error("MCP resource '{resource}' not found")]
    ResourceNotFound { resource: String },
}

/// Configuration errors
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ConfigError {
    /// Missing configuration
    #[error("Missing configuration: {name}")]
    Missing { name: String },

    /// Invalid configuration
    #[error("Invalid configuration '{name}': {reason}")]
    Invalid { name: String, reason: String },

    /// Environment variable error
    #[error("Environment variable '{var}' error: {reason}")]
    EnvVar { var: String, reason: String },
}

/// Result type alias for SwissArmyHammer operations
pub type Result<T> = std::result::Result<T, SwissArmyHammerError>;

/// Extension trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add context to an error
    fn context<S: Into<String>>(self, msg: S) -> Result<T>;

    /// Add context with a closure that's only called on error
    fn with_context<F, S>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>;
}

impl<T, E> ErrorContext<T> for std::result::Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context<S: Into<String>>(self, msg: S) -> Result<T> {
        self.map_err(|e| SwissArmyHammerError::Context {
            message: msg.into(),
            source: Box::new(e),
        })
    }

    fn with_context<F, S>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> S,
        S: Into<String>,
    {
        self.map_err(|e| SwissArmyHammerError::Context {
            message: f().into(),
            source: Box::new(e),
        })
    }
}

/// Error chain formatter for detailed error reporting
pub struct ErrorChain<'a>(&'a dyn std::error::Error);

impl<'a> fmt::Display for ErrorChain<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Error: {}", self.0)?;

        let mut current = self.0.source();
        let mut level = 1;

        while let Some(err) = current {
            writeln!(f, "{:indent$}Caused by: {}", "", err, indent = level * 2)?;
            current = err.source();
            level += 1;
        }

        Ok(())
    }
}

/// Extension trait for error types to format the full error chain
pub trait ErrorChainExt {
    /// Format the full error chain
    fn error_chain(&self) -> ErrorChain<'_>;
}

impl<E: std::error::Error> ErrorChainExt for E {
    fn error_chain(&self) -> ErrorChain<'_> {
        ErrorChain(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_context() {
        let err: Result<()> = Err(io::Error::new(io::ErrorKind::NotFound, "file not found").into());
        let err_with_context = err.context("Failed to open config file");

        assert!(err_with_context.is_err());
        let msg = err_with_context.unwrap_err().to_string();
        assert!(msg.contains("Failed to open config file"));
    }

    #[test]
    fn test_error_chain_display() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let err = SwissArmyHammerError::Context {
            message: "Failed to load workflow".to_string(),
            source: Box::new(io_err),
        };

        let chain = err.error_chain().to_string();
        assert!(chain.contains("Failed to load workflow"));
        assert!(chain.contains("file not found"));
    }
}
