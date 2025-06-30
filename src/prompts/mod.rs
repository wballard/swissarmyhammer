use std::collections::HashMap;
use std::path::Path;
use std::fs;
use walkdir::WalkDir;
use anyhow::{Result, Context};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "var/prompts/"]
struct BuiltinPrompts;

#[derive(Debug, Clone, PartialEq)]
pub struct Prompt {
    pub name: String,
    pub content: String,
    pub source_path: String,
}

impl Prompt {
    pub fn new(name: String, content: String, source_path: String) -> Self {
        Self {
            name,
            content,
            source_path,
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
                        self.prompts.insert(prompt.name.clone(), prompt);
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
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read prompt file: {:?}", path))?;

        // Extract prompt name from file path
        let name = self.extract_prompt_name(path)?;
        let source_path = path.to_string_lossy().to_string();

        Ok(Prompt::new(name, content, source_path))
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
                
                // Extract name from path (remove .md extension)
                let name = file_path
                    .strip_suffix(".md")
                    .unwrap_or(&file_path)
                    .to_string();
                
                let source_path = format!("builtin:/{}", file_path);
                let prompt = Prompt::new(name.clone(), content_str.to_string(), source_path);
                
                self.prompts.insert(name, prompt);
            }
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
}