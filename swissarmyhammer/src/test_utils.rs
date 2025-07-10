/// Test utilities for SwissArmyHammer tests
///
/// This module provides shared testing infrastructure to ensure tests use a
/// preconfigured test home directory instead of the actual ~/.swissarmyhammer directory,
/// ensuring consistent test behavior in both local development and CI environments.
///
/// # Architecture
///
/// The test home system works by temporarily overriding the HOME environment variable
/// to point to a preconfigured test directory at `<workspace_root>/tests/test-home`.
/// This directory contains test fixtures including prompts and workflows that tests
/// can reliably depend on.
///
/// # Why HOME Override?
///
/// SwissArmyHammer reads user configuration from `~/.swissarmyhammer`. In tests, we need:
/// - Consistent, predictable content regardless of the developer's actual home directory
/// - Isolation from the developer's real SwissArmyHammer configuration
/// - The ability to test with specific prompt and workflow fixtures
///
/// # Usage Patterns
///
/// ## For Unit Tests
/// ```no_run
/// use swissarmyhammer::test_utils::create_test_home_guard;
///
/// #[test]
/// fn test_something() {
///     let _guard = create_test_home_guard();
///     // Test code here - HOME is now set to test directory
/// }
/// ```
///
/// ## For Integration Tests
/// Integration tests that create their own temporary directories (like MCP tests)
/// don't need to use this module. They can set HOME directly to their temp directory.
///
/// # Thread Safety
///
/// The module uses a global mutex to ensure thread-safe modification of the HOME
/// environment variable. This means tests using TestHomeGuard will serialize access
/// to HOME, which may impact parallel test execution performance.
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Global test home path, initialized once
static TEST_HOME_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Mutex to ensure thread-safe HOME environment variable modification
static HOME_MUTEX: Mutex<()> = Mutex::new(());

/// Initialize and get the test home directory path
///
/// This calculates the correct path to the test-home directory regardless of
/// whether the test is run from the workspace root or from within a crate.
fn get_test_home_path() -> &'static PathBuf {
    TEST_HOME_PATH.get_or_init(|| {
        // Try to find the workspace root by looking for the test-home directory
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        // Check if we're already at the workspace root (has tests/test-home)
        let direct_path = manifest_dir.join("tests").join("test-home");
        if direct_path.exists() {
            validate_path_safety(&direct_path, &manifest_dir);
            return direct_path;
        }

        // Otherwise, we're in a subcrate, go up one level
        let workspace_root = manifest_dir
            .parent()
            .expect("Failed to find parent directory")
            .to_path_buf();
        let path = workspace_root.join("tests").join("test-home");
        validate_path_safety(&path, &workspace_root);
        path
    })
}

/// Validate that a path doesn't escape the project directory
fn validate_path_safety(path: &PathBuf, project_root: &PathBuf) {
    // Canonicalize paths to resolve any .. or symlinks
    match (path.canonicalize(), project_root.canonicalize()) {
        (Ok(canonical_path), Ok(canonical_root)) => {
            if !canonical_path.starts_with(&canonical_root) {
                panic!(
                    "Security error: Test home path '{}' escapes project directory '{}'",
                    canonical_path.display(),
                    canonical_root.display()
                );
            }
        }
        (Err(_), _) => {
            // Path doesn't exist yet, which is ok for test setup
            // Just validate it doesn't contain suspicious patterns
            let path_str = path.to_string_lossy();
            if path_str.contains("../..") || path_str.contains("....") {
                panic!(
                    "Security error: Test home path '{}' contains suspicious patterns",
                    path_str
                );
            }
        }
        (_, Err(e)) => {
            panic!("Failed to canonicalize project root: {}", e);
        }
    }
}

