/// Test helper module for setting up test environment with consistent home directory
///
/// This module provides utilities to ensure tests use a preconfigured test home directory
/// instead of the actual ~/.swissarmyhammer directory, ensuring consistent test behavior
/// in both local development and GitHub Actions.

use std::path::PathBuf;
use std::sync::Once;

static INIT: Once = Once::new();
static mut TEST_HOME_PATH: Option<PathBuf> = None;

/// Initialize the test home directory path once for all tests
fn init_test_home_path() {
    INIT.call_once(|| {
        let test_home = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("tests")
            .join("test-home");
        unsafe {
            TEST_HOME_PATH = Some(test_home);
        }
    });
}

/// Set up the test environment by configuring HOME to use the test directory
///
/// This function should be called at the beginning of each test that needs
/// to access the ~/.swissarmyhammer directory.
///
/// # Example
///
/// ```
/// #[test]
/// fn test_something() {
///     setup_test_home();
///     // Your test code here
/// }
/// ```
pub fn setup_test_home() {
    init_test_home_path();
    
    let test_home = unsafe {
        TEST_HOME_PATH.as_ref().expect("Test home path not initialized")
    };
    
    // Set the HOME environment variable to our test directory
    std::env::set_var("HOME", test_home);
}

/// Get the path to the test home directory
///
/// This is useful when you need to access files within the test home directory.
pub fn get_test_home() -> PathBuf {
    init_test_home_path();
    
    unsafe {
        TEST_HOME_PATH.as_ref().expect("Test home path not initialized").clone()
    }
}

/// Get the path to the test .swissarmyhammer directory
pub fn get_test_swissarmyhammer_dir() -> PathBuf {
    get_test_home().join(".swissarmyhammer")
}

/// Set up test environment and return a guard that restores the original HOME on drop
///
/// This is useful for tests that need to ensure the HOME environment variable
/// is restored after the test completes.
pub struct TestHomeGuard {
    original_home: Option<String>,
}

impl TestHomeGuard {
    pub fn new() -> Self {
        let original_home = std::env::var("HOME").ok();
        setup_test_home();
        Self { original_home }
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
/// ```
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
}