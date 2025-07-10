//! Unified file system utilities for SwissArmyHammer
//!
//! This module provides a consistent abstraction over file I/O operations,
//! offering better error handling, testability, and security than direct
//! `std::fs` usage.

use crate::error::{Result, SwissArmyHammerError};
use std::path::{Path, PathBuf};
use std::sync::Arc;

/// Trait for file system operations
///
/// This abstraction allows for easy testing by providing mock implementations
/// while maintaining the same interface for production code.
pub trait FileSystem: Send + Sync {
    /// Read a file to string with enhanced error context
    fn read_to_string(&self, path: &Path) -> Result<String>;

    /// Write string content to a file atomically
    fn write(&self, path: &Path, content: &str) -> Result<()>;

    /// Check if a path exists
    fn exists(&self, path: &Path) -> bool;

    /// Check if a path is a file
    fn is_file(&self, path: &Path) -> bool;

    /// Check if a path is a directory  
    fn is_dir(&self, path: &Path) -> bool;

    /// Create directories recursively
    fn create_dir_all(&self, path: &Path) -> Result<()>;

    /// Read directory entries
    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>>;

    /// Remove a file
    fn remove_file(&self, path: &Path) -> Result<()>;
}

/// Production file system implementation using std::fs
#[derive(Default)]
pub struct StdFileSystem;

impl FileSystem for StdFileSystem {
    fn read_to_string(&self, path: &Path) -> Result<String> {
        std::fs::read_to_string(path).map_err(|e| {
            SwissArmyHammerError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read file '{}': {}", path.display(), e),
            ))
        })
    }

    fn write(&self, path: &Path, content: &str) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            self.create_dir_all(parent)?;
        }

        // Write atomically by writing to a temporary file first, then renaming
        let temp_path = path.with_extension(format!(
            "{}.tmp",
            path.extension().and_then(|s| s.to_str()).unwrap_or("")
        ));

        std::fs::write(&temp_path, content).map_err(|e| {
            SwissArmyHammerError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write temp file '{}': {}", temp_path.display(), e),
            ))
        })?;

        std::fs::rename(&temp_path, path).map_err(|e| {
            // Clean up temp file on rename failure
            let _ = std::fs::remove_file(&temp_path);
            SwissArmyHammerError::Io(std::io::Error::new(
                e.kind(),
                format!(
                    "Failed to rename temp file '{}' to '{}': {}",
                    temp_path.display(),
                    path.display(),
                    e
                ),
            ))
        })
    }

    fn exists(&self, path: &Path) -> bool {
        path.exists()
    }

    fn is_file(&self, path: &Path) -> bool {
        path.is_file()
    }

    fn is_dir(&self, path: &Path) -> bool {
        path.is_dir()
    }

    fn create_dir_all(&self, path: &Path) -> Result<()> {
        std::fs::create_dir_all(path).map_err(|e| {
            SwissArmyHammerError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to create directory '{}': {}", path.display(), e),
            ))
        })
    }

    fn read_dir(&self, path: &Path) -> Result<Vec<PathBuf>> {
        let entries = std::fs::read_dir(path).map_err(|e| {
            SwissArmyHammerError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read directory '{}': {}", path.display(), e),
            ))
        })?;

        let mut paths = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| {
                SwissArmyHammerError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed to read directory entry in '{}': {}",
                        path.display(),
                        e
                    ),
                ))
            })?;
            paths.push(entry.path());
        }

        Ok(paths)
    }

    fn remove_file(&self, path: &Path) -> Result<()> {
        std::fs::remove_file(path).map_err(|e| {
            SwissArmyHammerError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to remove file '{}': {}", path.display(), e),
            ))
        })
    }
}

/// File system utility with dependency injection support
pub struct FileSystemUtils {
    fs: Arc<dyn FileSystem>,
}

impl FileSystemUtils {
    /// Create new file system utils with the default std implementation
    pub fn new() -> Self {
        Self {
            fs: Arc::new(StdFileSystem),
        }
    }

    /// Create new file system utils with a custom implementation (for testing)
    pub fn with_fs(fs: Arc<dyn FileSystem>) -> Self {
        Self { fs }
    }

