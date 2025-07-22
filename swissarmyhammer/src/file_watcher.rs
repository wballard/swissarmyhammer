//! File watching functionality for prompt directories
//!
//! This module provides a unified file watching system that can monitor
//! prompt directories for changes and trigger appropriate reload actions.

use crate::common::{file_types::is_any_prompt_file, mcp_errors::ToSwissArmyHammerError};
use crate::{PromptResolver, Result};
use notify::{
    event::{Event, EventKind},
    RecommendedWatcher, RecursiveMode, Watcher,
};
use tokio::sync::mpsc;

/// File watcher for monitoring prompt directories
pub struct FileWatcher {
    /// Handle to the background watcher task
    watcher_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Configuration for file watching behavior
pub struct FileWatcherConfig {
    /// Channel buffer size for file system events
    pub channel_buffer_size: usize,
    /// Whether to watch directories recursively
    pub recursive: bool,
}

impl Default for FileWatcherConfig {
    fn default() -> Self {
        Self {
            channel_buffer_size: 100,
            recursive: true,
        }
    }
}

/// Callback trait for handling file system events
pub trait FileWatcherCallback: Send + Sync + 'static {
    /// Called when a relevant file change is detected
    fn on_file_changed(
        &self,
        paths: Vec<std::path::PathBuf>,
    ) -> impl std::future::Future<Output = Result<()>> + Send;

    /// Called when the file watcher encounters an error
    fn on_error(&self, error: String) -> impl std::future::Future<Output = ()> + Send;
}

impl FileWatcher {
    /// Create a new file watcher
    pub fn new() -> Self {
        Self {
            watcher_handle: None,
        }
    }

    /// Start watching prompt directories for changes
    pub async fn start_watching<C>(&mut self, callback: C) -> Result<()>
    where
        C: FileWatcherCallback + Clone,
    {
        self.start_watching_with_config(callback, FileWatcherConfig::default())
            .await
    }

    /// Start watching with custom configuration
    pub async fn start_watching_with_config<C>(
        &mut self,
        callback: C,
        config: FileWatcherConfig,
    ) -> Result<()>
    where
        C: FileWatcherCallback + Clone,
    {
        // Stop existing watcher if running
        self.stop_watching();

        tracing::info!("Starting file watching for prompt directories");

        // Get the directories to watch using the same logic as PromptResolver
        let resolver = PromptResolver::new();
        let watch_paths = resolver.get_prompt_directories()?;

        tracing::info!(
            "Found {} directories to watch: {:?}",
            watch_paths.len(),
            watch_paths
        );

        // The resolver already returns only existing paths
        if watch_paths.is_empty() {
            tracing::warn!("No prompt directories found to watch");
            return Ok(());
        }

        // Create the file watcher
        let (tx, mut rx) = mpsc::channel(config.channel_buffer_size);
        let mut watcher = RecommendedWatcher::new(
            move |result: std::result::Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    if let Err(e) = tx.blocking_send(event) {
                        tracing::error!("Failed to send file watch event: {}", e);
                    }
                }
            },
            notify::Config::default(),
        )
        .to_swiss_error_with_context("Failed to create file watcher")?;

        // Watch all directories
        let recursive_mode = if config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        for path in &watch_paths {
            watcher
                .watch(path, recursive_mode)
                .to_swiss_error_with_context(&format!("Failed to watch directory {path:?}"))?;
            tracing::info!("Watching directory: {:?}", path);
        }

        // Spawn the event handler task
        let handle = tokio::spawn(async move {
            // Keep the watcher alive for the duration of this task
            // The watcher must be moved into the task to prevent it from being dropped
            let _watcher = watcher;

            while let Some(event) = rx.recv().await {
                tracing::debug!("📁 File system event: {:?}", event);

                // Check if this is a relevant event
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        // Check if it's a prompt file
                        let relevant_paths: Vec<std::path::PathBuf> = event
                            .paths
                            .iter()
                            .filter(|p| is_any_prompt_file(p))
                            .cloned()
                            .collect();

                        if !relevant_paths.is_empty() {
                            tracing::info!("📄 Prompt file changed: {:?}", relevant_paths);

                            // Notify callback about the change
                            if let Err(e) = callback.on_file_changed(relevant_paths).await {
                                tracing::error!("❌ File watcher callback failed: {}", e);
                                callback.on_error(format!("Callback failed: {e}")).await;
                            }
                        } else {
                            tracing::debug!("🚫 Ignoring non-prompt file: {:?}", event.paths);
                        }
                    }
                    _ => {
                        tracing::debug!("🚫 Ignoring event type: {:?}", event.kind);
                    }
                }
            }
        });

        // Store the handle
        self.watcher_handle = Some(handle);

        Ok(())
    }

    /// Stop file watching
    pub fn stop_watching(&mut self) {
        if let Some(handle) = self.watcher_handle.take() {
            handle.abort();
        }
    }

    // File type detection moved to common::file_types module
}

