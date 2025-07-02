//! Storage abstractions and implementations

use crate::{Prompt, Result, SwissArmyHammerError};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Trait for prompt storage backends
pub trait StorageBackend: Send + Sync {
    /// Store a prompt
    fn store(&mut self, prompt: Prompt) -> Result<()>;
    
    /// Get a prompt by name
    fn get(&self, name: &str) -> Result<Prompt>;
    
    /// List all prompts
    fn list(&self) -> Result<Vec<Prompt>>;
    
    /// Remove a prompt
    fn remove(&mut self, name: &str) -> Result<()>;
    
    /// Search prompts by query
    fn search(&self, query: &str) -> Result<Vec<Prompt>>;
    
    /// Check if a prompt exists
    fn exists(&self, name: &str) -> Result<bool> {
        self.get(name).map(|_| true).or_else(|e| match e {
            SwissArmyHammerError::PromptNotFound(_) => Ok(false),
            _ => Err(e),
        })
    }
    
    /// Get total count of prompts
    fn count(&self) -> Result<usize> {
        self.list().map(|prompts| prompts.len())
    }
}

/// In-memory storage implementation
pub struct MemoryStorage {
    prompts: HashMap<String, Prompt>,
}

impl MemoryStorage {
    /// Create a new memory storage
    pub fn new() -> Self {
        Self {
            prompts: HashMap::new(),
        }
    }
}

impl Default for MemoryStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl StorageBackend for MemoryStorage {
    fn store(&mut self, prompt: Prompt) -> Result<()> {
        self.prompts.insert(prompt.name.clone(), prompt);
        Ok(())
    }
    
    fn get(&self, name: &str) -> Result<Prompt> {
        self.prompts
            .get(name)
            .cloned()
            .ok_or_else(|| SwissArmyHammerError::PromptNotFound(name.to_string()))
    }
    
    fn list(&self) -> Result<Vec<Prompt>> {
        Ok(self.prompts.values().cloned().collect())
    }
    
    fn remove(&mut self, name: &str) -> Result<()> {
        self.prompts
            .remove(name)
            .ok_or_else(|| SwissArmyHammerError::PromptNotFound(name.to_string()))?;
        Ok(())
    }
    
