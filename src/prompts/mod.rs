use std::path::{Path, PathBuf};
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use std::collections::HashMap;
use walkdir::WalkDir;
use anyhow::{Result, Context};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use dashmap::DashMap;
use notify::{Watcher, RecommendedWatcher, RecursiveMode, Event, EventKind};
use tokio::sync::mpsc;
use tokio::time::timeout;
use serde_json::Value;
use crate::template::{LiquidEngine, TemplateArgument as TemplateArg};

#[derive(RustEmbed)]
#[folder = "prompts/builtin/"]
struct BuiltinPrompts;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PromptSource {
    BuiltIn,
    User,
    Local,
}

impl PromptSource {
    pub fn priority(&self) -> u8 {
        match self {
            PromptSource::BuiltIn => 1,
            PromptSource::User => 2,
            PromptSource::Local => 3,
        }
    }

    pub fn from_path(path: &str) -> Self {
        if path.starts_with("builtin:") {
            PromptSource::BuiltIn
        } else if path.contains("/.swissarmyhammer/") {
            // Check if this is a user directory path
            if let Some(home_dir) = dirs::home_dir() {
                let home_path = home_dir.to_string_lossy();
                let expected_user_path = format!("{}/.swissarmyhammer/", home_path);
                if path.contains(&expected_user_path) {
                    return PromptSource::User;
                }
            }
            // If not user directory, it's local
            PromptSource::Local
        } else {
            // Any other path is considered local
            PromptSource::Local
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PromptArgument {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct PromptFrontMatter {
    pub name: Option<String>,
    pub title: String,
    pub description: String,
    #[serde(default)]
    pub arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Prompt {
    pub name: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub arguments: Vec<PromptArgument>,
    pub content: String,
    pub source_path: String,
    pub source: PromptSource,
    pub relative_path: String,
}

impl Prompt {
    pub fn new(name: String, content: String, source_path: String) -> Self {
        let source = PromptSource::from_path(&source_path);
        let relative_path = Self::extract_relative_path(&source_path);
        
        Self {
            name,
            title: None,
            description: None,
            arguments: Vec::new(),
            content,
            source_path,
            source,
            relative_path,
        }
    }

    pub fn new_with_front_matter(
        name: String,
        front_matter: Option<PromptFrontMatter>,
        content: String,
        source_path: String,
    ) -> Self {
        let source = PromptSource::from_path(&source_path);
        let relative_path = Self::extract_relative_path(&source_path);
        
        if let Some(fm) = front_matter {
            Self {
                name: fm.name.unwrap_or(name),
                title: Some(fm.title),
                description: Some(fm.description),
                arguments: fm.arguments.into_iter().map(|arg| {
                    // Ensure default field is mapped correctly
                    PromptArgument {
                        name: arg.name,
                        description: arg.description,
                        required: arg.required,
                        default: arg.default,
                    }
                }).collect(),
                content,
                source_path,
                source,
                relative_path,
            }
        } else {
            Self::new(name, content, source_path)
        }
    }

    fn extract_relative_path(source_path: &str) -> String {
        if source_path.starts_with("builtin:/") {
            // For builtin prompts, remove "builtin:/" prefix
            source_path.strip_prefix("builtin:/").unwrap_or(source_path).to_string()
        } else if let Some(pos) = source_path.find("/.swissarmyhammer/") {
            // For user/local prompts, extract path after .swissarmyhammer/
            let after_dir = &source_path[pos + "/.swissarmyhammer/".len()..];
            after_dir.to_string()
        } else {
            // Fallback to using the filename
            std::path::Path::new(source_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or(source_path)
                .to_string()
        }
    }
    
    /// Process the prompt template with the given arguments
    pub fn process_template(&self, arguments: &HashMap<String, Value>) -> Result<String> {
        let engine = LiquidEngine::new();
        
        // Convert our PromptArgument to template::TemplateArgument
        let template_args: Vec<TemplateArg> = self.arguments.iter()
            .map(|arg| TemplateArg {
                name: arg.name.clone(),
                description: Some(arg.description.clone()),
                required: arg.required,
                default_value: arg.default.clone(),
            })
            .collect();
        
        engine.process_with_validation(&self.content, arguments, &template_args)
    }
}

#[derive(Clone)]
pub struct PromptStorage {
    prompts: Arc<DashMap<String, Prompt>>,
}

impl Default for PromptStorage {
    fn default() -> Self {
        Self {
            prompts: Arc::new(DashMap::new()),
        }
    }
}

impl PromptStorage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&self, name: String, prompt: Prompt) {
        self.prompts.insert(name, prompt);
    }

    pub fn get(&self, name: &str) -> Option<Prompt> {
        self.prompts.get(name).map(|entry| entry.value().clone())
    }

    pub fn remove(&self, name: &str) -> Option<Prompt> {
        self.prompts.remove(name).map(|(_, prompt)| prompt)
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.prompts.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = (String, Prompt)> + '_ {
        self.prompts.iter().map(|entry| (entry.key().clone(), entry.value().clone()))
    }

    pub fn find_by_relative_path(&self, relative_path: &str) -> Option<(String, Prompt)> {
        self.prompts.iter()
            .find(|entry| entry.value().relative_path == relative_path)
            .map(|entry| (entry.key().clone(), entry.value().clone()))
    }
}

#[derive(Default)]
pub struct PromptLoader {
    pub storage: PromptStorage,
}

impl PromptLoader {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            // Missing directories are okay, just skip
            return Ok(());
        }

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                match self.load_prompt_from_file(path) {
                    Ok(prompt) => {
                        self.insert_prompt_with_override(prompt);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load prompt from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(())
    }

    pub fn load_prompt_from_file(&self, path: &Path) -> Result<Prompt> {
        let file_content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read prompt file: {:?}", path))?;

        // Parse front matter and content
        let (front_matter, markdown_content) = self.parse_front_matter(&file_content)?;

        // Extract prompt name from file path
        let name = self.extract_prompt_name(path)?;
        let source_path = path.to_string_lossy().to_string();

        // Validate front matter if present
        if let Some(ref fm) = front_matter {
            self.validate_front_matter(fm)?;
        }

        Ok(Prompt::new_with_front_matter(name, front_matter, markdown_content, source_path))
    }

    fn extract_prompt_name(&self, path: &Path) -> Result<String> {
        // Remove .md extension and use the path relative to the scan directory
        let stem = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid file name"))?;

        // If the path has parent directories within the scan dir, include them in the name
        // For example: tools/debug.md -> tools/debug
        if let Some(parent) = path.parent() {
            if let Some(parent_name) = parent.file_name() {
                if parent_name != "." && parent_name != ".swissarmyhammer" {
                    return Ok(format!("{}/{}", parent_name.to_string_lossy(), stem));
                }
            }
        }

        Ok(stem.to_string())
    }

    pub fn load_all(&mut self) -> Result<()> {
        // Load built-in prompts first (will be implemented with rust-embed)
        self.load_builtin_prompts()?;

        // Load user prompts from ~/.swissarmyhammer/
        if let Some(home_dir) = dirs::home_dir() {
            let user_prompts_dir = home_dir.join(".swissarmyhammer");
            self.scan_directory(&user_prompts_dir)?;
        }

        // Load local prompts from $PWD/.swissarmyhammer/
        let local_prompts_dir = std::env::current_dir()?.join(".swissarmyhammer");
        self.scan_directory(&local_prompts_dir)?;

        Ok(())
    }

    fn load_builtin_prompts(&mut self) -> Result<()> {
        for file_path in BuiltinPrompts::iter() {
            if let Some(content) = BuiltinPrompts::get(&file_path) {
                let content_str = std::str::from_utf8(&content.data)
                    .with_context(|| format!("Invalid UTF-8 in builtin prompt: {}", file_path))?;
                
                // Parse front matter and content
                let (front_matter, markdown_content) = self.parse_front_matter(content_str)?;
                
                // Extract name from path (remove .md extension)
                let name = file_path
                    .strip_suffix(".md")
                    .unwrap_or(&file_path)
                    .to_string();
                
                // Validate front matter if present
                if let Some(ref fm) = front_matter {
                    self.validate_front_matter(fm)?;
                }
                
                let source_path = format!("builtin:/{}", file_path);
                let prompt = Prompt::new_with_front_matter(name.clone(), front_matter, markdown_content, source_path);
                
                self.insert_prompt_with_override(prompt);
            }
        }
        Ok(())
    }

    fn parse_front_matter(&self, content: &str) -> Result<(Option<PromptFrontMatter>, String)> {
        // Check if content starts with YAML front matter delimiter
        if !content.starts_with("---\n") {
            return Ok((None, content.to_string()));
        }

        // Find the end of the front matter
        let content_after_first_delimiter = &content[4..]; // Skip the first "---\n"
        if let Some(end_pos) = content_after_first_delimiter.find("\n---\n") {
            let front_matter_yaml = &content_after_first_delimiter[..end_pos];
            let markdown_content = &content_after_first_delimiter[end_pos + 5..]; // Skip "\n---\n"

            match serde_yaml::from_str::<PromptFrontMatter>(front_matter_yaml) {
                Ok(front_matter) => Ok((Some(front_matter), markdown_content.to_string())),
                Err(e) => {
                    tracing::warn!("Failed to parse YAML front matter: {}", e);
                    // If front matter is invalid, treat as pure markdown
                    Ok((None, content.to_string()))
                }
            }
        } else {
            // If we can't find the closing delimiter, treat as pure markdown
            tracing::warn!("Front matter opened but not properly closed");
            Ok((None, content.to_string()))
        }
    }

    fn validate_front_matter(&self, front_matter: &PromptFrontMatter) -> Result<()> {
        // Validate that argument names are valid identifiers
        for arg in &front_matter.arguments {
            if arg.name.is_empty() {
                return Err(anyhow::anyhow!("Argument name cannot be empty"));
            }
            
            // Check if argument name is a valid identifier (starts with letter/underscore, contains only alphanumeric/underscore)
            if !arg.name.chars().next().unwrap_or('0').is_alphabetic() && !arg.name.starts_with('_') {
                return Err(anyhow::anyhow!("Argument name '{}' must start with a letter or underscore", arg.name));
            }
            
            if !arg.name.chars().all(|c| c.is_alphanumeric() || c == '_') {
                return Err(anyhow::anyhow!("Argument name '{}' must contain only alphanumeric characters and underscores", arg.name));
            }
        }
        
        Ok(())
    }

    pub fn insert_prompt_with_override(&mut self, prompt: Prompt) {
        let name_key = prompt.name.clone();
        
        // Check if a prompt with the same relative path already exists
        if let Some((existing_name, existing_prompt)) = self.storage.find_by_relative_path(&prompt.relative_path) {
            if prompt.source.priority() > existing_prompt.source.priority() {
                tracing::debug!(
                    "Overriding prompt '{}' (relative path: '{}') from {:?} with {:?} (priority {} > {})",
                    existing_name,
                    prompt.relative_path,
                    existing_prompt.source,
                    prompt.source,
                    prompt.source.priority(),
                    existing_prompt.source.priority()
                );
                self.storage.remove(&existing_name);
                self.storage.insert(name_key, prompt);
            } else {
                tracing::debug!(
                    "Keeping existing prompt '{}' (relative path: '{}') from {:?}, ignoring {:?} (priority {} <= {})",
                    existing_name,
                    prompt.relative_path,
                    existing_prompt.source,
                    prompt.source,
                    existing_prompt.source.priority(),
                    prompt.source.priority()
                );
            }
        } else {
            tracing::debug!(
                "Adding prompt '{}' (relative path: '{}') from {:?}",
                name_key,
                prompt.relative_path,
                prompt.source
            );
            self.storage.insert(name_key, prompt);
        }
    }
}

#[derive(Debug)]
pub enum WatchEvent {
    PromptChanged(PathBuf),
    PromptDeleted(PathBuf),
    DirectoryChanged(PathBuf),
}

pub struct PromptWatcher {
    _watcher: RecommendedWatcher,
    event_receiver: mpsc::UnboundedReceiver<WatchEvent>,
    storage: PromptStorage,
}

impl PromptWatcher {
    pub fn new(storage: PromptStorage) -> Result<Self> {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        // Create a debounced sender to handle rapid file system events
        let debounced_sender = Self::create_debounced_sender(event_sender);
        
        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    if let Err(e) = Self::handle_fs_event(&debounced_sender, event) {
                        tracing::warn!("Failed to handle file system event: {}", e);
                    }
                }
                Err(e) => tracing::error!("File watcher error: {}", e),
            }
        })?;

        // Watch user and local .swissarmyhammer directories
        if let Some(home_dir) = dirs::home_dir() {
            let user_dir = home_dir.join(".swissarmyhammer");
            if user_dir.exists() {
                watcher.watch(&user_dir, RecursiveMode::Recursive)
                    .with_context(|| format!("Failed to watch user directory: {:?}", user_dir))?;
                tracing::info!("Watching user prompts directory: {:?}", user_dir);
            }
        }

        let local_dir = std::env::current_dir()?.join(".swissarmyhammer");
        if local_dir.exists() {
            watcher.watch(&local_dir, RecursiveMode::Recursive)
                .with_context(|| format!("Failed to watch local directory: {:?}", local_dir))?;
            tracing::info!("Watching local prompts directory: {:?}", local_dir);
        }

        Ok(Self {
            _watcher: watcher,
            event_receiver,
            storage,
        })
    }

