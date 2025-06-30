use std::collections::HashMap;
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use anyhow::{Result, Context};
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};

#[derive(RustEmbed)]
#[folder = "var/prompts/"]
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
                arguments: fm.arguments,
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
}

#[derive(Default)]
pub struct PromptLoader {
    pub prompts: HashMap<String, Prompt>,
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

    fn load_prompt_from_file(&self, path: &Path) -> Result<Prompt> {
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
        // We need to check by relative path for override detection, but store by name
        let mut should_insert = true;
        let mut replace_key: Option<String> = None;
        
        // Look for existing prompts with the same relative path
        for (existing_name, existing_prompt) in &self.prompts {
            if existing_prompt.relative_path == prompt.relative_path {
                // Found a prompt with the same relative path
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
                    replace_key = Some(existing_name.clone());
                    break;
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
                    should_insert = false;
                    break;
                }
            }
        }
        
        if should_insert {
            if let Some(key_to_remove) = replace_key {
                self.prompts.remove(&key_to_remove);
            }
            tracing::debug!(
                "Adding prompt '{}' (relative path: '{}') from {:?}",
                name_key,
                prompt.relative_path,
                prompt.source
            );
            self.prompts.insert(name_key, prompt);
        }
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
        assert!(loader.prompts.is_empty());
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
        assert!(loader.prompts.contains_key("example"));
        let example = &loader.prompts["example"];
        assert!(example.content.contains("Example Prompt"));
        assert_eq!(example.source_path, "builtin:/example.md");
    }

    #[test]
    fn test_load_all() {
        let mut loader = PromptLoader::new();
        let result = loader.load_all();
        assert!(result.is_ok());
        
        // At minimum, builtin prompts should be loaded
        assert!(!loader.prompts.is_empty());
        assert!(loader.prompts.contains_key("example"));
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
        assert!(loader.prompts.contains_key("help"));
        let help_prompt = &loader.prompts["help"];
        assert_eq!(help_prompt.title, Some("Help Assistant".to_string()));
        assert_eq!(help_prompt.description, Some("A prompt for providing helpful assistance and guidance to users".to_string()));
        assert_eq!(help_prompt.arguments.len(), 2);
        assert_eq!(help_prompt.arguments[0].name, "topic");
        assert!(!help_prompt.arguments[0].required);
        assert_eq!(help_prompt.arguments[0].default, Some("general assistance".to_string()));
        
        // Check that plan prompt is loaded with front matter
        assert!(loader.prompts.contains_key("plan"));
        let plan_prompt = &loader.prompts["plan"];
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
        assert!(loader.prompts.contains_key("example"));
        let final_prompt = &loader.prompts["example"];
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
        assert!(loader.prompts.contains_key("example"));
        let builtin_example = &loader.prompts["example"];
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
        let user_example = &loader.prompts["example"];
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
        let final_example = &loader.prompts["example"];
        assert_eq!(final_example.source, PromptSource::Local);
        assert_eq!(final_example.title, Some("Local Project Example".to_string()));
        assert!(final_example.content.contains("Local Override"));
        assert_eq!(final_example.relative_path, "example.md");
        
        // Verify that only one "example" prompt exists (the local one)
        assert_eq!(loader.prompts.len(), 3); // example, help, plan
        assert_eq!(
            loader.prompts.values()
                .filter(|p| p.relative_path == "example.md")
                .count(),
            1
        );
    }
}