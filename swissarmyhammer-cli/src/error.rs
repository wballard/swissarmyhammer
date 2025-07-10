//! Error handling for the SwissArmyHammer CLI
//!
//! This module provides a robust error handling approach that preserves
//! error context while still providing appropriate exit codes for CLI applications.

use std::error::Error;
use std::fmt;

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

    /// Create a CLI error from another error with a specific exit code
    pub fn from_error<E: Error + Send + Sync + 'static>(error: E, exit_code: i32) -> Self {
        let message = error.to_string();
        Self {
            message,
            exit_code,
            source: Some(Box::new(error)),
        }
    }

    /// Create a CLI error with exit code 1 (general error)
    pub fn general<E: Error + Send + Sync + 'static>(error: E) -> Self {
        Self::from_error(error, 1)
    }

    /// Create a CLI error with exit code 2 (validation error)
    pub fn validation<E: Error + Send + Sync + 'static>(error: E) -> Self {
        Self::from_error(error, 2)
    }

    /// Get the full error chain as a formatted string
    pub fn full_chain(&self) -> String {
        let mut result = self.message.clone();

        let mut current_source = self.source();
        while let Some(err) = current_source {
            result.push_str(&format!("\n  Caused by: {}", err));
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

/// Extension trait for converting results to CLI results
pub trait IntoCliResult<T> {
    fn cli_error(self, exit_code: i32) -> CliResult<T>;
    fn cli_general_error(self) -> CliResult<T>;
    fn cli_validation_error(self) -> CliResult<T>;
}

impl<T, E: Error + Send + Sync + 'static> IntoCliResult<T> for Result<T, E> {
    fn cli_error(self, exit_code: i32) -> CliResult<T> {
        self.map_err(|e| CliError::from_error(e, exit_code))
    }

    fn cli_general_error(self) -> CliResult<T> {
        self.map_err(|e| CliError::general(e))
    }

    fn cli_validation_error(self) -> CliResult<T> {
        self.map_err(|e| CliError::validation(e))
    }
}


/// Convert a CliResult to an exit code, printing the full error chain if needed
pub fn handle_cli_result<T>(result: CliResult<T>) -> i32 {
    match result {
        Ok(_) => 0,
        Err(e) => {
            eprintln!("Error: {}", e.full_chain());
            e.exit_code
        }
    }
}
