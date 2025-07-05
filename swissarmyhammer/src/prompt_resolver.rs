use crate::{PromptLibrary, PromptLoader};
use anyhow::Result;
use std::collections::HashMap;

// Include the generated builtin prompts
include!(concat!(env!("OUT_DIR"), "/builtin_prompts.rs"));

/// Source of a prompt (builtin, user, local, or dynamic)
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub enum PromptSource {
    /// Builtin prompts embedded in the binary
    Builtin,
    /// User prompts from ~/.swissarmyhammer/prompts
    User,
    /// Local prompts from .swissarmyhammer/prompts directories
    Local,
    /// Dynamically generated prompts
    Dynamic,
}

impl std::fmt::Display for PromptSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PromptSource::Builtin => write!(f, "builtin"),
            PromptSource::User => write!(f, "user"),
            PromptSource::Local => write!(f, "local"),
            PromptSource::Dynamic => write!(f, "dynamic"),
        }
    }
}

/// Handles loading prompts from various sources with proper precedence
pub struct PromptResolver {
    /// Track the source of each prompt by name
    pub prompt_sources: HashMap<String, PromptSource>,
}

impl PromptResolver {
    /// Create a new PromptResolver
    pub fn new() -> Self {
        Self {
            prompt_sources: HashMap::new(),
        }
    }

    /// Load all prompts following the correct precedence:
    /// 1. Builtin prompts (least specific, embedded in binary)
    /// 2. User prompts from ~/.swissarmyhammer/prompts
    /// 3. Local prompts from .swissarmyhammer directories (most specific)
    pub fn load_all_prompts(&mut self, library: &mut PromptLibrary) -> Result<()> {
        // Load builtin prompts first (least precedence)
        self.load_builtin_prompts(library)?;

        // Load user prompts from home directory
        self.load_user_prompts(library)?;

        // Load local prompts recursively (highest precedence)
        self.load_local_prompts(library)?;

        Ok(())
    }

    /// Load builtin prompts from embedded binary data
    pub fn load_builtin_prompts(&mut self, library: &mut PromptLibrary) -> Result<()> {
        let builtin_prompts = get_builtin_prompts();
        let loader = PromptLoader::new();

        // Add each embedded prompt to the library
        for (name, content) in builtin_prompts {
            let prompt = if content.starts_with("---\n") {
                // Parse as a prompt file with frontmatter
                loader.load_from_string(name, content)?
            } else {
                // Treat as a simple template
                crate::prompts::Prompt::new(name, content)
            };

            // Track the prompt source using the actual prompt name
            self.prompt_sources
                .insert(prompt.name.clone(), PromptSource::Builtin);
            library.add(prompt)?;
        }

        Ok(())
    }

    /// Load user prompts from ~/.swissarmyhammer/prompts
    pub fn load_user_prompts(&mut self, library: &mut PromptLibrary) -> Result<()> {
        if let Some(home) = dirs::home_dir() {
            let user_prompts_dir = home.join(".swissarmyhammer").join("prompts");
            if user_prompts_dir.exists() {
                // Load user prompts from the directory
                let loader = crate::prompts::PromptLoader::new();
                let user_prompts = loader.load_directory(&user_prompts_dir)?;
                
                // Add each user prompt and track it
                for prompt in user_prompts {
                    // User prompts override any existing prompt with the same name
                    self.prompt_sources.insert(prompt.name.clone(), PromptSource::User);
                    library.add(prompt)?;
                }
            }
        }
        Ok(())
    }

    /// Load local prompts by recursively searching up for .swissarmyhammer directories
    fn load_local_prompts(&mut self, library: &mut PromptLibrary) -> Result<()> {
        let current_dir = std::env::current_dir()?;

        // Find all .swissarmyhammer directories from root to current
        let mut prompt_dirs = Vec::new();
        let mut path = current_dir.as_path();

        loop {
            let swissarmyhammer_dir = path.join(".swissarmyhammer");
            if swissarmyhammer_dir.exists() && swissarmyhammer_dir.is_dir() {
                // Skip the user's home .swissarmyhammer directory to avoid duplicate loading
                // Get the user's home directory dynamically to handle test cases
                if let Some(home) = dirs::home_dir() {
                    let user_swissarmyhammer_dir = home.join(".swissarmyhammer");
                    if swissarmyhammer_dir == user_swissarmyhammer_dir {
                        match path.parent() {
                            Some(parent) => path = parent,
                            None => break,
                        }
                        continue;
                    }
                }

                let prompts_dir = swissarmyhammer_dir.join("prompts");
                if prompts_dir.exists() && prompts_dir.is_dir() {
                    prompt_dirs.push(prompts_dir);
                }
            }

            match path.parent() {
                Some(parent) => path = parent,
                None => break,
            }
        }

        // Load in reverse order (root to current) so deeper paths override
        for prompts_dir in prompt_dirs.into_iter().rev() {
            // Load local prompts from the directory
            let loader = crate::prompts::PromptLoader::new();
            let local_prompts = loader.load_directory(&prompts_dir)?;
            
            // Add each local prompt and track it
            for prompt in local_prompts {
                // Local prompts override any existing prompt with the same name
                self.prompt_sources.insert(prompt.name.clone(), PromptSource::Local);
                library.add(prompt)?;
            }
        }

        Ok(())
    }
}