    fn create_debounced_sender(sender: mpsc::UnboundedSender<WatchEvent>) -> mpsc::UnboundedSender<(PathBuf, EventKind)> {
        let (debounce_sender, mut debounce_receiver) = mpsc::unbounded_channel::<(PathBuf, EventKind)>();
        
        // Spawn a task to handle debouncing
        tokio::spawn(async move {
            let mut pending_events: HashMap<PathBuf, EventKind> = HashMap::new();
            
            loop {
                // Wait for events or timeout
                let event_result = timeout(Duration::from_millis(100), debounce_receiver.recv()).await;
                
                match event_result {
                    Ok(Some((path, kind))) => {
                        // New event received, add to pending
                        pending_events.insert(path, kind);
                    }
                    Ok(None) => {
                        // Channel closed
                        break;
                    }
                    Err(_) => {
                        // Timeout - process pending events
                        for (path, kind) in pending_events.drain() {
                            let watch_event = match kind {
                                EventKind::Remove(_) => WatchEvent::PromptDeleted(path),
                                EventKind::Create(_) | EventKind::Modify(_) => {
                                    if path.is_dir() {
                                        WatchEvent::DirectoryChanged(path)
                                    } else {
                                        WatchEvent::PromptChanged(path)
                                    }
                                }
                                _ => continue,
                            };
                            
                            if let Err(e) = sender.send(watch_event) {
                                tracing::error!("Failed to send debounced event: {}", e);
                                break;
                            }
                        }
                    }
                }
            }
        });
        
        debounce_sender
    }