/// Set up the test environment by configuring HOME to use the test directory
///
/// This function should be called at the beginning of each test that needs
/// to access the ~/.swissarmyhammer directory.
///
/// # Example
///
/// ```no_run
/// # use swissarmyhammer::test_utils::setup_test_home;
/// #[test]
/// fn test_something() {
///     setup_test_home();
///     // Your test code here
/// }
/// ```
pub fn setup_test_home() {
    let _guard = HOME_MUTEX.lock().expect("Failed to lock HOME mutex");
    let test_home = get_test_home_path();

    // Ensure the test home directory exists
    if !test_home.exists() {
        panic!(
            "Test home directory does not exist: {}. Please ensure tests/test-home is properly set up.",
            test_home.display()
        );
    }

    // Set the HOME environment variable to our test directory
    std::env::set_var("HOME", test_home);
}

/// Get the path to the test home directory
///
/// This is useful when you need to access files within the test home directory.
pub fn get_test_home() -> PathBuf {
    get_test_home_path().clone()
}

/// Get the path to the test .swissarmyhammer directory
pub fn get_test_swissarmyhammer_dir() -> PathBuf {
    get_test_home().join(".swissarmyhammer")
}

/// Guard that sets up test environment and restores the original HOME on drop
///
/// This is useful for tests that need to ensure the HOME environment variable
/// is restored after the test completes, preventing test pollution.
pub struct TestHomeGuard {
    original_home: Option<String>,
    _guard: std::sync::MutexGuard<'static, ()>,
}

impl TestHomeGuard {
    /// Create a new test home guard
    pub fn new() -> Self {
        let guard = HOME_MUTEX.lock().expect("Failed to lock HOME mutex");
        let original_home = std::env::var("HOME").ok();

        let test_home = get_test_home_path();

        // Ensure the test home directory exists
        if !test_home.exists() {
            panic!(
                "Test home directory does not exist: {}. Please ensure tests/test-home is properly set up.",
                test_home.display()
            );
        }

        std::env::set_var("HOME", test_home);

        Self {
            original_home,
            _guard: guard,
        }
    }
}

impl Default for TestHomeGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TestHomeGuard {
    fn drop(&mut self) {
        // Restore original HOME environment variable
        match &self.original_home {
            Some(home) => std::env::set_var("HOME", home),
            None => std::env::remove_var("HOME"),
        }
    }
}

/// Create a test home guard for scoped test environment setup
///
/// # Example
///
/// ```no_run
/// # use swissarmyhammer::test_utils::create_test_home_guard;
/// #[test]
/// fn test_with_guard() {
///     let _guard = create_test_home_guard();
///     // Your test code here
///     // HOME will be automatically restored when _guard goes out of scope
/// }
/// ```
pub fn create_test_home_guard() -> TestHomeGuard {
    TestHomeGuard::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_test_home() {
        let _guard = create_test_home_guard();

        let home = std::env::var("HOME").expect("HOME not set");
        assert!(home.contains("test-home"));

        let swissarmyhammer_dir = get_test_swissarmyhammer_dir();
        assert!(swissarmyhammer_dir.exists());
        assert!(swissarmyhammer_dir.join("prompts").exists());
        assert!(swissarmyhammer_dir.join("workflows").exists());
    }

    #[test]
    fn test_guard_restores_home() {
        let original_home = std::env::var("HOME").ok();

        {
            let _guard = create_test_home_guard();
            let test_home = std::env::var("HOME").expect("HOME not set");
            assert!(test_home.contains("test-home"));
        }

        // Check that HOME is restored after guard is dropped
        let restored_home = std::env::var("HOME").ok();
        assert_eq!(original_home, restored_home);
    }

    #[test]
    fn test_concurrent_access() {
        use std::thread;

        let handles: Vec<_> = (0..5)
            .map(|_| {
                thread::spawn(|| {
                    let _guard = create_test_home_guard();
                    let home = std::env::var("HOME").expect("HOME not set");
                    assert!(home.contains("test-home"));
                })
            })
            .collect();

        for handle in handles {
            handle.join().expect("Thread panicked");
        }
    }
}