impl Default for PromptResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_prompt_resolver_loads_user_prompts() {
        let temp_dir = TempDir::new().unwrap();
        let user_prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
        fs::create_dir_all(&user_prompts_dir).unwrap();

        // Create a test prompt file
        let prompt_file = user_prompts_dir.join("test_prompt.md");
        fs::write(&prompt_file, "This is a test prompt").unwrap();

        let mut resolver = PromptResolver::new();
        let mut library = PromptLibrary::new();

        // Temporarily change home directory for test
        std::env::set_var("HOME", temp_dir.path());

        resolver.load_user_prompts(&mut library).unwrap();

        let prompts = library.list().unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "test_prompt");
        assert_eq!(
            resolver.prompt_sources.get("test_prompt"),
            Some(&PromptSource::User)
        );
    }

    #[test]
    fn test_prompt_resolver_loads_local_prompts() {
        let temp_dir = TempDir::new().unwrap();
        let local_prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
        fs::create_dir_all(&local_prompts_dir).unwrap();

        // Create a test prompt file
        let prompt_file = local_prompts_dir.join("local_prompt.md");
        fs::write(&prompt_file, "This is a local prompt").unwrap();

        let mut resolver = PromptResolver::new();
        let mut library = PromptLibrary::new();

        // Change to the temp directory to simulate local prompts
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        resolver.load_local_prompts(&mut library).unwrap();

        // Restore original directory
        std::env::set_current_dir(original_dir).unwrap();

        let prompts = library.list().unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0].name, "local_prompt");
        assert_eq!(
            resolver.prompt_sources.get("local_prompt"),
            Some(&PromptSource::Local)
        );
    }

    #[test]
    fn test_debug_error_prompt_is_correctly_tracked_as_builtin() {
        let mut resolver = PromptResolver::new();
        let mut library = PromptLibrary::new();

        // Load builtin prompts
        resolver.load_builtin_prompts(&mut library).unwrap();

        // The debug/error prompt should be loaded and tracked as builtin
        // First check that it exists in the library
        let prompts = library.list().unwrap();
        let debug_error_prompt = prompts.iter().find(|p| p.name == "debug/error");
        
        if let Some(_prompt) = debug_error_prompt {
            // Check that it's tracked as a builtin
            assert_eq!(
                resolver.prompt_sources.get("debug/error"),
                Some(&PromptSource::Builtin),
                "debug/error prompt should be tracked as Builtin, but was tracked as: {:?}",
                resolver.prompt_sources.get("debug/error")
            );
        } else {
            // If debug/error doesn't exist, check if debug-error exists instead
            let debug_hyphen_error_prompt = prompts.iter().find(|p| p.name == "debug-error");
            if let Some(_prompt) = debug_hyphen_error_prompt {
                // This would indicate the bug where frontmatter name overrides build script name
                panic!("Found prompt named 'debug-error' instead of 'debug/error'. This indicates the frontmatter is overriding the build script name.");
            } else {
                // Check what builtin prompts actually exist
                let builtin_prompt_names: Vec<String> = prompts.iter().map(|p| p.name.clone()).collect();
                panic!("debug/error prompt not found. Available builtin prompts: {:?}", builtin_prompt_names);
            }
        }
    }

    #[test]
    fn test_user_prompt_overrides_builtin_source_tracking() {
        let temp_dir = TempDir::new().unwrap();
        let user_prompts_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
        fs::create_dir_all(&user_prompts_dir).unwrap();

        // Create a user prompt with the same name as a builtin prompt
        let prompt_file = user_prompts_dir.join("debug").join("error.md");
        fs::create_dir_all(prompt_file.parent().unwrap()).unwrap();
        let user_prompt_content = r#"---
title: User Debug Error
description: User-defined error debugging prompt
---

This is a user-defined debug/error prompt that should override the builtin one.
"#;
        fs::write(&prompt_file, user_prompt_content).unwrap();

        let mut resolver = PromptResolver::new();
        let mut library = PromptLibrary::new();

        // Temporarily change home directory for test
        std::env::set_var("HOME", temp_dir.path());

        // Load builtin prompts first
        resolver.load_builtin_prompts(&mut library).unwrap();
        
        // Verify it's initially tracked as builtin
        assert_eq!(
            resolver.prompt_sources.get("debug/error"),
            Some(&PromptSource::Builtin)
        );

        // Load user prompts (should override the builtin)
        resolver.load_user_prompts(&mut library).unwrap();

        // Now it should be tracked as a user prompt
        assert_eq!(
            resolver.prompt_sources.get("debug/error"),
            Some(&PromptSource::User),
            "debug/error should now be tracked as User prompt after being overridden"
        );

        // Verify the prompt content was updated
        let prompt = library.get("debug/error").unwrap();
        assert!(prompt.template.contains("user-defined"), "Prompt should contain user-defined content");
    }
}