impl Drop for FileWatcher {
    fn drop(&mut self) {
        self.stop_watching();
    }
}

impl Default for FileWatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    #[derive(Clone)]
    struct TestCallback {
        changes: Arc<Mutex<Vec<Vec<std::path::PathBuf>>>>,
        errors: Arc<Mutex<Vec<String>>>,
    }

    impl TestCallback {
        fn new() -> Self {
            Self {
                changes: Arc::new(Mutex::new(Vec::new())),
                errors: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl FileWatcherCallback for TestCallback {
        async fn on_file_changed(&self, paths: Vec<std::path::PathBuf>) -> Result<()> {
            self.changes.lock().await.push(paths);
            Ok(())
        }

        async fn on_error(&self, error: String) {
            self.errors.lock().await.push(error);
        }
    }

    #[tokio::test]
    async fn test_file_watcher_creation() {
        let watcher = FileWatcher::new();
        assert!(watcher.watcher_handle.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_file_watcher_start_stop() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
        fs::create_dir_all(&test_prompts_dir).unwrap();

        // Create a test prompt file so directory isn't empty
        let test_file = test_prompts_dir.join("test.md");
        fs::write(&test_file, "test prompt").unwrap();

        // Set current directory to temp dir so it finds our test directory
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let mut watcher = FileWatcher::new();
        let callback = TestCallback::new();

        // Start watching - should now succeed
        let result = watcher.start_watching(callback).await;
        assert!(result.is_ok());
        assert!(watcher.watcher_handle.is_some());

        // Stop watching
        watcher.stop_watching();
        assert!(watcher.watcher_handle.is_none());

        // Restore original directory
        let _ = std::env::set_current_dir(original_dir);
    }

    #[test]
    fn test_is_prompt_file() {
        use std::path::Path;

        assert!(is_any_prompt_file(Path::new("test.md")));
        assert!(is_any_prompt_file(Path::new("test.yaml")));
        assert!(is_any_prompt_file(Path::new("test.yml")));
        assert!(!is_any_prompt_file(Path::new("test.txt")));
        assert!(!is_any_prompt_file(Path::new("test")));
    }

    #[test]
    fn test_file_watcher_config_default() {
        let config = FileWatcherConfig::default();
        assert_eq!(config.channel_buffer_size, 100);
        assert!(config.recursive);
    }

    #[test]
    fn test_file_watcher_default_trait() {
        let watcher1 = FileWatcher::default();
        let watcher2 = FileWatcher::new();
        // Both should create watchers without handles
        assert!(watcher1.watcher_handle.is_none());
        assert!(watcher2.watcher_handle.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_file_watcher_custom_config() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
        fs::create_dir_all(&test_prompts_dir).unwrap();

        // Create a test prompt file
        let test_file = test_prompts_dir.join("test.yaml");
        fs::write(&test_file, "name: test\ndescription: test prompt").unwrap();

        // Set current directory to temp dir
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let mut watcher = FileWatcher::new();
        let callback = TestCallback::new();
        let config = FileWatcherConfig {
            channel_buffer_size: 200,
            recursive: false,
        };

        // Start with custom config - should now succeed
        let result = watcher.start_watching_with_config(callback, config).await;
        assert!(result.is_ok());
        assert!(watcher.watcher_handle.is_some());

        watcher.stop_watching();
        assert!(watcher.watcher_handle.is_none());

        // Restore original directory before temp_dir is dropped
        let _ = std::env::set_current_dir(&original_dir);
    }

    #[tokio::test]
    #[serial]
    async fn test_file_watcher_drop() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
        fs::create_dir_all(&test_prompts_dir).unwrap();
        fs::write(test_prompts_dir.join("test.md"), "test").unwrap();

        // Set current directory to temp dir
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let mut watcher = FileWatcher::new();
        let callback = TestCallback::new();

        // Start watching
        let result = watcher.start_watching(callback).await;
        assert!(result.is_ok());
        assert!(watcher.watcher_handle.is_some());

        // Drop the watcher - should stop watching
        drop(watcher);
        // Cannot test after drop, but Drop trait should have been called

        // Restore original directory
        let _ = std::env::set_current_dir(original_dir);
    }

    #[tokio::test]
    #[serial]
    async fn test_file_watcher_restart() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();

        // Save original directory first
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        let test_prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
        fs::create_dir_all(&test_prompts_dir).unwrap();
        fs::write(test_prompts_dir.join("test.yml"), "name: test").unwrap();

        // Set current directory to temp dir
        std::env::set_current_dir(temp_dir.path()).unwrap();

        struct DirGuard {
            original_dir: std::path::PathBuf,
        }

        impl Drop for DirGuard {
            fn drop(&mut self) {
                let _ = std::env::set_current_dir(&self.original_dir);
            }
        }

        let _guard = DirGuard {
            original_dir: original_dir.clone(),
        };

        let mut watcher = FileWatcher::new();
        let callback1 = TestCallback::new();
        let callback2 = TestCallback::new();

        // Start watching first time
        let result1 = watcher.start_watching(callback1).await;
        assert!(result1.is_ok());
        assert!(watcher.watcher_handle.is_some());

        // Start watching again - should stop previous and start new
        let result2 = watcher.start_watching(callback2).await;
        assert!(result2.is_ok());
        assert!(watcher.watcher_handle.is_some());

        watcher.stop_watching();
    }

    #[derive(Clone)]
    struct ErrorCallback {
        calls: Arc<Mutex<Vec<Vec<std::path::PathBuf>>>>,
    }

    impl ErrorCallback {
        fn new() -> Self {
            Self {
                calls: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }

    impl FileWatcherCallback for ErrorCallback {
        async fn on_file_changed(&self, paths: Vec<std::path::PathBuf>) -> Result<()> {
            self.calls.lock().await.push(paths.clone());
            // Return error to test error handling
            Err(crate::SwissArmyHammerError::Other("Test error".to_string()))
        }

        async fn on_error(&self, _error: String) {
            // Track that error handler was called
        }
    }

    #[test]
    fn test_is_prompt_file_edge_cases() {
        use std::path::Path;

        // Test file without extension
        assert!(!is_any_prompt_file(Path::new("README")));

        // Test hidden files
        assert!(is_any_prompt_file(Path::new(".test.md")));
        assert!(is_any_prompt_file(Path::new(".config.yaml")));
        assert!(!is_any_prompt_file(Path::new(".gitignore")));

        // Test files with multiple dots
        assert!(is_any_prompt_file(Path::new("file.test.md")));
        assert!(is_any_prompt_file(Path::new("config.prod.yaml")));

        // Test case insensitivity (our implementation is case-insensitive for user-friendliness)
        assert!(is_any_prompt_file(Path::new("file.MD")));
        assert!(is_any_prompt_file(Path::new("file.YML")));
        assert!(is_any_prompt_file(Path::new("file.YAML")));
    }

    #[tokio::test]
    async fn test_file_watcher_multiple_stops() {
        let mut watcher = FileWatcher::new();

        // Multiple stops should be safe
        watcher.stop_watching();
        watcher.stop_watching();
        assert!(watcher.watcher_handle.is_none());
    }

    #[tokio::test]
    #[serial]
    async fn test_file_watcher_error_callback() {
        use std::fs;
        use tempfile::TempDir;

        // Create a temporary directory for testing
        let temp_dir = TempDir::new().unwrap();
        let test_prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
        fs::create_dir_all(&test_prompts_dir).unwrap();
        fs::write(test_prompts_dir.join("test.yaml"), "name: test").unwrap();

        // Set current directory to temp dir
        let original_dir =
            std::env::current_dir().unwrap_or_else(|_| temp_dir.path().to_path_buf());
        std::env::set_current_dir(temp_dir.path()).unwrap();

        let mut watcher = FileWatcher::new();
        let callback = ErrorCallback::new();

        // Start watching with error callback
        let result = watcher.start_watching(callback.clone()).await;
        assert!(result.is_ok());
        assert!(watcher.watcher_handle.is_some());

        watcher.stop_watching();

        // Restore original directory before temp_dir is dropped
        let _ = std::env::set_current_dir(&original_dir);
    }
}
