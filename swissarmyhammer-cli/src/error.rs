//! Error handling for the SwissArmyHammer CLI
//!
//! This module provides a robust error handling approach that preserves
//! error context while still providing appropriate exit codes for CLI applications.

use std::error::Error;
use std::fmt;

use crate::exit_codes::{EXIT_ERROR, EXIT_SUCCESS};

/// CLI-specific result type that preserves error information
pub type CliResult<T> = Result<T, CliError>;

/// CLI error type that includes both error information and suggested exit code
#[derive(Debug)]
pub struct CliError {
    pub message: String,
    pub exit_code: i32,
    pub source: Option<Box<dyn Error + Send + Sync>>,
}

impl CliError {
    /// Create a new CLI error with a message and exit code
    pub fn new(message: impl Into<String>, exit_code: i32) -> Self {
        Self {
            message: message.into(),
            exit_code,
            source: None,
        }
    }

    /// Create a CLI error from a SwissArmyHammer error, with proper exit code handling for abort errors
    #[allow(dead_code)]
    pub fn from_swissarmyhammer_error(error: swissarmyhammer::SwissArmyHammerError) -> Self {
        // Check if this is an abort error by examining the error message
        let error_msg = error.to_string();
        if error_msg.contains("ABORT ERROR") {
            tracing::error!("Detected abort error, triggering immediate shutdown");
            Self {
                message: format!("Execution aborted: {error_msg}"),
                exit_code: EXIT_ERROR,
                source: Some(Box::new(error)),
            }
        } else {
            // Regular error handling
            Self {
                message: error_msg,
                exit_code: EXIT_ERROR,
                source: Some(Box::new(error)),
            }
        }
    }

    /// Get the full error chain as a formatted string
    pub fn full_chain(&self) -> String {
        let mut result = self.message.clone();

        let mut current_source = self.source();
        while let Some(err) = current_source {
            result.push_str(&format!("\n  Caused by: {err}"));
            current_source = err.source();
        }

        result
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CliError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.source
            .as_ref()
            .map(|e| e.as_ref() as &(dyn Error + 'static))
    }
}

/// Convert a CliResult to an exit code, printing the full error chain if needed
pub fn handle_cli_result<T>(result: CliResult<T>) -> i32 {
    match result {
        Ok(_) => EXIT_SUCCESS,
        Err(e) => {
            tracing::error!("Error: {}", e.full_chain());
            e.exit_code
        }
    }
}