    /// Read and parse a YAML file
    pub fn read_yaml<T>(&self, path: &Path) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let content = self.fs.read_to_string(path)?;
        serde_yaml::from_str(&content).map_err(SwissArmyHammerError::Serialization)
    }

    /// Write data as YAML to a file
    pub fn write_yaml<T>(&self, path: &Path, data: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let content = serde_yaml::to_string(data)?;
        self.fs.write(path, &content)
    }

    /// Read and parse a JSON file
    pub fn read_json<T>(&self, path: &Path) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let content = self.fs.read_to_string(path)?;
        serde_json::from_str(&content).map_err(SwissArmyHammerError::Json)
    }

    /// Write data as JSON to a file
    pub fn write_json<T>(&self, path: &Path, data: &T) -> Result<()>
    where
        T: serde::Serialize,
    {
        let content = serde_json::to_string_pretty(data)?;
        self.fs.write(path, &content)
    }

    /// Read a text file
    pub fn read_text(&self, path: &Path) -> Result<String> {
        self.fs.read_to_string(path)
    }

    /// Write text to a file
    pub fn write_text(&self, path: &Path, content: &str) -> Result<()> {
        self.fs.write(path, content)
    }

    /// Get a reference to the underlying file system
    pub fn fs(&self) -> &dyn FileSystem {
        &*self.fs
    }
}

impl Default for FileSystemUtils {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;

    /// Mock file system for testing
    pub struct MockFileSystem {
        files: Mutex<HashMap<PathBuf, String>>,
        dirs: Mutex<std::collections::HashSet<PathBuf>>,
    }

    impl MockFileSystem {
        pub fn new() -> Self {
            Self {
                files: Mutex::new(HashMap::new()),
                dirs: Mutex::new(std::collections::HashSet::new()),
            }
        }
    }

    impl FileSystem for MockFileSystem {
        fn read_to_string(&self, path: &Path) -> Result<String> {
            self.files
                .lock()
                .unwrap()
                .get(path)
                .cloned()
                .ok_or_else(|| {
                    SwissArmyHammerError::Io(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("File not found: {}", path.display()),
                    ))
                })
        }

        fn write(&self, path: &Path, content: &str) -> Result<()> {
            self.files
                .lock()
                .unwrap()
                .insert(path.to_path_buf(), content.to_string());
            Ok(())
        }

        fn exists(&self, path: &Path) -> bool {
            self.files.lock().unwrap().contains_key(path)
                || self.dirs.lock().unwrap().contains(path)
        }

        fn is_file(&self, path: &Path) -> bool {
            self.files.lock().unwrap().contains_key(path)
        }

        fn is_dir(&self, path: &Path) -> bool {
            self.dirs.lock().unwrap().contains(path)
        }

        fn create_dir_all(&self, path: &Path) -> Result<()> {
            self.dirs.lock().unwrap().insert(path.to_path_buf());
            Ok(())
        }

        fn read_dir(&self, _path: &Path) -> Result<Vec<PathBuf>> {
            // Simplified implementation for tests
            Ok(vec![])
        }

        fn remove_file(&self, path: &Path) -> Result<()> {
            self.files.lock().unwrap().remove(path);
            Ok(())
        }
    }

    #[test]
    fn test_mock_filesystem_read_write() {
        let mock_fs = Arc::new(MockFileSystem::new());
        let utils = FileSystemUtils::with_fs(mock_fs.clone());

        let path = Path::new("test.txt");
        let content = "Hello, world!";

        utils.write_text(path, content).unwrap();
        let read_content = utils.read_text(path).unwrap();

        assert_eq!(content, read_content);
    }

    #[test]
    fn test_yaml_serialization() {
        let mock_fs = Arc::new(MockFileSystem::new());
        let utils = FileSystemUtils::with_fs(mock_fs);

        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let path = Path::new("test.yaml");
        utils.write_yaml(path, &data).unwrap();
        let read_data: TestData = utils.read_yaml(path).unwrap();

        assert_eq!(data, read_data);
    }

    #[test]
    fn test_json_serialization() {
        let mock_fs = Arc::new(MockFileSystem::new());
        let utils = FileSystemUtils::with_fs(mock_fs);

        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct TestData {
            name: String,
            value: i32,
        }

        let data = TestData {
            name: "test".to_string(),
            value: 42,
        };

        let path = Path::new("test.json");
        utils.write_json(path, &data).unwrap();
        let read_data: TestData = utils.read_json(path).unwrap();

        assert_eq!(data, read_data);
    }
}
