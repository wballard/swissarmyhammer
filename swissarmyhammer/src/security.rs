//! Security utilities for path validation and resource limits
//!
//! This module provides functions to ensure safe file operations and prevent
//! potential security vulnerabilities like path traversal attacks and denial
//! of service through excessive resource consumption.

use crate::{Result, SwissArmyHammerError};
use std::path::{Path, PathBuf};

/// Maximum allowed depth for directory traversal operations
pub const MAX_DIRECTORY_DEPTH: usize = 10;

/// Maximum allowed complexity for workflow graphs (states + transitions)
pub const MAX_WORKFLOW_COMPLEXITY: usize = 1000;

/// Checks if a path is safe to access within a given root directory
///
/// This function validates that:
/// - The path doesn't contain dangerous components like ".."
/// - The canonical path is within the root directory
/// - The path doesn't follow symlinks outside the root
///
/// # Arguments
///
/// * `path` - The path to validate
/// * `root` - The root directory that the path must be within
///
/// # Returns
///
/// The canonical path if safe, or an error if the path is unsafe
pub fn validate_path_security(path: &Path, root: &Path) -> Result<PathBuf> {
    // First check for obvious dangerous patterns
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                return Err(SwissArmyHammerError::Other(
                    "Path contains parent directory references (..)".to_string(),
                ));
            }
            std::path::Component::RootDir => {
                return Err(SwissArmyHammerError::Other(
                    "Path contains absolute root reference".to_string(),
                ));
            }
            _ => {}
        }
    }

    // Get canonical paths to resolve symlinks and relative paths
    let canonical_root = root.canonicalize().map_err(|e| {
        SwissArmyHammerError::Other(format!("Failed to canonicalize root path: {}", e))
    })?;

    let full_path = if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    };

    // For files that don't exist yet, we need to check the parent directory
    let canonical_path = if full_path.exists() {
        full_path.canonicalize().map_err(|e| {
            SwissArmyHammerError::Other(format!("Failed to canonicalize path: {}", e))
        })?
    } else {
        // Check the parent directory exists and is valid
        let parent = full_path.parent().ok_or_else(|| {
            SwissArmyHammerError::Other("Path has no parent directory".to_string())
        })?;

        let canonical_parent = parent.canonicalize().map_err(|e| {
            SwissArmyHammerError::Other(format!("Failed to canonicalize parent path: {}", e))
        })?;

        // Ensure the parent is within the root
        if !canonical_parent.starts_with(&canonical_root) {
            return Err(SwissArmyHammerError::Other(format!(
                "Path '{}' parent is outside allowed directory",
                path.display()
            )));
        }

        // Return the intended path (parent + filename)
        let filename = full_path.file_name().ok_or_else(|| {
            SwissArmyHammerError::Other("Path has no filename component".to_string())
        })?;

        canonical_parent.join(filename)
    };

    // Ensure the canonical path is within the root
    if !canonical_path.starts_with(&canonical_root) {
        return Err(SwissArmyHammerError::Other(format!(
            "Path '{}' is outside allowed directory",
            path.display()
        )));
    }

    Ok(canonical_path)
}

/// Calculates the depth of a path relative to a root directory
///
/// # Arguments
///
/// * `path` - The path to check
/// * `root` - The root directory to calculate depth from
///
/// # Returns
///
/// The depth as a usize, or 0 if the path is not within the root
pub fn calculate_path_depth(path: &Path, root: &Path) -> usize {
    let canonical_root = match root.canonicalize() {
        Ok(p) => p,
        Err(_) => return 0,
    };

    let canonical_path = match path.canonicalize() {
        Ok(p) => p,
        Err(_) => return 0,
    };

    if !canonical_path.starts_with(&canonical_root) {
        return 0;
    }

    canonical_path
        .strip_prefix(&canonical_root)
        .map(|p| p.components().count())
        .unwrap_or(0)
}

/// Checks if a workflow's complexity is within acceptable limits
///
/// # Arguments
///
/// * `states_count` - Number of states in the workflow
/// * `transitions_count` - Number of transitions in the workflow
///
/// # Returns
///
/// Ok if within limits, error if too complex
pub fn validate_workflow_complexity(states_count: usize, transitions_count: usize) -> Result<()> {
    let total_complexity = states_count + transitions_count;

    if total_complexity > MAX_WORKFLOW_COMPLEXITY {
        return Err(SwissArmyHammerError::Other(format!(
            "Workflow too complex: {} states + {} transitions = {} (max allowed: {})",
            states_count, transitions_count, total_complexity, MAX_WORKFLOW_COMPLEXITY
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validate_path_security_safe_path() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();
        let safe_file = root.join("test.txt");
        fs::write(&safe_file, "test").unwrap();

        // Test with relative path
        let result = validate_path_security(Path::new("test.txt"), root);
        if let Err(e) = &result {
            panic!("Expected Ok, got error: {}", e);
        }
        assert_eq!(result.unwrap(), safe_file.canonicalize().unwrap());
    }

    #[test]
    fn test_validate_path_security_parent_dir() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Test with relative path containing parent directory
        let result = validate_path_security(Path::new("../outside.txt"), root);
        match &result {
            Ok(_) => panic!("Expected error, got Ok"),
            Err(e) => {
                let error_str = e.to_string();
                assert!(
                    error_str.contains("parent directory"),
                    "Expected 'parent directory' in error, got: {}",
                    error_str
                );
            }
        }
    }

    #[test]
    fn test_validate_path_security_absolute_path() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create a file outside the temp directory
        let other_temp = TempDir::new().unwrap();
        let outside_file = other_temp.path().join("outside.txt");
        fs::write(&outside_file, "test").unwrap();

        let result = validate_path_security(&outside_file, root);
        match &result {
            Ok(_) => panic!("Expected error, got Ok"),
            Err(e) => {
                let error_str = e.to_string();
                // Absolute paths are rejected early in the validation
                assert!(error_str.contains("absolute root reference") || 
                        error_str.contains("outside allowed directory"), 
                    "Expected 'absolute root reference' or 'outside allowed directory' in error, got: {}", error_str);
            }
        }
    }

    #[test]
    fn test_calculate_path_depth() {
        let temp_dir = TempDir::new().unwrap();
        let root = temp_dir.path();

        // Create nested directories
        let level1 = root.join("level1");
        let level2 = level1.join("level2");
        let level3 = level2.join("level3");

        fs::create_dir_all(&level3).unwrap();

        assert_eq!(calculate_path_depth(root, root), 0);
        assert_eq!(calculate_path_depth(&level1, root), 1);
        assert_eq!(calculate_path_depth(&level2, root), 2);
        assert_eq!(calculate_path_depth(&level3, root), 3);
    }

    #[test]
    fn test_validate_workflow_complexity_within_limits() {
        assert!(validate_workflow_complexity(10, 20).is_ok());
        assert!(validate_workflow_complexity(100, 200).is_ok());
        assert!(validate_workflow_complexity(500, 499).is_ok());
    }

    #[test]
    fn test_validate_workflow_complexity_exceeds_limits() {
        let result = validate_workflow_complexity(600, 600);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too complex"));
    }
}
