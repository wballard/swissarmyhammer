//! Prompt management and loading functionality

use crate::{Result, SwissArmyHammerError, Template};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a single prompt with metadata and template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Unique name of the prompt
    pub name: String,
    
    /// Description of what the prompt does
    pub description: Option<String>,
    
    /// Category for organization
    pub category: Option<String>,
    
    /// Tags for searching
    pub tags: Vec<String>,
    
    /// Template content
    pub template: String,
    
    /// Required arguments
    pub arguments: Vec<ArgumentSpec>,
    
    /// Source file path
    pub source: Option<PathBuf>,
    
    /// Additional metadata
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Specification for a template argument
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentSpec {
    /// Argument name
    pub name: String,
    
    /// Description of the argument
    pub description: Option<String>,
    
    /// Whether the argument is required
    pub required: bool,
    
    /// Default value if not provided
    pub default: Option<String>,
    
    /// Argument type hint
    pub type_hint: Option<String>,
}

impl Prompt {
    /// Create a new prompt
    pub fn new(name: impl Into<String>, template: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            category: None,
            tags: Vec::new(),
            template: template.into(),
            arguments: Vec::new(),
            source: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Render the prompt with given arguments
    pub fn render(&self, args: &HashMap<String, String>) -> Result<String> {
        let template = Template::new(&self.template)?;
        
        // Validate required arguments
        for arg in &self.arguments {
            if arg.required && !args.contains_key(&arg.name) {
                return Err(SwissArmyHammerError::Template(
                    format!("Required argument '{}' not provided", arg.name)
                ));
            }
        }
        
        // Start with all provided arguments
        let mut render_args = args.clone();
        
        // Add defaults for missing arguments
        for arg in &self.arguments {
            if !render_args.contains_key(&arg.name) {
                if let Some(default) = &arg.default {
                    render_args.insert(arg.name.clone(), default.clone());
                }
            }
        }
        
        template.render(&render_args)
    }
    
    /// Add an argument specification
    pub fn add_argument(mut self, arg: ArgumentSpec) -> Self {
        self.arguments.push(arg);
        self
    }
    
    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Set the category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
    
    /// Add tags
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Manages a collection of prompts
pub struct PromptLibrary {
    storage: Box<dyn crate::StorageBackend>,
}

impl PromptLibrary {
    /// Create a new prompt library with default storage
    pub fn new() -> Self {
        Self {
            storage: Box::new(crate::storage::MemoryStorage::new()),
        }
    }
    
    /// Create a prompt library with custom storage
    pub fn with_storage(storage: Box<dyn crate::StorageBackend>) -> Self {
        Self { storage }
    }
    
    /// Add prompts from a directory
    pub fn add_directory(&mut self, path: impl AsRef<Path>) -> Result<usize> {
        let loader = PromptLoader::new();
        let prompts = loader.load_directory(path)?;
        let count = prompts.len();
        
        for prompt in prompts {
            self.storage.store(prompt)?;
        }
        
        Ok(count)
    }
    
    /// Get a prompt by name
    pub fn get(&self, name: &str) -> Result<Prompt> {
        self.storage.get(name)
    }
    
    /// List all prompts
    pub fn list(&self) -> Result<Vec<Prompt>> {
        self.storage.list()
    }
    
    /// Search prompts
    pub fn search(&self, query: &str) -> Result<Vec<Prompt>> {
        self.storage.search(query)
    }
    
    /// Add a single prompt
    pub fn add(&mut self, prompt: Prompt) -> Result<()> {
        self.storage.store(prompt)
    }
    
    /// Remove a prompt
    pub fn remove(&mut self, name: &str) -> Result<()> {
        self.storage.remove(name)
    }
}

impl Default for PromptLibrary {
    fn default() -> Self {
        Self::new()
    }
}

/// Loads prompts from various sources
pub struct PromptLoader {
    /// File extensions to consider
    extensions: Vec<String>,
}

impl PromptLoader {
    /// Create a new prompt loader
    pub fn new() -> Self {
        Self {
            extensions: vec!["md".to_string(), "markdown".to_string()],
        }
    }
    
    /// Load prompts from a directory
    pub fn load_directory(&self, path: impl AsRef<Path>) -> Result<Vec<Prompt>> {
        let path = path.as_ref();
        let mut prompts = Vec::new();
        
        if !path.exists() {
            return Err(SwissArmyHammerError::Io(
                std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("Directory not found: {}", path.display())
                )
            ));
        }
        
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && self.is_prompt_file(path) {
                if let Ok(prompt) = self.load_file(path) {
                    prompts.push(prompt);
                }
            }
        }
        
        Ok(prompts)
    }
    
    /// Load a single prompt file
    pub fn load_file(&self, path: impl AsRef<Path>) -> Result<Prompt> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;
        
        let (metadata, template) = self.parse_front_matter(&content)?;
        
        let name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| SwissArmyHammerError::Other(
                "Invalid file name".to_string()
            ))?
            .to_string();
        
        let mut prompt = Prompt::new(name, template);
        prompt.source = Some(path.to_path_buf());
        
        // Parse metadata
        if let Some(metadata) = metadata {
            if let Some(desc) = metadata.get("description").and_then(|v| v.as_str()) {
                prompt.description = Some(desc.to_string());
            }
            if let Some(cat) = metadata.get("category").and_then(|v| v.as_str()) {
                prompt.category = Some(cat.to_string());
            }
            if let Some(tags) = metadata.get("tags").and_then(|v| v.as_array()) {
                prompt.tags = tags
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(String::from)
                    .collect();
            }
            if let Some(args) = metadata.get("arguments").and_then(|v| v.as_array()) {
                for arg in args {
                    if let Some(arg_obj) = arg.as_object() {
                        let name = arg_obj.get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();
                        
                        let arg_spec = ArgumentSpec {
                            name,
                            description: arg_obj.get("description")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            required: arg_obj.get("required")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            default: arg_obj.get("default")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            type_hint: arg_obj.get("type")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                        };
                        
                        prompt.arguments.push(arg_spec);
                    }
                }
            }
        }
        
        Ok(prompt)
    }
    
    /// Check if a path is a prompt file
    fn is_prompt_file(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| self.extensions.contains(&ext.to_lowercase()))
            .unwrap_or(false)
    }
    
    /// Parse front matter from content
    fn parse_front_matter(&self, content: &str) -> Result<(Option<serde_json::Value>, String)> {
        if content.starts_with("---\n") {
            let parts: Vec<&str> = content.splitn(3, "---\n").collect();
            if parts.len() >= 3 {
                let yaml_content = parts[1];
                let template = parts[2].trim_start().to_string();
                
                let metadata: serde_yaml::Value = serde_yaml::from_str(yaml_content)?;
                let json_value = serde_json::to_value(metadata)
                    .map_err(|e| SwissArmyHammerError::Other(e.to_string()))?;
                
                return Ok((Some(json_value), template));
            }
        }
        
        Ok((None, content.to_string()))
    }
}

impl Default for PromptLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_prompt_creation() {
        let prompt = Prompt::new("test", "Hello {{ name }}!");
        assert_eq!(prompt.name, "test");
        assert_eq!(prompt.template, "Hello {{ name }}!");
    }
    
    #[test]
    fn test_prompt_render() {
        let prompt = Prompt::new("test", "Hello {{ name }}!")
            .add_argument(ArgumentSpec {
                name: "name".to_string(),
                description: None,
                required: true,
                default: None,
                type_hint: None,
            });
        
        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());
        
        let result = prompt.render(&args).unwrap();
        assert_eq!(result, "Hello World!");
    }
}