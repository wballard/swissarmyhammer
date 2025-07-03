//! Prompt management and loading functionality
//!
//! This module provides the core types and functionality for managing prompts,
//! including loading from files, rendering with arguments, and organizing in libraries.
//!
//! # Examples
//!
//! Creating and rendering a simple prompt:
//!
//! ```
//! use swissarmyhammer::{Prompt, ArgumentSpec};
//! use std::collections::HashMap;
//!
//! let prompt = Prompt::new("greet", "Hello {{name}}!")
//!     .with_description("A greeting prompt")
//!     .add_argument(ArgumentSpec {
//!         name: "name".to_string(),
//!         description: Some("Name to greet".to_string()),
//!         required: true,
//!         default: None,
//!         type_hint: Some("string".to_string()),
//!     });
//!
//! let mut args = HashMap::new();
//! args.insert("name".to_string(), "World".to_string());
//! let result = prompt.render(&args).unwrap();
//! assert_eq!(result, "Hello World!");
//! ```

use crate::{Result, SwissArmyHammerError, Template};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a single prompt with metadata and template content.
///
/// A [`Prompt`] encapsulates all the information needed to use a template, including
/// its name, description, required arguments, and the template content itself.
/// Prompts are typically loaded from markdown files with YAML front matter.
///
/// # Prompt File Format
///
/// ```markdown
/// ---
/// title: Code Review
/// description: Reviews code for best practices
/// category: development
/// tags: ["code", "review", "quality"]
/// arguments:
///   - name: code
///     description: The code to review
///     required: true
///   - name: language
///     description: Programming language
///     required: false
///     default: "auto-detect"
/// ---
///
/// Please review this {{language}} code:
///
/// \`\`\`
/// {{code}}
/// \`\`\`
///
/// Focus on best practices, potential bugs, and performance.
/// ```
///
/// # Examples
///
/// ```
/// use swissarmyhammer::{Prompt, ArgumentSpec};
/// use std::collections::HashMap;
///
/// // Create a prompt programmatically
/// let prompt = Prompt::new("debug", "Debug this {{language}} error: {{error}}")
///     .with_description("Helps debug programming errors")
///     .with_category("debugging")
///     .add_argument(ArgumentSpec {
///         name: "error".to_string(),
///         description: Some("The error message".to_string()),
///         required: true,
///         default: None,
///         type_hint: Some("string".to_string()),
///     })
///     .add_argument(ArgumentSpec {
///         name: "language".to_string(),
///         description: Some("Programming language".to_string()),
///         required: false,
///         default: Some("unknown".to_string()),
///         type_hint: Some("string".to_string()),
///     });
///
/// // Render with arguments
/// let mut args = HashMap::new();
/// args.insert("error".to_string(), "NullPointerException".to_string());
/// args.insert("language".to_string(), "Java".to_string());
///
/// let rendered = prompt.render(&args).unwrap();
/// assert!(rendered.contains("Java"));
/// assert!(rendered.contains("NullPointerException"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prompt {
    /// Unique identifier for the prompt.
    ///
    /// This should be a valid filename without extension (e.g., "code-review", "debug-helper").
    /// Used to reference the prompt from CLI and library code.
    pub name: String,

    /// Human-readable description of what the prompt does.
    ///
    /// This appears in help text and prompt listings to help users understand
    /// the prompt's purpose.
    pub description: Option<String>,

    /// Category for organizing prompts into groups.
    ///
    /// Examples: "development", "writing", "analysis", "debugging".
    /// Used for filtering and organizing prompt collections.
    pub category: Option<String>,

    /// Tags for improved searchability.
    ///
    /// Used by search functionality to find relevant prompts.
    /// Should include relevant keywords and concepts.
    pub tags: Vec<String>,

    /// The template content using Liquid syntax.
    ///
    /// This is the actual prompt template that gets rendered with user arguments.
    /// Supports Liquid template syntax including variables (`{{var}}`), conditionals,
    /// loops, and filters.
    ///
    /// # Template Syntax
    ///
    /// - Variables: `{{variable_name}}`
    /// - Conditionals: `{% if condition %}...{% endif %}`
    /// - Loops: `{% for item in items %}...{% endfor %}`
    /// - Filters: `{{text | upper}}`
    pub template: String,

    /// Specifications for template arguments.
    ///
    /// Defines what arguments the template expects, whether they're required,
    /// default values, and documentation. Used for validation and help generation.
    pub arguments: Vec<ArgumentSpec>,

    /// Path to the source file (if loaded from file).
    ///
    /// Used for debugging and file watching functionality.
    /// `None` for programmatically created prompts.
    pub source: Option<PathBuf>,

    /// Additional metadata from the prompt file.
    ///
    /// Contains any extra fields from the YAML front matter that aren't
    /// part of the core prompt structure. Useful for custom metadata.
    #[serde(flatten)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Specification for a template argument.