    fn handle_fs_event(sender: &mpsc::UnboundedSender<(PathBuf, EventKind)>, event: Event) -> Result<()> {
        // Filter for markdown files in .swissarmyhammer directories
        for path in event.paths {
            if (path.extension().and_then(|s| s.to_str()) == Some("md") || path.is_dir()) && path.to_string_lossy().contains("/.swissarmyhammer/") {
                sender.send((path, event.kind))
                    .map_err(|e| anyhow::anyhow!("Failed to send file system event: {}", e))?;
            }
        }
        Ok(())
    }

    pub async fn run(mut self, mut prompt_loader: PromptLoader) -> Result<()> {
        tracing::info!("Starting prompt file watcher...");
        
        while let Some(event) = self.event_receiver.recv().await {
            match event {
                WatchEvent::PromptChanged(path) => {
                    tracing::debug!("Prompt file changed: {:?}", path);
                    if let Err(e) = self.handle_prompt_changed(&mut prompt_loader, &path).await {
                        tracing::warn!("Failed to reload changed prompt {:?}: {}", path, e);
                    }
                }
                WatchEvent::PromptDeleted(path) => {
                    tracing::debug!("Prompt file deleted: {:?}", path);
                    if let Err(e) = self.handle_prompt_deleted(&path).await {
                        tracing::warn!("Failed to handle deleted prompt {:?}: {}", path, e);
                    }
                }
                WatchEvent::DirectoryChanged(path) => {
                    tracing::debug!("Directory changed: {:?}", path);
                    if let Err(e) = self.handle_directory_changed(&mut prompt_loader, &path).await {
                        tracing::warn!("Failed to handle directory change {:?}: {}", path, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    async fn handle_prompt_changed(&self, prompt_loader: &mut PromptLoader, path: &Path) -> Result<()> {
        if !path.exists() {
            return Ok(()); // File might have been deleted between events
        }

        match prompt_loader.load_prompt_from_file(path) {
            Ok(prompt) => {
                let name = prompt.name.clone();
                prompt_loader.insert_prompt_with_override(prompt);
                tracing::info!("Reloaded prompt '{}' from {:?}", name, path);
            }
            Err(e) => {
                tracing::warn!("Failed to reload prompt from {:?}: {}", path, e);
            }
        }
        
        Ok(())
    }

    async fn handle_prompt_deleted(&self, path: &Path) -> Result<()> {
        // Find and remove the prompt by searching for its path
        let path_str = path.to_string_lossy();
        let mut to_remove = Vec::new();
        
        for (name, prompt) in self.storage.iter() {
            if prompt.source_path == path_str {
                to_remove.push(name);
            }
        }
        
        for name in to_remove {
            self.storage.remove(&name);
            tracing::info!("Removed deleted prompt '{}' from {:?}", name, path);
        }
        
        Ok(())
    }

    async fn handle_directory_changed(&self, prompt_loader: &mut PromptLoader, path: &Path) -> Result<()> {
        if path.exists() && path.is_dir() {
            tracing::info!("Rescanning directory: {:?}", path);
            prompt_loader.scan_directory(path)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_prompt_creation() {
        let prompt = Prompt::new(
            "test-prompt".to_string(),
            "# Test Prompt\nThis is a test".to_string(),
            "/path/to/prompt.md".to_string(),
        );
        
        assert_eq!(prompt.name, "test-prompt");
        assert_eq!(prompt.content, "# Test Prompt\nThis is a test");
        assert_eq!(prompt.source_path, "/path/to/prompt.md");
    }

    #[test]
    fn test_prompt_loader_creation() {
        let loader = PromptLoader::new();
        assert!(loader.storage.is_empty());
    }

    #[test]
    fn test_scan_directory() {
        let mut loader = PromptLoader::new();
        let test_dir = PathBuf::from("test_data");
        
        // This will fail initially as scan_directory isn't implemented
        let result = loader.scan_directory(&test_dir);
        assert!(result.is_ok());
    }

    #[test]
    fn test_load_builtin_prompts() {
        let mut loader = PromptLoader::new();
        let result = loader.load_builtin_prompts();
        assert!(result.is_ok());
        
        // Check that at least the example prompt is loaded
        assert!(loader.storage.contains_key("example"));
        let example = loader.storage.get("example").unwrap();
        assert!(example.content.contains("Example Prompt"));
        assert_eq!(example.source_path, "builtin:/example.md");
    }

    #[test]
    fn test_load_all() {
        let mut loader = PromptLoader::new();
        let result = loader.load_all();
        assert!(result.is_ok());
        
        // At minimum, builtin prompts should be loaded
        assert!(!loader.storage.is_empty());
        assert!(loader.storage.contains_key("example"));
    }

    #[test]
    fn test_parse_front_matter() {
        let content = r#"---
title: Test Prompt
description: A test prompt for parsing
arguments:
  - name: arg1
    description: First argument
    required: true
  - name: arg2
    description: Second argument
    default: "default_value"
---

# Test Content

This is the markdown content."#;

        let loader = PromptLoader::new();
        let (front_matter, markdown_content) = loader.parse_front_matter(content).unwrap();
        
        assert!(front_matter.is_some());
        let fm = front_matter.unwrap();
        assert_eq!(fm.title, "Test Prompt");
        assert_eq!(fm.description, "A test prompt for parsing");
        assert_eq!(fm.arguments.len(), 2);
        assert_eq!(fm.arguments[0].name, "arg1");
        assert!(fm.arguments[0].required);
        assert_eq!(fm.arguments[1].name, "arg2");
        assert_eq!(fm.arguments[1].default, Some("default_value".to_string()));
        
        assert_eq!(markdown_content.trim(), "# Test Content\n\nThis is the markdown content.");
    }

    #[test]
    fn test_parse_no_front_matter() {
        let content = "# Simple Markdown\n\nNo front matter here.";
        
        let loader = PromptLoader::new();
        let (front_matter, markdown_content) = loader.parse_front_matter(content).unwrap();
        
        assert!(front_matter.is_none());
        assert_eq!(markdown_content, content);
    }

    #[test]
    fn test_load_prompts_with_front_matter() {
        let mut loader = PromptLoader::new();
        let result = loader.load_builtin_prompts();
        assert!(result.is_ok());
        
        // Check that help prompt is loaded with front matter
        assert!(loader.storage.contains_key("help"));
        let help_prompt = loader.storage.get("help").unwrap();
        assert_eq!(help_prompt.title, Some("Help Assistant".to_string()));
        assert_eq!(help_prompt.description, Some("A prompt for providing helpful assistance and guidance to users".to_string()));
        assert_eq!(help_prompt.arguments.len(), 2);
        assert_eq!(help_prompt.arguments[0].name, "topic");
        assert!(!help_prompt.arguments[0].required);
        assert_eq!(help_prompt.arguments[0].default, Some("general assistance".to_string()));
        
        // Check that plan prompt is loaded with front matter
        assert!(loader.storage.contains_key("plan"));
        let plan_prompt = loader.storage.get("plan").unwrap();
        assert_eq!(plan_prompt.title, Some("Task Planning Assistant".to_string()));
        assert_eq!(plan_prompt.description, Some("A prompt for creating structured plans and breaking down complex tasks".to_string()));
        assert_eq!(plan_prompt.arguments.len(), 3);
        assert_eq!(plan_prompt.arguments[0].name, "task");
        assert!(plan_prompt.arguments[0].required);
        assert_eq!(plan_prompt.arguments[0].default, None);
    }

    #[test]
    fn test_prompt_source_tracking() {
        let builtin_prompt = Prompt::new(
            "test".to_string(),
            "content".to_string(),
            "builtin:/test.md".to_string(),
        );
        assert_eq!(builtin_prompt.source, PromptSource::BuiltIn);
        assert_eq!(builtin_prompt.relative_path, "test.md");

        // Use actual home directory for user prompt test
        if let Some(home_dir) = dirs::home_dir() {
            let user_path = format!("{}/.swissarmyhammer/test.md", home_dir.to_string_lossy());
            let user_prompt = Prompt::new(
                "test".to_string(),
                "content".to_string(),
                user_path,
            );
            assert_eq!(user_prompt.source, PromptSource::User);
            assert_eq!(user_prompt.relative_path, "test.md");
        }

        let local_prompt = Prompt::new(
            "test".to_string(),
            "content".to_string(),
            "/project/.swissarmyhammer/test.md".to_string(),
        );
        assert_eq!(local_prompt.source, PromptSource::Local);
        assert_eq!(local_prompt.relative_path, "test.md");
    }

    #[test]
    fn test_prompt_source_priority() {
        assert!(PromptSource::Local.priority() > PromptSource::User.priority());
        assert!(PromptSource::User.priority() > PromptSource::BuiltIn.priority());
    }

    #[test]
    fn test_prompt_override_logic() {
        let mut loader = PromptLoader::new();
        
        // Test that override works - this test will fail until we implement it
        let builtin_prompt = Prompt::new(
            "example".to_string(),
            "builtin content".to_string(),
            "builtin:/example.md".to_string(),
        );
        
        // Use actual home directory for consistent testing
        let user_path = if let Some(home_dir) = dirs::home_dir() {
            format!("{}/.swissarmyhammer/example.md", home_dir.to_string_lossy())
        } else {
            "/fallback/home/.swissarmyhammer/example.md".to_string()
        };
        
        let user_prompt = Prompt::new(
            "example".to_string(),
            "user content".to_string(),
            user_path,
        );
        
        let local_prompt = Prompt::new(
            "example".to_string(),
            "local content".to_string(),
            "/project/.swissarmyhammer/example.md".to_string(),
        );
        
        // Insert prompts with override logic
        loader.insert_prompt_with_override(builtin_prompt);
        loader.insert_prompt_with_override(user_prompt);
        loader.insert_prompt_with_override(local_prompt);
        
        // Local should win due to highest priority
        assert!(loader.storage.contains_key("example"));
        let final_prompt = loader.storage.get("example").unwrap();
        assert_eq!(final_prompt.content, "local content");
        assert_eq!(final_prompt.source, PromptSource::Local);
    }

    #[test]
    fn test_three_level_override_scenario() {
        let mut loader = PromptLoader::new();
        
        // Simulate the exact scenario described in the requirements:
        // Built-in example.md, User override, Local override
        
        // 1. Load builtin example.md (already exists from previous tests)
        loader.load_builtin_prompts().unwrap();
        
        // Verify builtin is loaded
        assert!(loader.storage.contains_key("example"));
        let builtin_example = loader.storage.get("example").unwrap();
        assert_eq!(builtin_example.source, PromptSource::BuiltIn);
        assert!(builtin_example.content.contains("Example Prompt"));
        
        // 2. Add user override for example.md
        let user_path = if let Some(home_dir) = dirs::home_dir() {
            format!("{}/.swissarmyhammer/example.md", home_dir.to_string_lossy())
        } else {
            "/fallback/home/.swissarmyhammer/example.md".to_string()
        };
        
        let user_override = Prompt::new_with_front_matter(
            "example".to_string(),
            Some(PromptFrontMatter {
                name: None,
                title: "User Customized Example".to_string(),
                description: "A user-customized version of the example prompt".to_string(),
                arguments: vec![],
            }),
            "# User Override\nThis is a user-customized example prompt.".to_string(),
            user_path,
        );
        
        loader.insert_prompt_with_override(user_override);
        
        // Verify user override took effect
        let user_example = loader.storage.get("example").unwrap();
        assert_eq!(user_example.source, PromptSource::User);
        assert_eq!(user_example.title, Some("User Customized Example".to_string()));
        assert!(user_example.content.contains("User Override"));
        
        // 3. Add local override for example.md
        let local_override = Prompt::new_with_front_matter(
            "example".to_string(),
            Some(PromptFrontMatter {
                name: None,
                title: "Local Project Example".to_string(),
                description: "A project-specific version of the example prompt".to_string(),
                arguments: vec![],
            }),
            "# Local Override\nThis is a project-specific example prompt.".to_string(),
            "/project/.swissarmyhammer/example.md".to_string(),
        );
        
        loader.insert_prompt_with_override(local_override);
        
        // Verify local override won (highest priority)
        let final_example = loader.storage.get("example").unwrap();
        assert_eq!(final_example.source, PromptSource::Local);
        assert_eq!(final_example.title, Some("Local Project Example".to_string()));
        assert!(final_example.content.contains("Local Override"));
        assert_eq!(final_example.relative_path, "example.md");
        
        // Verify that only one "example" prompt exists (the local one) 
        // We now have many more prompts in the library
        assert!(loader.storage.len() >= 15); // At least 15 prompts as per requirements
        assert_eq!(
            loader.storage.iter()
                .filter(|(_, p)| p.relative_path == "example.md")
                .count(),
            1
        );
    }

    #[test]
    fn test_prompt_storage_operations() {
        let storage = PromptStorage::new();
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
        assert!(!storage.contains_key("test"));
        
        let prompt = Prompt::new(
            "test".to_string(),
            "content".to_string(),
            "/path/test.md".to_string(),
        );
        
        storage.insert("test".to_string(), prompt.clone());
        
        assert!(!storage.is_empty());
        assert_eq!(storage.len(), 1);
        assert!(storage.contains_key("test"));
        
        let retrieved = storage.get("test").unwrap();
        assert_eq!(retrieved.name, "test");
        assert_eq!(retrieved.content, "content");
        
        let removed = storage.remove("test").unwrap();
        assert_eq!(removed.name, "test");
        assert!(storage.is_empty());
    }

    #[test]
    fn test_prompt_storage_find_by_relative_path() {
        let storage = PromptStorage::new();
        
        let prompt1 = Prompt::new(
            "test1".to_string(),
            "content1".to_string(),
            "builtin:/test.md".to_string(),
        );
        
        let prompt2 = Prompt::new(
            "test2".to_string(),
            "content2".to_string(),
            "/project/.swissarmyhammer/test.md".to_string(),
        );
        
        storage.insert("test1".to_string(), prompt1);
        storage.insert("test2".to_string(), prompt2);
        
        let found = storage.find_by_relative_path("test.md");
        assert!(found.is_some());
        
        // Should find the first one that matches
        let (name, prompt) = found.unwrap();
        assert!(name == "test1" || name == "test2");
        assert_eq!(prompt.relative_path, "test.md");
    }

    #[tokio::test]
    async fn test_prompt_watcher_creation() {
        let storage = PromptStorage::new();
        let watcher_result = PromptWatcher::new(storage);
        
        // This might fail if no .swissarmyhammer directories exist, which is fine
        // We're mainly testing that the function can be called without panicking
        match watcher_result {
            Ok(_) => {
                // Watcher created successfully
            }
            Err(e) => {
                // This is acceptable - no directories to watch might exist
                eprintln!("Watcher creation failed (expected in test environment): {}", e);
            }
        }
    }

    #[test]
    fn test_watch_event_types() {
        use std::path::PathBuf;
        
        let path = PathBuf::from("/test/path.md");
        
        let event1 = WatchEvent::PromptChanged(path.clone());
        let event2 = WatchEvent::PromptDeleted(path.clone());
        let event3 = WatchEvent::DirectoryChanged(path.clone());
        
        // Test that we can create different event types
        match event1 {
            WatchEvent::PromptChanged(_) => {}
            _ => panic!("Expected PromptChanged"),
        }
        
        match event2 {
            WatchEvent::PromptDeleted(_) => {}
            _ => panic!("Expected PromptDeleted"),
        }
        
        match event3 {
            WatchEvent::DirectoryChanged(_) => {}
            _ => panic!("Expected DirectoryChanged"),
        }
    }
    
    #[test]
    fn test_example_prompts_loaded() {
        let mut loader = PromptLoader::new();
        loader.load_all().unwrap();
        
        // Check that code-review prompt is loaded
        let code_review = loader.storage.get("code-review");
        assert!(code_review.is_some(), "code-review prompt should be loaded");
        let code_review = code_review.unwrap();
        assert_eq!(code_review.name, "code-review");
        assert!(code_review.description.is_some());
        assert_eq!(code_review.arguments.len(), 2);
        assert!(code_review.arguments.iter().any(|a| a.name == "file_path" && a.required));
        assert!(code_review.arguments.iter().any(|a| a.name == "context" && !a.required));
        
        // Check that refactor prompt is loaded
        let refactor = loader.storage.get("refactor-patterns");
        assert!(refactor.is_some(), "refactor-patterns prompt should be loaded");
        let refactor = refactor.unwrap();
        assert_eq!(refactor.name, "refactor-patterns");
        assert!(refactor.description.is_some());
        assert_eq!(refactor.arguments.len(), 2);
        assert!(refactor.arguments.iter().all(|a| a.required));
    }
    
    #[test]
    fn test_prompt_template_processing() {
        // Create a prompt with arguments
        let prompt = Prompt {
            name: "test".to_string(),
            title: Some("Test Prompt".to_string()),
            description: Some("A test prompt".to_string()),
            arguments: vec![
                PromptArgument {
                    name: "name".to_string(),
                    description: "The name".to_string(),
                    required: true,
                    default: None,
                },
                PromptArgument {
                    name: "greeting".to_string(),
                    description: "The greeting".to_string(),
                    required: false,
                    default: Some("Hello".to_string()),
                },
            ],
            content: "{{greeting}}, {{name}}! Welcome to {{place}}.".to_string(),
            source_path: "test.md".to_string(),
            source: PromptSource::Local,
            relative_path: "test.md".to_string(),
        };
        
        // Test with all arguments provided
        let mut args = HashMap::new();
        args.insert("name".to_string(), serde_json::json!("Alice"));
        args.insert("greeting".to_string(), serde_json::json!("Hi"));
        args.insert("place".to_string(), serde_json::json!("Wonderland"));
        
        let result = prompt.process_template(&args).unwrap();
        assert_eq!(result, "Hi, Alice! Welcome to Wonderland.");
        
        // Test with default value
        let mut args2 = HashMap::new();
        args2.insert("name".to_string(), serde_json::json!("Bob"));
        args2.insert("place".to_string(), serde_json::json!("the party"));
        
        let result2 = prompt.process_template(&args2).unwrap();
        assert_eq!(result2, "Hello, Bob! Welcome to the party.");
        
        // Test missing required argument
        let args3 = HashMap::new();
        let result3 = prompt.process_template(&args3);
        assert!(result3.is_err());
        assert!(result3.unwrap_err().to_string().contains("Missing required argument 'name'"));
    }
}