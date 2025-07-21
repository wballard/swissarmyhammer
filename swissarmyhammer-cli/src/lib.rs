//! SwissArmyHammer CLI Library
//!
//! This library provides the core functionality for the SwissArmyHammer CLI,
//! including command-line interface definitions, validation, and exit codes.

// Re-export modules for use in tests
/// Command-line interface definitions and argument parsing
pub mod cli;
/// Exit codes used by the CLI application
pub mod exit_codes;
/// Validation functionality for prompts and workflows
pub mod validate;
