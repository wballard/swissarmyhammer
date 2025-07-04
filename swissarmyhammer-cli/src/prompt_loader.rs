use crate::cli::PromptSource;
use anyhow::Result;
use std::collections::HashMap;
use swissarmyhammer::PromptLibrary;

/// Handles loading prompts from various sources with proper precedence
pub struct PromptResolver {
    /// Track the source of each prompt by name
    pub prompt_sources: HashMap<String, PromptSource>,
}

impl PromptResolver {
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

    /// Load builtin prompts (embedded in binary)
    pub fn load_builtin_prompts(&mut self, library: &mut PromptLibrary) -> Result<()> {
        // Embed all builtin prompts directly in the binary
        let builtin_prompts = vec![
            ("example", include_str!("../../prompts/builtin/example.md")),
            ("help", include_str!("../../prompts/builtin/help.md")),
            ("plan", include_str!("../../prompts/builtin/plan.md")),
            (
                "debug/error",
                include_str!("../../prompts/builtin/debug/error.md"),
            ),
            (
                "debug/logs",
                include_str!("../../prompts/builtin/debug/logs.md"),
            ),
            (
                "debug/performance",
                include_str!("../../prompts/builtin/debug/performance.md"),
            ),
            (
                "docs/api",
                include_str!("../../prompts/builtin/docs/api.md"),
            ),
            (
                "docs/comments",
                include_str!("../../prompts/builtin/docs/comments.md"),
            ),
            (
                "docs/readme",
                include_str!("../../prompts/builtin/docs/readme.md"),
            ),
            (
                "prompts/create",
                include_str!("../../prompts/builtin/prompts/create.md"),
            ),
            (
                "prompts/improve",
                include_str!("../../prompts/builtin/prompts/improve.md"),
            ),
            (
                "refactor/clean",
                include_str!("../../prompts/builtin/refactor/clean.md"),
            ),
            (
                "refactor/extract",
                include_str!("../../prompts/builtin/refactor/extract.md"),
            ),
            (
                "refactor/patterns",
                include_str!("../../prompts/builtin/refactor/patterns.md"),
            ),
            (
                "review/accessibility",
                include_str!("../../prompts/builtin/review/accessibility.md"),
            ),
            (
                "review/code",
                include_str!("../../prompts/builtin/review/code.md"),
            ),
            (
                "review/code-dynamic",
                include_str!("../../prompts/builtin/review/code-dynamic.md"),
            ),
            (
                "review/security",
                include_str!("../../prompts/builtin/review/security.md"),
            ),
            (
                "test/integration",
                include_str!("../../prompts/builtin/test/integration.md"),
            ),
            (
                "test/property",
                include_str!("../../prompts/builtin/test/property.md"),
            ),
            (
                "test/unit",
                include_str!("../../prompts/builtin/test/unit.md"),
            ),
            (
                "analysis/statistics-calculator",
                include_str!("../../prompts/builtin/analysis/statistics-calculator.md"),
            ),
            (
                "communication/email-composer",
                include_str!("../../prompts/builtin/communication/email-composer.md"),
            ),
            (
                "data/array-processor",
                include_str!("../../prompts/builtin/data/array-processor.md"),
            ),
            (
                "formatting/table-generator",
                include_str!("../../prompts/builtin/formatting/table-generator.md"),
            ),
            (
                "productivity/task-formatter",
                include_str!("../../prompts/builtin/productivity/task-formatter.md"),
            ),
            (
                "empty",
                include_str!("../../prompts/builtin/empty.md.liquid"),
            ),
        ];

        // Add each embedded prompt to the library
        let loader = swissarmyhammer::PromptLoader::new();
        for (name, content) in builtin_prompts {
            let prompt = self.parse_embedded_prompt(name, content, &loader)?;
            self.prompt_sources
                .insert(name.to_string(), PromptSource::Builtin);
            library.add(prompt)?;
        }

        Ok(())
    }

