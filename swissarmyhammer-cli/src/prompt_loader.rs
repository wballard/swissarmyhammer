use anyhow::Result;
use std::path::PathBuf;
use swissarmyhammer::PromptLibrary;

/// Handles loading prompts from various sources with proper precedence
pub struct PromptResolver;

impl PromptResolver {
    pub fn new() -> Self {
        Self
    }

    /// Load all prompts following the correct precedence:
    /// 1. Builtin prompts (least specific, embedded in binary)
    /// 2. User prompts from ~/.swissarmyhammer/prompts
    /// 3. Local prompts from .swissarmyhammer directories (most specific)
    pub fn load_all_prompts(&self, library: &mut PromptLibrary) -> Result<()> {
        // Load builtin prompts first (least precedence)
        self.load_builtin_prompts(library)?;
        
        // Load user prompts from home directory
        self.load_user_prompts(library)?;
        
        // Load local prompts recursively (highest precedence)
        self.load_local_prompts(library)?;
        
        Ok(())
    }

    /// Load builtin prompts (embedded in binary)
    fn load_builtin_prompts(&self, library: &mut PromptLibrary) -> Result<()> {
        // For now, load from the prompts/builtin directory
        // TODO: In the future, these should be embedded in the binary using include_str!
        let builtin_path = PathBuf::from("prompts/builtin");
        if builtin_path.exists() {
            library.add_directory(&builtin_path)?;
        }
        Ok(())
    }

    /// Load user prompts from ~/.swissarmyhammer/prompts
    fn load_user_prompts(&self, library: &mut PromptLibrary) -> Result<()> {
        if let Some(home) = dirs::home_dir() {
            let user_prompts_dir = home.join(".swissarmyhammer").join("prompts");
            if user_prompts_dir.exists() {
                library.add_directory(&user_prompts_dir)?;
            }
        }
        Ok(())
    }

    /// Load local prompts by recursively searching up for .swissarmyhammer directories
    fn load_local_prompts(&self, library: &mut PromptLibrary) -> Result<()> {
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
            library.add_directory(&prompts_dir)?;
        }
        
        Ok(())
    }

    /// Get all prompt directories that would be searched
    #[allow(dead_code)]
    pub fn get_prompt_directories() -> Vec<PathBuf> {
        let mut dirs = Vec::new();
        
        // Builtin directory
        dirs.push(PathBuf::from("prompts/builtin"));
        
        // User directory
        if let Some(home) = dirs::home_dir() {
            dirs.push(home.join(".swissarmyhammer").join("prompts"));
        }
        
        // Local directories (recursive up)
        if let Ok(current_dir) = std::env::current_dir() {
            let mut path = current_dir.as_path();
            let mut local_dirs = Vec::new();
            
            loop {
                let swissarmyhammer_dir = path.join(".swissarmyhammer").join("prompts");
                if swissarmyhammer_dir.exists() {
                    local_dirs.push(swissarmyhammer_dir);
                }
                
                match path.parent() {
                    Some(parent) => path = parent,
                    None => break,
                }
            }
            
            // Add in order from root to current
            dirs.extend(local_dirs.into_iter().rev());
        }
        
        dirs
    }
}

impl Default for PromptResolver {
    fn default() -> Self {
        Self::new()
    }
}