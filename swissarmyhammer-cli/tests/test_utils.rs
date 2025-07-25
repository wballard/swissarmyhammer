//! Test utilities for SwissArmyHammer CLI tests
//!
//! This module extends the test utilities from the main crate with CLI-specific helpers.

use anyhow::Result;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

// Re-export the ProcessGuard from the main crate's test_utils
#[allow(unused_imports)]
pub use swissarmyhammer::test_utils::ProcessGuard;

// Re-export commonly used test utilities from the main crate
#[allow(unused_imports)]
pub use swissarmyhammer::test_utils::{
    create_simple_test_prompt, create_test_home_guard, create_test_prompt_library,
    create_test_prompts, get_test_home, get_test_swissarmyhammer_dir, TestHomeGuard,
};

/// Create a temporary directory for testing
///
/// This is a convenience wrapper that provides consistent error handling
#[allow(dead_code)]
pub fn create_temp_dir() -> Result<TempDir> {
    Ok(TempDir::new()?)
}

/// Create test prompt files in a directory
///
/// This creates actual prompt files on disk for integration testing.
/// Different from the main crate's create_test_prompts which creates Prompt objects.
#[allow(dead_code)]
pub fn create_test_prompt_files(prompts_dir: &Path) -> Result<()> {
    let test_prompts = vec![
        ("simple", "Hello, world!", vec![]),
        (
            "with_args",
            "Hello {{name}}, you are {{age}} years old",
            vec![("name", "User's name", true), ("age", "User's age", true)],
        ),
        (
            "code_review",
            "Review this code: {{ code }}",
            vec![("code", "Code to review", true)],
        ),
        (
            "bug_fix",
            "Fix this bug: {{ error }}",
            vec![("error", "Error message", true)],
        ),
        (
            "test_generation",
            "Generate tests for: {{ function }}",
            vec![("function", "Function to test", true)],
        ),
    ];

    for (name, template, args) in test_prompts {
        let prompt_file = prompts_dir.join(format!("{name}.prompt"));
        let mut yaml_content = String::from("---\n");
        yaml_content.push_str(&format!("name: {name}\n"));
        yaml_content.push_str(&format!("description: Test prompt for {name}\n"));

        if !args.is_empty() {
            yaml_content.push_str("arguments:\n");
            for (arg_name, desc, required) in args {
                yaml_content.push_str(&format!("  - name: {arg_name}\n"));
                yaml_content.push_str(&format!("    description: {desc}\n"));
                yaml_content.push_str(&format!("    required: {required}\n"));
            }
        }

        yaml_content.push_str("---\n");
        yaml_content.push_str(template);

        std::fs::write(&prompt_file, yaml_content)?;
    }

    Ok(())
}

/// Create a temporary test environment with prompts
///
/// Returns a TempDir and the path to the prompts directory
#[allow(dead_code)]
pub fn create_test_environment() -> Result<(TempDir, PathBuf)> {
    let temp_dir = create_temp_dir()?;
    let swissarmyhammer_dir = temp_dir.path().join(".swissarmyhammer");
    let prompts_dir = swissarmyhammer_dir.join("prompts");

    std::fs::create_dir_all(&prompts_dir)?;
    create_test_prompt_files(&prompts_dir)?;

    Ok((temp_dir, prompts_dir))
}

/// Setup environment for MCP tests
///
/// Sets HOME to a temporary directory and creates the necessary structure
#[allow(dead_code)]
pub fn setup_mcp_test_env() -> Result<(TempDir, PathBuf)> {
    let temp_dir = create_temp_dir()?;
    std::env::set_var("HOME", temp_dir.path());

    let swissarmyhammer_dir = temp_dir.path().join(".swissarmyhammer");
    let prompts_dir = swissarmyhammer_dir.join("prompts");

    std::fs::create_dir_all(&prompts_dir)?;
    create_test_prompt_files(&prompts_dir)?;

    Ok((temp_dir, prompts_dir))
}

/// Guard that manages test environment variables for semantic search tests
///
/// This sets up a controlled API key environment for testing semantic search
/// functionality without requiring real API credentials.
pub struct SemanticTestGuard {
    _home_guard: TestHomeGuard,
    original_api_key: Option<String>,
}

impl SemanticTestGuard {
    /// Create a new semantic test guard with isolated environment
    pub fn new() -> Self {
        let home_guard = create_test_home_guard();
        let original_api_key = std::env::var("NOMIC_API_KEY").ok();

        // Set a test API key that allows the command to start but will fail gracefully
        std::env::set_var("NOMIC_API_KEY", "test-key-for-cli-integration-testing");

        Self {
            _home_guard: home_guard,
            original_api_key,
        }
    }
}

impl Default for SemanticTestGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SemanticTestGuard {
    fn drop(&mut self) {
        // Restore original API key environment variable
        match &self.original_api_key {
            Some(key) => std::env::set_var("NOMIC_API_KEY", key),
            None => std::env::remove_var("NOMIC_API_KEY"),
        }
    }
}

/// Create a semantic test environment guard
///
/// This provides isolated environment setup for semantic search tests
/// with proper cleanup and restoration of environment variables.
#[allow(dead_code)]
pub fn create_semantic_test_guard() -> SemanticTestGuard {
    SemanticTestGuard::new()
}
