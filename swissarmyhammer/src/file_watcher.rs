//! File watching functionality for prompt directories
//!
//! This module provides a unified file watching system that can monitor
//! prompt directories for changes and trigger appropriate reload actions.

use crate::{PromptResolver, Result};
use notify::{
    event::{Event, EventKind},
    RecommendedWatcher, RecursiveMode, Watcher,
};
use std::path::Path;
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
        .map_err(|e| crate::SwissArmyHammerError::Other(format!("Failed to create file watcher: {}", e)))?;

        // Watch all directories
        let recursive_mode = if config.recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };

        for path in &watch_paths {
            watcher.watch(path, recursive_mode)
                .map_err(|e| crate::SwissArmyHammerError::Other(format!("Failed to watch directory {:?}: {}", path, e)))?;
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
                        // Check if it's a prompt file (*.md, *.yaml, *.yml)
                        let relevant_paths: Vec<std::path::PathBuf> = event
                            .paths
                            .iter()
                            .filter(|p| Self::is_prompt_file(p))
                            .cloned()
                            .collect();

                        if !relevant_paths.is_empty() {
                            tracing::info!("📄 Prompt file changed: {:?}", relevant_paths);

                            // Notify callback about the change
                            if let Err(e) = callback.on_file_changed(relevant_paths).await {
                                tracing::error!("❌ File watcher callback failed: {}", e);
                                callback.on_error(format!("Callback failed: {}", e)).await;
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

    /// Check if a file is a prompt file based on its extension
    fn is_prompt_file(path: &Path) -> bool {
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("md") | Some("yaml") | Some("yml"))
        } else {
            false
        }
    }
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
    async fn test_file_watcher_start_stop() {
        let mut watcher = FileWatcher::new();
        let callback = TestCallback::new();

        // Start watching
        let result = watcher.start_watching(callback).await;
        // This may fail if no prompt directories exist, which is fine for testing
        if result.is_ok() {
            assert!(watcher.watcher_handle.is_some());
        }

        // Stop watching
        watcher.stop_watching();
        assert!(watcher.watcher_handle.is_none());
    }

    #[test]
    fn test_is_prompt_file() {
        assert!(FileWatcher::is_prompt_file(Path::new("test.md")));
        assert!(FileWatcher::is_prompt_file(Path::new("test.yaml")));
        assert!(FileWatcher::is_prompt_file(Path::new("test.yml")));
        assert!(!FileWatcher::is_prompt_file(Path::new("test.txt")));
        assert!(!FileWatcher::is_prompt_file(Path::new("test")));
    }

    #[test]
    fn test_file_watcher_config_default() {
        let config = FileWatcherConfig::default();
        assert_eq!(config.channel_buffer_size, 100);
        assert!(config.recursive);
    }
}