///
/// Defines metadata about an argument that a template expects, including
/// whether it's required, default values, and documentation. Used for
/// validation, help generation, and IDE support.
///
/// # Examples
///
/// ```
/// use swissarmyhammer::ArgumentSpec;
///
/// // Required argument with no default
/// let required_arg = ArgumentSpec {
///     name: "filename".to_string(),
///     description: Some("Path to the file to process".to_string()),
///     required: true,
///     default: None,
///     type_hint: Some("path".to_string()),
/// };
///
/// // Optional argument with default value
/// let optional_arg = ArgumentSpec {
///     name: "format".to_string(),
///     description: Some("Output format".to_string()),
///     required: false,
///     default: Some("markdown".to_string()),
///     type_hint: Some("string".to_string()),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArgumentSpec {
    /// The name of the argument as used in templates.
    ///
    /// This is how the argument is referenced in the template (e.g., `{{filename}}`).
    /// Should be a valid identifier using letters, numbers, and underscores.
    pub name: String,

    /// Human-readable description of the argument's purpose.
    ///
    /// Used in help text and documentation generation. Should explain
    /// what the argument is for and any constraints or expected formats.
    pub description: Option<String>,

    /// Whether this argument must be provided.
    ///
    /// If `true`, template rendering will fail if this argument is not provided
    /// and no default value is specified.
    pub required: bool,

    /// Default value to use if the argument is not provided.
    ///
    /// Only used when `required` is `false` or when the user doesn't provide
    /// a value for a required argument. The default is used as-is in the template.
    pub default: Option<String>,

    /// Type hint for the argument.
    ///
    /// Helps tools and users understand what kind of value is expected.
    /// Common values: "string", "number", "boolean", "path", "url", "json".
    /// This is primarily for documentation and tooling support.
    pub type_hint: Option<String>,
}

impl Prompt {
    /// Creates a new prompt with the given name and template.
    ///
    /// This is the minimal constructor for a prompt. Additional metadata can be added
    /// using the builder methods like [`with_description`](Self::with_description),
    /// [`with_category`](Self::with_category), and [`add_argument`](Self::add_argument).
    ///
    /// # Arguments
    ///
    /// * `name` - Unique identifier for the prompt
    /// * `template` - Template content using Liquid syntax
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::Prompt;
    ///
    /// let prompt = Prompt::new("hello", "Hello {{name}}!");
    /// assert_eq!(prompt.name, "hello");
    /// assert_eq!(prompt.template, "Hello {{name}}!");
    /// ```
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

