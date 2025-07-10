//! Virtual file system for loading files from .swissarmyhammer directories
//!
//! This module provides a unified way to load files from the hierarchical
//! .swissarmyhammer directory structure, handling precedence and overrides.

use crate::Result;
use crate::security::MAX_DIRECTORY_DEPTH;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Source of a file (builtin, user, local, or dynamic)
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum FileSource {
    /// Builtin files embedded in the binary
    Builtin,
    /// User files from ~/.swissarmyhammer
    User,
    /// Local files from .swissarmyhammer directories
    Local,
    /// Dynamically generated files
    Dynamic,
}

impl std::fmt::Display for FileSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileSource::Builtin => write!(f, "builtin"),
            FileSource::User => write!(f, "user"),
            FileSource::Local => write!(f, "local"),
            FileSource::Dynamic => write!(f, "dynamic"),
        }
    }
}

/// Represents a file with its metadata
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// The logical name of the file (without extension)
    pub name: String,
    /// The full path to the file
    pub path: PathBuf,
    /// The file content
    pub content: String,
    /// Where this file came from
    pub source: FileSource,
}

impl FileEntry {
    /// Create a new FileEntry with explicit name
    pub fn new(name: impl Into<String>, path: PathBuf, content: String, source: FileSource) -> Self {
        Self {
            name: name.into(),
            path,
            content,
            source,
        }
    }

    /// Create a FileEntry from path and content, deriving name from the path
    pub fn from_path_and_content(path: PathBuf, content: String, source: FileSource) -> Self {
        // Extract the name from the path
        // For a path like /path/to/prompts/category/subcategory/test.md
        // We want to extract "category/subcategory/test"
        
        // Get the file stem
        let stem = path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default();
        
        // Check if we have prompts or workflows in the path
        let path_str = path.to_string_lossy();
        let name = if path_str.contains("/prompts/") || path_str.contains("/workflows/") {
            // Find the relative path after prompts/workflows
            let split_on = if path_str.contains("/prompts/") {
                "/prompts/"
            } else {
                "/workflows/"
            };
            
            if let Some(after_split) = path_str.split(split_on).nth(1) {
                // Remove the file extension
                after_split.trim_end_matches(".md")
                    .trim_end_matches(".mermaid")
                    .to_string()
            } else {
                stem.to_string()
            }
        } else {
            // No prompts/workflows directory in path, just use the stem
            stem.to_string()
        };

        Self {
            name,
            path,
            content,
            source,
        }
    }
}

/// Virtual file system that manages files from multiple sources
pub struct VirtualFileSystem {
    /// The subdirectory to look for (e.g., "prompts" or "workflows")
    pub subdirectory: String,
    /// Map of file names to file entries
    pub files: HashMap<String, FileEntry>,
    /// Track sources for each file
    pub file_sources: HashMap<String, FileSource>,
}

impl VirtualFileSystem {
    /// Create a new virtual file system for a specific subdirectory
    pub fn new(subdirectory: impl Into<String>) -> Self {
        Self {
            subdirectory: subdirectory.into(),
            files: HashMap::new(),
            file_sources: HashMap::new(),
        }
    }

    /// Add a builtin file
    pub fn add_builtin(&mut self, name: impl Into<String>, content: impl Into<String>) {
        let name = name.into();
        let entry = FileEntry::new(
            name.clone(),
            PathBuf::from(format!("builtin:/{}/{}", self.subdirectory, name)),
            content.into(),
            FileSource::Builtin,
        );
        self.add_file(entry);
    }

    /// Add a file entry
    pub fn add_file(&mut self, entry: FileEntry) {
        self.file_sources.insert(entry.name.clone(), entry.source.clone());
        self.files.insert(entry.name.clone(), entry);
    }

    /// Get a file by name
    pub fn get(&self, name: &str) -> Option<&FileEntry> {
        self.files.get(name)
    }

    /// Get the source of a file
    pub fn get_source(&self, name: &str) -> Option<&FileSource> {
        self.file_sources.get(name)
    }

    /// List all files
    pub fn list(&self) -> Vec<&FileEntry> {
        self.files.values().collect()
    }