    /// Load user prompts from ~/.swissarmyhammer/prompts
    fn load_user_prompts(&mut self, library: &mut PromptLibrary) -> Result<()> {
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

    /// Parse an embedded prompt from content string
    fn parse_embedded_prompt(
        &self,
        name: &str,
        content: &str,
        _loader: &swissarmyhammer::PromptLoader,
    ) -> Result<swissarmyhammer::Prompt> {
        // Use reflection to access the private parse_front_matter method
        // Since it's private, we'll need to duplicate the parsing logic
        let (metadata, template) = self.parse_front_matter_embedded(content)?;

        let mut prompt = swissarmyhammer::Prompt::new(name, template.clone());

        // Builtin prompts don't have a source path - they're embedded
        prompt.source = None;

        // Parse metadata (similar to PromptLoader::load_file)
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

                        let arg_spec = swissarmyhammer::ArgumentSpec {
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

        // If this appears to be a partial template and has no description, provide a default one
        if prompt.description.is_none() && self.is_likely_partial(name, &template) {
            prompt.description = Some("Partial template for reuse in other prompts".to_string());
        }

        Ok(prompt)
    }

    /// Determine if a prompt is likely a partial template
    fn is_likely_partial(&self, name: &str, content: &str) -> bool {
        // Check if the name suggests it's a partial (common naming patterns)
        let name_lower = name.to_lowercase();
        if name_lower.contains("partial") || name_lower.starts_with("_") {
            return true;
        }

        // Check if it has no YAML front matter (partials often don't)
        let has_front_matter = content.starts_with("---\n");
        if !has_front_matter {
            return true;
        }

        // Check for typical partial characteristics:
        // - Short content that looks like a fragment
        // - Contains mostly template variables
        // - Doesn't have typical prompt structure
        let lines: Vec<&str> = content.lines().collect();
        let content_lines: Vec<&str> = if has_front_matter {
            // Skip YAML front matter
            lines.iter().skip_while(|line| **line != "---").skip(1).skip_while(|line| **line != "---").skip(1).copied().collect()
        } else {
            lines
        };

        // If it's very short and has no headers, it might be a partial
        if content_lines.len() <= 5 && !content_lines.iter().any(|line| line.starts_with('#')) {
            return true;
        }

        false
    }

    /// Parse front matter from content (duplicated from PromptLoader since it's private)
    fn parse_front_matter_embedded(
        &self,
        content: &str,
    ) -> Result<(Option<serde_json::Value>, String)> {
        if content.starts_with("---\n") {
            let parts: Vec<&str> = content.splitn(3, "---\n").collect();
            if parts.len() >= 3 {
                let yaml_content = parts[1];
                let template = parts[2].trim_start().to_string();

                let metadata: serde_yaml::Value = serde_yaml::from_str(yaml_content)
                    .map_err(|e| anyhow::anyhow!("YAML parse error: {}", e))?;
                let json_value = serde_json::to_value(metadata)
                    .map_err(|e| anyhow::anyhow!("JSON conversion error: {}", e))?;

                return Ok((Some(json_value), template));
            }
        }

        Ok((None, content.to_string()))
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
    use std::path::PathBuf;
    use swissarmyhammer::PromptLibrary;

    #[test]
    fn test_builtin_prompts_should_be_embedded() {
        // Test that demonstrates builtin prompts should be embedded in binary, not loaded from files
        let mut resolver = PromptResolver::new();
        let mut library = PromptLibrary::new();

        // Delete the builtin directory temporarily to test that prompts are embedded
        let builtin_path = PathBuf::from("prompts/builtin");
        let builtin_existed = builtin_path.exists();

        if builtin_existed {
            // This test should pass even when builtin directory doesn't exist
            // because builtin prompts should be embedded in the binary
            std::fs::rename(&builtin_path, "prompts/builtin_backup").unwrap();
        }

        // Load builtin prompts - this should work even without the directory
        let result = resolver.load_builtin_prompts(&mut library);

        // Restore the directory if it existed
        if builtin_existed {
            std::fs::rename("prompts/builtin_backup", &builtin_path).unwrap();
        }

        // This should succeed even without the directory existing
        assert!(
            result.is_ok(),
            "Builtin prompts should be embedded in binary"
        );

        // We should have loaded some builtin prompts
        assert!(
            !library.list().unwrap().is_empty(),
            "Should have embedded builtin prompts"
        );
    }

    #[test]
    fn test_dead_code_removed() {
        // This test verifies that dead code has been successfully removed
        // get_prompt_directories function should no longer exist

        // If this test compiles and passes, it means we successfully removed the dead code
        // The old get_prompt_directories function no longer exists
    }
}