    /// Renders the prompt template with the provided arguments.
    ///
    /// This method validates that all required arguments are provided, applies
    /// default values for missing optional arguments, and renders the template
    /// using the Liquid template engine.
    ///
    /// # Arguments
    ///
    /// * `args` - Map of argument names to values
    ///
    /// # Returns
    ///
    /// The rendered template as a string, or an error if:
    /// - Required arguments are missing
    /// - Template syntax is invalid
    /// - Template rendering fails
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::{Prompt, ArgumentSpec};
    /// use std::collections::HashMap;
    ///
    /// let prompt = Prompt::new("greet", "Hello {{name}}!")
    ///     .add_argument(ArgumentSpec {
    ///         name: "name".to_string(),
    ///         description: None,
    ///         required: true,
    ///         default: None,
    ///         type_hint: None,
    ///     });
    ///
    /// let mut args = HashMap::new();
    /// args.insert("name".to_string(), "Alice".to_string());
    ///
    /// let result = prompt.render(&args).unwrap();
    /// assert_eq!(result, "Hello Alice!");
    /// ```
    pub fn render(&self, args: &HashMap<String, String>) -> Result<String> {
        let template = Template::new(&self.template)?;

        // Validate required arguments
        for arg in &self.arguments {
            if arg.required && !args.contains_key(&arg.name) {
                return Err(SwissArmyHammerError::Template(format!(
                    "Required argument '{}' not provided",
                    arg.name
                )));
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

    /// Adds an argument specification to the prompt.
    ///
    /// Arguments define what inputs the template expects, whether they're required,
    /// and provide documentation for users of the prompt.
    ///
    /// # Arguments
    ///
    /// * `arg` - The argument specification to add
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::{Prompt, ArgumentSpec};
    ///
    /// let prompt = Prompt::new("example", "Processing {{file}}")
    ///     .add_argument(ArgumentSpec {
    ///         name: "file".to_string(),
    ///         description: Some("Path to input file".to_string()),
    ///         required: true,
    ///         default: None,
    ///         type_hint: Some("path".to_string()),
    ///     });
    ///
    /// assert_eq!(prompt.arguments.len(), 1);
    /// assert_eq!(prompt.arguments[0].name, "file");
    /// ```
    pub fn add_argument(mut self, arg: ArgumentSpec) -> Self {
        self.arguments.push(arg);
        self
    }

    /// Sets the description for the prompt.
    ///
    /// The description helps users understand what the prompt does and when to use it.
    /// It appears in help text and prompt listings.
    ///
    /// # Arguments
    ///
    /// * `description` - Human-readable description of the prompt's purpose
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::Prompt;
    ///
    /// let prompt = Prompt::new("debug", "Debug this error: {{error}}")
    ///     .with_description("Helps analyze and debug programming errors");
    ///
    /// assert_eq!(prompt.description, Some("Helps analyze and debug programming errors".to_string()));
    /// ```
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the category for the prompt.
    ///
    /// Categories help organize prompts into logical groups. Common categories
    /// include "development", "writing", "analysis", and "debugging".
    ///
    /// # Arguments
    ///
    /// * `category` - Category name for organizing the prompt
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::Prompt;
    ///
    /// let prompt = Prompt::new("code-review", "Review this code: {{code}}")
    ///     .with_category("development");
    ///
    /// assert_eq!(prompt.category, Some("development".to_string()));
    /// ```
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Sets the tags for the prompt.
    ///
    /// Tags improve searchability by providing keywords that describe the prompt's
    /// functionality and use cases. They're used by the search system to find
    /// relevant prompts.
    ///
    /// # Arguments
    ///
    /// * `tags` - Vector of tag strings
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::Prompt;
    ///
    /// let prompt = Prompt::new("sql-gen", "Generate SQL: {{description}}")
    ///     .with_tags(vec![
    ///         "sql".to_string(),
    ///         "database".to_string(),
    ///         "generation".to_string()
    ///     ]);
    ///
    /// assert_eq!(prompt.tags.len(), 3);
    /// assert!(prompt.tags.contains(&"sql".to_string()));
    /// ```
    pub fn with_tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }
}

/// Manages a collection of prompts with storage and retrieval capabilities.
///
/// The [`PromptLibrary`] is the main interface for working with collections of prompts.
/// It provides methods to load prompts from directories, search through them, and
/// manage them programmatically. The library uses a pluggable storage backend
/// system to support different storage strategies.
///
/// # Examples
///
/// ```no_run
/// use swissarmyhammer::PromptLibrary;
///
/// // Create a new library with default in-memory storage
/// let mut library = PromptLibrary::new();
///
/// // Load prompts from a directory
/// let count = library.add_directory("./prompts").unwrap();
/// println!("Loaded {} prompts", count);
///
/// // Get a specific prompt
/// let prompt = library.get("code-review").unwrap();
///
/// // Search for prompts
/// let debug_prompts = library.search("debug").unwrap();
/// ```
pub struct PromptLibrary {
    storage: Box<dyn crate::StorageBackend>,
}

impl PromptLibrary {
    /// Creates a new prompt library with default in-memory storage.
    ///
    /// The default storage backend stores prompts in memory, which is suitable
    /// for testing and temporary use. For persistent storage, use
    /// [`with_storage`](Self::with_storage) with a file-based backend.
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::PromptLibrary;
    ///
    /// let library = PromptLibrary::new();
    /// // Library is ready to use with in-memory storage
    /// ```
    pub fn new() -> Self {
        Self {
            storage: Box::new(crate::storage::MemoryStorage::new()),
        }
    }

    /// Creates a prompt library with a custom storage backend.
    ///
    /// This allows you to use different storage strategies such as file-based
    /// storage, database storage, or custom implementations.
    ///
    /// # Arguments
    ///
    /// * `storage` - The storage backend to use
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::{PromptLibrary, storage::MemoryStorage};
    ///
    /// let storage = Box::new(MemoryStorage::new());
    /// let library = PromptLibrary::with_storage(storage);
    /// ```
    pub fn with_storage(storage: Box<dyn crate::StorageBackend>) -> Self {
        Self { storage }
    }

    /// Loads all prompts from a directory and adds them to the library.
    ///
    /// Recursively scans the directory for markdown files (`.md` and `.markdown`)
    /// and loads them as prompts. Files should have YAML front matter with prompt
    /// metadata.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the directory containing prompt files
    ///
    /// # Returns
    ///
    /// The number of prompts successfully loaded, or an error if the directory
    /// cannot be read or prompts cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use swissarmyhammer::PromptLibrary;
    ///
    /// let mut library = PromptLibrary::new();
    /// let count = library.add_directory("./prompts").unwrap();
    /// println!("Loaded {} prompts from directory", count);
    /// ```
    pub fn add_directory(&mut self, path: impl AsRef<Path>) -> Result<usize> {
        let loader = PromptLoader::new();
        let prompts = loader.load_directory(path)?;
        let count = prompts.len();

        for prompt in prompts {
            self.storage.store(prompt)?;
        }

        Ok(count)
    }

    /// Retrieves a prompt by its name.
    ///
    /// # Arguments
    ///
    /// * `name` - The unique name of the prompt to retrieve
    ///
    /// # Returns
    ///
    /// The prompt if found, or a [`SwissArmyHammerError::PromptNotFound`] error.
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::{PromptLibrary, Prompt};
    ///
    /// let mut library = PromptLibrary::new();
    ///
    /// // Add a prompt first
    /// let prompt = Prompt::new("test", "Hello {{name}}!");
    /// library.add(prompt).unwrap();
    ///
    /// // Retrieve it
    /// let retrieved = library.get("test").unwrap();
    /// assert_eq!(retrieved.name, "test");
    /// ```
    pub fn get(&self, name: &str) -> Result<Prompt> {
        self.storage.get(name)
    }

    /// Lists all prompts in the library.
    ///
    /// # Returns
    ///
    /// A vector of all prompts currently stored in the library.
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::{PromptLibrary, Prompt};
    ///
    /// let mut library = PromptLibrary::new();
    /// library.add(Prompt::new("test1", "Template 1")).unwrap();
    /// library.add(Prompt::new("test2", "Template 2")).unwrap();
    ///
    /// let prompts = library.list().unwrap();
    /// assert_eq!(prompts.len(), 2);
    /// ```
    pub fn list(&self) -> Result<Vec<Prompt>> {
        self.storage.list()
    }

    /// Searches for prompts matching the given query.
    ///
    /// The search implementation depends on the storage backend. Basic implementations
    /// search through prompt names, descriptions, and content. Advanced backends
    /// may provide full-text search capabilities.
    ///
    /// # Arguments
    ///
    /// * `query` - Search query string
    ///
    /// # Returns
    ///
    /// A vector of prompts matching the search query.
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::{PromptLibrary, Prompt};
    ///
    /// let mut library = PromptLibrary::new();
    /// library.add(Prompt::new("debug-js", "Debug JavaScript code")
    ///     .with_description("Helps debug JavaScript errors")).unwrap();
    /// library.add(Prompt::new("format-py", "Format Python code")).unwrap();
    ///
    /// let results = library.search("debug").unwrap();
    /// assert_eq!(results.len(), 1);
    /// assert_eq!(results[0].name, "debug-js");
    /// ```
    pub fn search(&self, query: &str) -> Result<Vec<Prompt>> {
        self.storage.search(query)
    }

    /// Adds a single prompt to the library.
    ///
    /// If a prompt with the same name already exists, it will be replaced.
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to add
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::{PromptLibrary, Prompt};
    ///
    /// let mut library = PromptLibrary::new();
    /// let prompt = Prompt::new("example", "Example template");
    /// library.add(prompt).unwrap();
    ///
    /// assert!(library.get("example").is_ok());
    /// ```
    pub fn add(&mut self, prompt: Prompt) -> Result<()> {
        self.storage.store(prompt)
    }

    /// Removes a prompt from the library.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the prompt to remove
    ///
    /// # Returns
    ///
    /// Ok(()) if the prompt was removed, or an error if it doesn't exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use swissarmyhammer::{PromptLibrary, Prompt};
    ///
    /// let mut library = PromptLibrary::new();
    /// library.add(Prompt::new("temp", "Temporary prompt")).unwrap();
    ///
    /// library.remove("temp").unwrap();
    /// assert!(library.get("temp").is_err());
    /// ```
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
            return Err(SwissArmyHammerError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Directory not found: {}", path.display()),
            )));
        }

        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let entry_path = entry.path();
            if entry_path.is_file() && self.is_prompt_file(entry_path) {
                if let Ok(prompt) = self.load_file(entry_path) {
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
            .ok_or_else(|| SwissArmyHammerError::Other("Invalid file name".to_string()))?
            .to_string();

        let mut prompt = Prompt::new(name, template);
        prompt.source = Some(path.to_path_buf());

        // Parse metadata
        if let Some(metadata) = metadata {
            if let Some(title) = metadata.get("title").and_then(|v| v.as_str()) {
                prompt.metadata.insert(
                    "title".to_string(),
                    serde_json::Value::String(title.to_string()),
                );
            }
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
                        let name = arg_obj
                            .get("name")
                            .and_then(|v| v.as_str())
                            .unwrap_or_default()
                            .to_string();

                        let arg_spec = ArgumentSpec {
                            name,
                            description: arg_obj
                                .get("description")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            required: arg_obj
                                .get("required")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(false),
                            default: arg_obj
                                .get("default")
                                .and_then(|v| v.as_str())
                                .map(String::from),
                            type_hint: arg_obj
                                .get("type")
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
        let prompt = Prompt::new("test", "Hello {{ name }}!").add_argument(ArgumentSpec {
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

    #[test]
    fn test_prompt_loader_loads_only_valid_prompts() {
        use std::fs;
        use tempfile::TempDir;

        // This test verifies that PromptLoader only successfully loads files 
        // that are valid prompts (with proper YAML front matter)
        let temp_dir = TempDir::new().unwrap();

        // Create some directories with invalid markdown files
        let test_dirs = ["issues", "doc", "examples"];

        for dir_name in &test_dirs {
            let dir_path = temp_dir.path().join(dir_name);
            fs::create_dir_all(&dir_path).unwrap();

            // Create a markdown file without YAML front matter (will be skipped during loading)
            let file_path = dir_path.join("invalid.md");
            fs::write(
                &file_path,
                "# Just a regular markdown file\n\nNo YAML front matter here.",
            )
            .unwrap();
        }

        // Create a valid prompt that SHOULD be loaded
        let valid_prompt = temp_dir.path().join("valid.md");
        let valid_content = r#"---
title: Valid Prompt
description: A valid prompt for testing
arguments:
  - name: topic
    description: The topic
    required: true
---

# Valid Prompt

Discuss {{topic}}.
"#;
        fs::write(&valid_prompt, valid_content).unwrap();

        // Create another valid prompt in a subdirectory
        let sub_dir = temp_dir.path().join("prompts");
        fs::create_dir_all(&sub_dir).unwrap();
        let sub_prompt = sub_dir.join("another.md");
        let sub_content = r#"---
title: Another Prompt
description: Another valid prompt
---

This is another prompt.
"#;
        fs::write(&sub_prompt, sub_content).unwrap();

        let loader = PromptLoader::new();
        let prompts = loader.load_directory(temp_dir.path()).unwrap();

        // Should load all markdown files (5 total: 3 invalid + 2 valid)
        // But only the valid ones will have proper metadata
        assert_eq!(
            prompts.len(),
            5,
            "Should load 5 prompts total, but loaded: {}",
            prompts.len()
        );
        
        // Find the prompts with proper metadata
        let prompts_with_descriptions: Vec<&Prompt> = prompts.iter()
            .filter(|p| p.description.is_some())
            .collect();
        
        assert_eq!(prompts_with_descriptions.len(), 2, "Should have 2 prompts with descriptions");
        
        let prompt_names: Vec<String> = prompts.iter().map(|p| p.name.clone()).collect();
        assert!(prompt_names.contains(&"valid".to_string()));
        assert!(prompt_names.contains(&"another".to_string()));
    }
}
