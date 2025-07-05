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

            self.prompt_sources
                .insert(name.to_string(), PromptSource::Builtin);
            library.add(prompt)?;
        }

        Ok(())
    }

    /// Load user prompts from ~/.swissarmyhammer/prompts
    pub fn load_user_prompts(&mut self, library: &mut PromptLibrary) -> Result<()> {
        if let Some(home) = dirs::home_dir() {
            let user_prompts_dir = home.join(".swissarmyhammer").join("prompts");
            if user_prompts_dir.exists() {
                // Get the count before and after to track new prompts
                let before_count = library.list()?.len();
                library.add_directory(&user_prompts_dir)?;
                let after_count = library.list()?.len();

                // Mark all newly added prompts as user prompts
                let prompts = library.list()?;
                for i in before_count..after_count {
                    if let Some(prompt) = prompts.get(i) {
                        self.prompt_sources
                            .insert(prompt.name.clone(), PromptSource::User);
                    }
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
            let before_count = library.list()?.len();
            library.add_directory(&prompts_dir)?;
            let after_count = library.list()?.len();

            // Mark all newly added prompts as local prompts
            let prompts = library.list()?;
            for i in before_count..after_count {
                if let Some(prompt) = prompts.get(i) {
                    self.prompt_sources
                        .insert(prompt.name.clone(), PromptSource::Local);
                }
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
}