    /// Load files from a directory
    pub fn load_directory(&mut self, base_path: &Path, source: FileSource) -> Result<()> {
        let target_dir = base_path.join(&self.subdirectory);
        if !target_dir.exists() {
            return Ok(());
        }

        for entry in walkdir::WalkDir::new(&target_dir)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                // Check for supported extensions
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if ext == "md" || ext == "mermaid" {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let file_entry = FileEntry::from_path_and_content(
                                path.to_path_buf(),
                                content,
                                source.clone(),
                            );
                            self.add_file(file_entry);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Load all files following the standard precedence
    pub fn load_all(&mut self) -> Result<()> {
        // Load builtin files (least precedence)
        // Note: Builtin files are typically added via add_builtin method

        // Load user files from home directory
        if let Some(home) = dirs::home_dir() {
            let user_dir = home.join(".swissarmyhammer");
            self.load_directory(&user_dir, FileSource::User)?;
        }

        // Load local files by walking up the directory tree
        self.load_local_files()?;

        Ok(())
    }

    /// Load local files by walking up the directory tree
    fn load_local_files(&mut self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let mut path = current_dir.as_path();
        let mut depth = 0;

        // Collect directories from current to root
        let mut directories = Vec::new();

        loop {
            if depth >= MAX_DIRECTORY_DEPTH {
                break;
            }

            let swissarmyhammer_dir = path.join(".swissarmyhammer");
            if swissarmyhammer_dir.exists() && swissarmyhammer_dir.is_dir() {
                // Skip the user's home .swissarmyhammer directory
                if let Some(home) = dirs::home_dir() {
                    let user_swissarmyhammer_dir = home.join(".swissarmyhammer");
                    if swissarmyhammer_dir == user_swissarmyhammer_dir {
                        match path.parent() {
                            Some(parent) => {
                                path = parent;
                                depth += 1;
                            }
                            None => break,
                        }
                        continue;
                    }
                }

                directories.push(swissarmyhammer_dir);
            }

            match path.parent() {
                Some(parent) => {
                    path = parent;
                    depth += 1;
                }
                None => break,
            }
        }

        // Load in reverse order (root to current) so deeper paths override
        for dir in directories.into_iter().rev() {
            self.load_directory(&dir, FileSource::Local)?;
        }

        Ok(())
    }

    /// Get all directories that are being monitored
    pub fn get_directories(&self) -> Result<Vec<PathBuf>> {
        let mut directories = Vec::new();

        // User directory
        if let Some(home) = dirs::home_dir() {
            let user_dir = home.join(".swissarmyhammer").join(&self.subdirectory);
            if user_dir.exists() {
                directories.push(user_dir);
            }
        }

        // Local directories
        let current_dir = std::env::current_dir()?;
        let mut path = current_dir.as_path();
        let mut local_dirs = Vec::new();
        let mut depth = 0;

        loop {
            if depth >= MAX_DIRECTORY_DEPTH {
                break;
            }

            let swissarmyhammer_dir = path.join(".swissarmyhammer");
            if swissarmyhammer_dir.exists() && swissarmyhammer_dir.is_dir() {
                // Skip user's home directory
                if let Some(home) = dirs::home_dir() {
                    let user_swissarmyhammer_dir = home.join(".swissarmyhammer");
                    if swissarmyhammer_dir == user_swissarmyhammer_dir {
                        match path.parent() {
                            Some(parent) => {
                                path = parent;
                                depth += 1;
                            }
                            None => break,
                        }
                        continue;
                    }
                }

                let subdir = swissarmyhammer_dir.join(&self.subdirectory);
                if subdir.exists() && subdir.is_dir() {
                    local_dirs.push(subdir);
                }
            }

            match path.parent() {
                Some(parent) => {
                    path = parent;
                    depth += 1;
                }
                None => break,
            }
        }

        // Add local directories in reverse order
        for dir in local_dirs.into_iter().rev() {
            directories.push(dir);
        }

        Ok(directories)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_source_display() {
        assert_eq!(FileSource::Builtin.to_string(), "builtin");
        assert_eq!(FileSource::User.to_string(), "user");
        assert_eq!(FileSource::Local.to_string(), "local");
        assert_eq!(FileSource::Dynamic.to_string(), "dynamic");
    }

    #[test]
    fn test_file_source_equality() {
        assert_eq!(FileSource::Builtin, FileSource::Builtin);
        assert_ne!(FileSource::Builtin, FileSource::User);
    }

    #[test]
    fn test_file_entry_creation() {
        let entry = FileEntry::new(
            "test_file",
            PathBuf::from("/path/to/file"),
            "content".to_string(),
            FileSource::Local,
        );
        assert_eq!(entry.name, "test_file");
        assert_eq!(entry.path, PathBuf::from("/path/to/file"));
        assert_eq!(entry.content, "content");
        assert_eq!(entry.source, FileSource::Local);
    }

    #[test]
    fn test_file_entry_name_from_path() {
        let entry = FileEntry::from_path_and_content(
            PathBuf::from("/path/to/test.md"),
            "content".to_string(),
            FileSource::User,
        );
        assert_eq!(entry.name, "test");
        assert_eq!(entry.content, "content");
        assert_eq!(entry.source, FileSource::User);
    }

    #[test]
    fn test_file_entry_nested_name() {
        let entry = FileEntry::from_path_and_content(
            PathBuf::from("/path/to/prompts/category/subcategory/test.md"),
            "content".to_string(),
            FileSource::Builtin,
        );
        assert_eq!(entry.name, "category/subcategory/test");
    }

    #[test]
    fn test_virtual_file_system_new() {
        let vfs = VirtualFileSystem::new("prompts");
        assert_eq!(vfs.subdirectory, "prompts");
        assert!(vfs.files.is_empty());
    }

    #[test]
    fn test_virtual_file_system_add_builtin() {
        let mut vfs = VirtualFileSystem::new("prompts");
        vfs.add_builtin("test", "content");
        
        let file = vfs.get("test").unwrap();
        assert_eq!(file.name, "test");
        assert_eq!(file.content, "content");
        assert_eq!(file.source, FileSource::Builtin);
    }

    #[test]
    fn test_virtual_file_system_load_directory() {
        use tempfile::TempDir;
        use std::fs;
        
        let temp_dir = TempDir::new().unwrap();
        let prompts_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&prompts_dir).unwrap();
        
        // Create a test file
        let test_file = prompts_dir.join("test.md");
        fs::write(&test_file, "test content").unwrap();
        
        let mut vfs = VirtualFileSystem::new("prompts");
        vfs.load_directory(temp_dir.path(), FileSource::Local).unwrap();
        
        let file = vfs.get("test").unwrap();
        assert_eq!(file.name, "test");
        assert_eq!(file.content, "test content");
        assert_eq!(file.source, FileSource::Local);
    }

    #[test]
    fn test_virtual_file_system_precedence() {
        let mut vfs = VirtualFileSystem::new("prompts");
        
        // Add builtin first
        vfs.add_builtin("test", "builtin content");
        
        // Add user version (should override)
        let entry = FileEntry::new(
            "test",
            PathBuf::from("/home/user/.swissarmyhammer/prompts/test.md"),
            "user content".to_string(),
            FileSource::User,
        );
        vfs.add_file(entry);
        
        // The user version should have overridden the builtin
        let file = vfs.get("test").unwrap();
        assert_eq!(file.content, "user content");
        assert_eq!(file.source, FileSource::User);
    }

    #[test]
    fn test_virtual_file_system_list() {
        let mut vfs = VirtualFileSystem::new("prompts");
        
        vfs.add_builtin("test1", "content1");
        vfs.add_builtin("test2", "content2");
        
        let files = vfs.list();
        assert_eq!(files.len(), 2);
        
        let names: Vec<&str> = files.iter().map(|f| f.name.as_str()).collect();
        assert!(names.contains(&"test1"));
        assert!(names.contains(&"test2"));
    }

    #[test]
    fn test_virtual_file_system_get_source() {
        let mut vfs = VirtualFileSystem::new("prompts");
        
        vfs.add_builtin("test", "content");
        assert_eq!(vfs.get_source("test"), Some(&FileSource::Builtin));
        assert_eq!(vfs.get_source("nonexistent"), None);
    }
}