    fn search(&self, query: &str) -> Result<Vec<Prompt>> {
        let query_lower = query.to_lowercase();
        Ok(self.prompts
            .values()
            .filter(|prompt| {
                prompt.name.to_lowercase().contains(&query_lower)
                    || prompt.description.as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || prompt.tags.iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
                    || prompt.category.as_ref()
                        .map(|c| c.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .cloned()
            .collect())
    }
}

/// File system storage implementation
pub struct FileSystemStorage {
    base_path: std::path::PathBuf,
    cache: dashmap::DashMap<String, Prompt>,
}

impl FileSystemStorage {
    /// Create a new file system storage
    pub fn new(base_path: impl AsRef<Path>) -> Result<Self> {
        let base_path = base_path.as_ref().to_path_buf();
        
        if !base_path.exists() {
            std::fs::create_dir_all(&base_path)?;
        }
        
        let storage = Self {
            base_path,
            cache: dashmap::DashMap::new(),
        };
        
        // Load existing prompts into cache
        storage.reload_cache()?;
        
        Ok(storage)
    }
    
    /// Reload the cache from disk
    pub fn reload_cache(&self) -> Result<()> {
        self.cache.clear();
        
        for entry in walkdir::WalkDir::new(&self.base_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                if let Ok(content) = std::fs::read_to_string(path) {
                    if let Ok(prompt) = serde_yaml::from_str::<Prompt>(&content) {
                        self.cache.insert(prompt.name.clone(), prompt);
                    }
                }
            }
        }
        
        Ok(())
    }
    
    fn prompt_path(&self, name: &str) -> std::path::PathBuf {
        self.base_path.join(format!("{}.yaml", name))
    }
}

impl StorageBackend for FileSystemStorage {
    fn store(&mut self, prompt: Prompt) -> Result<()> {
        let path = self.prompt_path(&prompt.name);
        let content = serde_yaml::to_string(&prompt)?;
        std::fs::write(&path, content)?;
        self.cache.insert(prompt.name.clone(), prompt);
        Ok(())
    }
    
    fn get(&self, name: &str) -> Result<Prompt> {
        if let Some(prompt) = self.cache.get(name) {
            return Ok(prompt.clone());
        }
        
        let path = self.prompt_path(name);
        if !path.exists() {
            return Err(SwissArmyHammerError::PromptNotFound(name.to_string()));
        }
        
        let content = std::fs::read_to_string(&path)?;
        let prompt: Prompt = serde_yaml::from_str(&content)?;
        self.cache.insert(name.to_string(), prompt.clone());
        
        Ok(prompt)
    }
    
    fn list(&self) -> Result<Vec<Prompt>> {
        Ok(self.cache.iter().map(|entry| entry.value().clone()).collect())
    }
    
    fn remove(&mut self, name: &str) -> Result<()> {
        let path = self.prompt_path(name);
        if !path.exists() {
            return Err(SwissArmyHammerError::PromptNotFound(name.to_string()));
        }
        
        std::fs::remove_file(path)?;
        self.cache.remove(name);
        Ok(())
    }
    
    fn search(&self, query: &str) -> Result<Vec<Prompt>> {
        let query_lower = query.to_lowercase();
        Ok(self.cache
            .iter()
            .filter(|entry| {
                let prompt = entry.value();
                prompt.name.to_lowercase().contains(&query_lower)
                    || prompt.description.as_ref()
                        .map(|d| d.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
                    || prompt.tags.iter()
                        .any(|tag| tag.to_lowercase().contains(&query_lower))
                    || prompt.category.as_ref()
                        .map(|c| c.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .map(|entry| entry.value().clone())
            .collect())
    }
}

/// Main prompt storage that can use different backends
pub struct PromptStorage {
    backend: Arc<dyn StorageBackend>,
}

impl PromptStorage {
    /// Create a new prompt storage with the given backend
    pub fn new(backend: Arc<dyn StorageBackend>) -> Self {
        Self { backend }
    }
    
    /// Create with memory backend
    pub fn memory() -> Self {
        Self::new(Arc::new(MemoryStorage::new()))
    }
    
    /// Create with file system backend
    pub fn file_system(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::new(Arc::new(FileSystemStorage::new(path)?)))
    }
    
    /// Store a prompt
    pub fn store(&mut self, prompt: Prompt) -> Result<()> {
        Arc::get_mut(&mut self.backend)
            .ok_or_else(|| SwissArmyHammerError::Storage(
                "Cannot get mutable reference to storage backend".to_string()
            ))?
            .store(prompt)
    }
    
    /// Get a prompt by name
    pub fn get(&self, name: &str) -> Result<Prompt> {
        self.backend.get(name)
    }
    
    /// List all prompts
    pub fn list(&self) -> Result<Vec<Prompt>> {
        self.backend.list()
    }
    
    /// Remove a prompt
    pub fn remove(&mut self, name: &str) -> Result<()> {
        Arc::get_mut(&mut self.backend)
            .ok_or_else(|| SwissArmyHammerError::Storage(
                "Cannot get mutable reference to storage backend".to_string()
            ))?
            .remove(name)
    }
    
    /// Search prompts
    pub fn search(&self, query: &str) -> Result<Vec<Prompt>> {
        self.backend.search(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::ArgumentSpec;
    
    #[test]
    fn test_memory_storage() {
        let mut storage = MemoryStorage::new();
        
        let prompt = Prompt::new("test", "Hello {{ name }}!");
        storage.store(prompt.clone()).unwrap();
        
        let retrieved = storage.get("test").unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.template, "Hello {{ name }}!");
        
        let list = storage.list().unwrap();
        assert_eq!(list.len(), 1);
        
        storage.remove("test").unwrap();
        assert!(storage.get("test").is_err());
    }
    
    #[test]
    fn test_search() {
        let mut storage = MemoryStorage::new();
        
        let prompt1 = Prompt::new("code-review", "Review this code")
            .with_description("A prompt for code review")
            .with_tags(vec!["code".to_string(), "review".to_string()]);
        
        let prompt2 = Prompt::new("bug-fix", "Fix this bug")
            .with_description("A prompt for fixing bugs")
            .with_tags(vec!["bug".to_string(), "fix".to_string()]);
        
        storage.store(prompt1).unwrap();
        storage.store(prompt2).unwrap();
        
        let results = storage.search("code").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "code-review");
        
        let results = storage.search("bug").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "bug-fix");
    }
}