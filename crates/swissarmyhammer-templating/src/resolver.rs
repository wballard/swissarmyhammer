use crate::prompts::{Prompt, TemplateLibrary, TemplateLoader};
use std::collections::HashMap;
use swissarmyhammer_common::file_loader::{FileSource, VirtualFileSystem};
use swissarmyhammer_common::Result;

// Include shared partials from builtin/_partials/ (shared across prompts, skills, and agents)
include!(concat!(env!("OUT_DIR"), "/builtin_partials.rs"));

/// Handles loading prompts from various sources with proper precedence
pub struct PromptResolver {
    /// Track the source of each prompt by name
    pub prompt_sources: HashMap<String, FileSource>,
    /// Virtual file system for managing prompts
    vfs: VirtualFileSystem,
}

impl PromptResolver {
    /// Create a new PromptResolver
    ///
    /// Configures the VFS to load prompts from:
    /// - `$XDG_DATA_HOME/sah/prompts` (user prompts)
    /// - `{git_root}/.prompts` (local/project prompts)
    pub fn new() -> Self {
        let mut vfs = VirtualFileSystem::new("prompts");
        vfs.use_dot_directory_paths();
        Self {
            prompt_sources: HashMap::new(),
            vfs,
        }
    }

    /// Get all directories that prompts are loaded from
    /// Returns paths in the same order as loading precedence
    pub fn get_prompt_directories(&self) -> Result<Vec<std::path::PathBuf>> {
        Ok(self.vfs.get_directories()?)
    }

    /// Load all prompts following the correct precedence:
    /// 1. User prompts from $XDG_DATA_HOME/sah/prompts
    /// 2. Local prompts from .prompts in the project root (most specific)
    ///
    /// Also loads the shared builtin partials (used by skills and agents) into
    /// the library's storage for template rendering.
    pub fn load_all_prompts(&mut self, library: &mut TemplateLibrary) -> Result<()> {
        // Register the shared builtin partials first (least precedence)
        self.load_builtin_partials();

        // Load all files from directories using VFS
        self.vfs.load_all()?;

        // Process all loaded files into prompts and partials
        let loader = TemplateLoader::new();
        for file in self.vfs.list() {
            // Check if this is a partial template - either by tag or frontmatter
            let has_partial_tag = file.content.trim_start().starts_with("{% partial %}");
            let has_partial_frontmatter = crate::frontmatter::parse_frontmatter(&file.content)
                .ok()
                .and_then(|fm| fm.metadata)
                .and_then(|m| m.get("partial").cloned())
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            let is_partial = has_partial_tag || has_partial_frontmatter;

            if is_partial {
                // For partials, create a minimal prompt object to store in the library
                // Store with multiple name variants so liquid can find them regardless of how they're referenced
                let base_name = file.name.strip_suffix(".liquid").unwrap_or(&file.name);
                let base_name = base_name.strip_suffix(".md").unwrap_or(base_name);

                // Strip frontmatter from partial content so it doesn't bleed into rendered output
                let template_content = match crate::frontmatter::parse_frontmatter(&file.content) {
                    Ok(fm) => fm.content,
                    Err(_) => file.content.clone(),
                };

                // Add the partial with the base name (e.g., "workflow_guards")
                let mut partial_prompt = Prompt::new(base_name, template_content.clone());
                partial_prompt
                    .metadata
                    .insert("partial".to_string(), serde_json::Value::Bool(true));
                library.add(partial_prompt)?;
                self.prompt_sources
                    .insert(base_name.to_string(), file.source.clone());

                // Also add with .md extension (e.g., "workflow_guards.md")
                let name_with_md = format!("{}.md", base_name);
                let mut partial_with_md = Prompt::new(&name_with_md, template_content.clone());
                partial_with_md
                    .metadata
                    .insert("partial".to_string(), serde_json::Value::Bool(true));
                library.add(partial_with_md)?;
                self.prompt_sources
                    .insert(name_with_md, file.source.clone());

                // Also add with .liquid extension (e.g., "workflow_guards.liquid")
                let name_with_liquid = format!("{}.liquid", base_name);
                let mut partial_with_liquid =
                    Prompt::new(&name_with_liquid, template_content.clone());
                partial_with_liquid
                    .metadata
                    .insert("partial".to_string(), serde_json::Value::Bool(true));
                library.add(partial_with_liquid)?;
                self.prompt_sources
                    .insert(name_with_liquid, file.source.clone());
            } else {
                // Load regular prompts normally
                let mut prompt = loader.load_from_string(&file.name, &file.content)?;

                // Set source path for file-based prompts (not builtins)
                if file.source != FileSource::Builtin {
                    prompt.source = Some(file.path.clone());
                }

                // Track the source
                self.prompt_sources
                    .insert(prompt.name.clone(), file.source.clone());

                // Add to library
                library.add(prompt)?;
            }
        }

        Ok(())
    }

    /// Register the shared builtin partials from embedded binary data.
    ///
    /// Adds shared partials from `builtin/_partials/` with the `_partials/`
    /// prefix so `{% include "_partials/..." %}` references (used by skills and
    /// agents) resolve correctly.
    fn load_builtin_partials(&mut self) {
        let builtin_partials = get_builtin_partials();
        for (name, content) in builtin_partials {
            self.vfs.add_builtin(format!("_partials/{}", name), content);
        }
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
    use serial_test::serial;
    use std::fs;
    use swissarmyhammer_common::test_utils::CurrentDirGuard;
    use tempfile::TempDir;

    #[test]
    #[serial]
    fn test_load_all_prompts_registers_builtin_partials() {
        // The shared builtin partials (used by skills and agents) must always be
        // registered after load_all_prompts, even though builtin prompts no longer exist.
        let mut resolver = PromptResolver::new();
        let mut library = TemplateLibrary::new();

        resolver
            .load_all_prompts(&mut library)
            .expect("load_all_prompts should succeed");

        // Shared partials live in builtin/_partials/ and must be present.
        library
            .get("_partials/record-progress")
            .expect("_partials/record-progress should be registered");
    }

    #[test]
    #[serial]
    fn test_prompt_resolver_loads_user_prompts() {
        let temp_dir = TempDir::new().unwrap();
        // VFS dot-directory mode now uses $XDG_DATA_HOME/{xdg_name}/prompts for user-level
        let user_prompts_dir = temp_dir.path().join("sah").join("prompts");
        fs::create_dir_all(&user_prompts_dir).unwrap();

        // Create a test prompt file
        let prompt_file = user_prompts_dir.join("test_prompt.md");
        fs::write(&prompt_file, "This is a test prompt").unwrap();

        let original_xdg = std::env::var("XDG_DATA_HOME").ok();
        std::env::set_var("XDG_DATA_HOME", temp_dir.path());

        let mut resolver = PromptResolver::new();
        let mut library = TemplateLibrary::new();

        let result = resolver.load_all_prompts(&mut library);

        // Restore XDG_DATA_HOME before asserting
        match &original_xdg {
            Some(val) => std::env::set_var("XDG_DATA_HOME", val),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }

        result.unwrap();

        // Check that our test prompt was loaded
        let prompt = library.get("test_prompt").unwrap();
        assert_eq!(prompt.name, "test_prompt");
        assert_eq!(
            resolver.prompt_sources.get("test_prompt"),
            Some(&FileSource::User)
        );
    }

    #[test]
    #[serial]
    fn test_prompt_resolver_loads_local_prompts() {
        let temp_dir = TempDir::new().unwrap();

        // Create a .git directory to make it look like a Git repository
        let git_dir = temp_dir.path().join(".git");
        fs::create_dir_all(&git_dir).unwrap();

        let local_prompts_dir = temp_dir.path().join(".prompts");
        fs::create_dir_all(&local_prompts_dir).unwrap();

        // Create a test prompt file with proper header
        let prompt_file = local_prompts_dir.join("local_prompt.md");
        fs::write(
            &prompt_file,
            "# Local Test Prompt\n\nThis is a local prompt for testing",
        )
        .unwrap();

        let mut resolver = PromptResolver::new();
        let mut library = TemplateLibrary::new();

        // Change to the temp directory to simulate local prompts. The RAII
        // guard restores the original working directory on drop, even on panic.
        {
            let _cwd_guard = CurrentDirGuard::new(temp_dir.path())
                .expect("Failed to pin working directory to the isolated temp dir");

            resolver.load_all_prompts(&mut library).unwrap();
        }

        // Check that our test prompt was loaded
        let prompt = library.get("local_prompt").unwrap();
        assert_eq!(prompt.name, "local_prompt");
        assert_eq!(
            resolver.prompt_sources.get("local_prompt"),
            Some(&FileSource::Local)
        );
    }

    #[test]
    fn test_get_prompt_directories() {
        // Skip isolated test environment setup for now
        let resolver = PromptResolver::new();
        let directories = resolver.get_prompt_directories().unwrap();

        // Should return a vector of PathBuf (may be empty if no directories exist)
        // At minimum, should not panic and should return a valid result
        // Note: Vec::len() is always >= 0, so no need to test this

        // All returned paths should be absolute and existing
        // (The implementation only returns directories that exist)
        for dir in &directories {
            assert!(
                dir.is_absolute(),
                "Directory path should be absolute: {:?}",
                dir
            );
            // Note: Due to test isolation issues, directories may be cleaned up by other tests
            // The get_directories implementation only returns existing directories, so if we get here,
            // the directory existed at query time, but may not exist now due to test cleanup
            if dir.exists() {
                assert!(dir.is_dir(), "Path should be a directory: {:?}", dir);
            } else {
                // Directory was cleaned up between query and assertion - this is acceptable in tests
                println!(
                    "Warning: Directory {:?} was cleaned up during test execution",
                    dir
                );
            }
        }

        // Test that the function doesn't panic even when no directories exist
        // This is implicitly tested since we got here successfully
    }

    #[test]
    #[serial]
    fn test_user_prompt_overrides_builtin_source_tracking() {
        let temp_dir = TempDir::new().unwrap();
        // VFS dot-directory mode now uses $XDG_DATA_HOME/{xdg_name}/prompts for user-level
        let user_prompts_dir = temp_dir.path().join("sah").join("prompts");
        fs::create_dir_all(&user_prompts_dir).unwrap();

        // Create a user prompt with the same name as a builtin prompt
        let prompt_file = user_prompts_dir.join("debug").join("error.md");
        fs::create_dir_all(prompt_file.parent().unwrap()).unwrap();
        let user_prompt_content = r"---
title: User Debug Error
description: User-defined error debugging prompt
---

This is a user-defined debug/error prompt that should override the builtin one.
";
        fs::write(&prompt_file, user_prompt_content).unwrap();

        let mut resolver = PromptResolver::new();
        let mut library = TemplateLibrary::new();

        // Store original XDG_DATA_HOME to restore later
        let original_xdg = std::env::var("XDG_DATA_HOME").ok();

        // Temporarily set XDG_DATA_HOME for test
        std::env::set_var("XDG_DATA_HOME", temp_dir.path());

        // Load builtin prompts first
        resolver.load_all_prompts(&mut library).unwrap();

        // Check if debug/error exists as builtin (it might not always exist)
        let has_builtin_debug_error = resolver.prompt_sources.contains_key("debug/error");

        // Load user prompts (should override the builtin if it exists, or just add it if not)
        resolver.load_all_prompts(&mut library).unwrap();

        // Restore XDG_DATA_HOME before assertions so it's restored even on panic
        match &original_xdg {
            Some(val) => std::env::set_var("XDG_DATA_HOME", val),
            None => std::env::remove_var("XDG_DATA_HOME"),
        }

        // Now it should be tracked as a user prompt
        assert_eq!(
            resolver.prompt_sources.get("debug/error"),
            Some(&FileSource::User),
            "debug/error should be tracked as User prompt after loading user prompts"
        );

        // Verify the prompt content was updated/loaded
        let prompt = library.get("debug/error").unwrap();
        assert!(
            prompt.template.contains("user-defined"),
            "Prompt should contain user-defined content"
        );

        // If we had a builtin debug/error, verify it was actually overridden
        if has_builtin_debug_error {
            assert_eq!(
                resolver.prompt_sources.get("debug/error"),
                Some(&FileSource::User),
                "Builtin debug/error should have been overridden by user prompt"
            );
        }
    }

    #[test]
    fn test_partial_frontmatter_registers_all_name_variants() {
        // Create a partial with frontmatter `partial: true`
        let mut library = TemplateLibrary::new();
        let partial_content = r#"---
partial: true
---
This is partial content"#;

        // Simulate what the resolver does for partials
        let has_partial_frontmatter = crate::frontmatter::parse_frontmatter(partial_content)
            .ok()
            .and_then(|fm| fm.metadata)
            .and_then(|m| m.get("partial").cloned())
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        assert!(
            has_partial_frontmatter,
            "Should detect partial: true in frontmatter"
        );

        // Register with all variants like the resolver does
        let base_name = "test_partial";
        library
            .add(Prompt::new(base_name, partial_content))
            .unwrap();
        library
            .add(Prompt::new(format!("{}.md", base_name), partial_content))
            .unwrap();
        library
            .add(Prompt::new(
                format!("{}.liquid", base_name),
                partial_content,
            ))
            .unwrap();

        // All three variants should be retrievable
        assert!(
            library.get("test_partial").is_ok(),
            "Should find partial without extension"
        );
        assert!(
            library.get("test_partial.md").is_ok(),
            "Should find partial with .md extension"
        );
        assert!(
            library.get("test_partial.liquid").is_ok(),
            "Should find partial with .liquid extension"
        );
    }

    #[test]
    fn test_render_partial_with_and_without_extension() {
        use swissarmyhammer_config::TemplateContext;

        // Create a library with a partial that uses frontmatter
        let mut library = TemplateLibrary::new();

        // Add the partial with all name variants
        let partial_content = "PARTIAL_CONTENT_HERE";
        library
            .add(Prompt::new("_test/my_partial", partial_content))
            .unwrap();
        library
            .add(Prompt::new("_test/my_partial.md", partial_content))
            .unwrap();

        // Add a template that references the partial WITHOUT extension
        let template_without_ext = r#"Before {% render "_test/my_partial" %} After"#;
        library
            .add(Prompt::new("test_without_ext", template_without_ext))
            .unwrap();

        // Add a template that references the partial WITH extension
        let template_with_ext = r#"Before {% render "_test/my_partial.md" %} After"#;
        library
            .add(Prompt::new("test_with_ext", template_with_ext))
            .unwrap();

        let ctx = TemplateContext::new();

        // Both should render successfully
        let result_without = library.render("test_without_ext", &ctx);
        assert!(
            result_without.is_ok(),
            "Should render partial without extension: {:?}",
            result_without.err()
        );
        assert!(
            result_without.unwrap().contains("PARTIAL_CONTENT_HERE"),
            "Rendered content should include partial"
        );

        let result_with = library.render("test_with_ext", &ctx);
        assert!(
            result_with.is_ok(),
            "Should render partial with .md extension: {:?}",
            result_with.err()
        );
        assert!(
            result_with.unwrap().contains("PARTIAL_CONTENT_HERE"),
            "Rendered content should include partial"
        );
    }

    /// Name `get_builtin_partials` serves the Swift project-type guidelines
    /// under. `load_builtin_partials` registers it as `_partials/{name}`, which
    /// is the same path `swissarmyhammer-project-detection` builds for the
    /// `swift` project type through its `partial!` macro.
    const SWIFT_PARTIAL: &str = "project-types/swift";

    /// Return one embedded builtin partial's content by name.
    ///
    /// Reads the same table `load_builtin_partials` serves from, so an
    /// assertion here measures the text an agent actually receives rather than
    /// a file on disk.
    fn builtin_partial(name: &str) -> &'static str {
        get_builtin_partials()
            .into_iter()
            .find(|(partial_name, _)| *partial_name == name)
            .map(|(_, content)| content)
            .unwrap_or_else(|| panic!("builtin partial '{name}' should be embedded"))
    }

    /// Assert the Swift guidelines partial carries each phrase it must keep.
    ///
    /// Each row pairs the exact text the partial has to hold with the
    /// requirement that text satisfies, so a failure names the requirement
    /// rather than only the missing string.
    fn assert_swift_partial_states(requirements: &[(&str, &str)]) {
        let swift = builtin_partial(SWIFT_PARTIAL);
        for (needle, requirement) in requirements {
            assert!(
                swift.contains(needle),
                "the swift guidelines partial must {requirement} (missing {needle:?})"
            );
        }
    }

    /// `swift format` and `swiftformat` are different programs. The first is
    /// Apple's swift-format, which ships with the toolchain and reads
    /// `.swift-format`. The second is Nick Lockwood's SwiftFormat, installed
    /// separately, which reads `.swiftformat`. The guidelines once offered the
    /// two as interchangeable alternatives on one line, which sends an agent to
    /// the wrong config file.
    #[test]
    fn swift_partial_names_each_formatter_with_its_own_config_file() {
        assert_swift_partial_states(&[
            (
                ".swift-format",
                "name the config file Apple's swift-format reads",
            ),
            (".swiftformat", "name the config file SwiftFormat reads"),
            (".swiftlint.yml", "name the config file SwiftLint reads"),
            (
                "DIFFERENT programs",
                "state that the two formatters are not one tool",
            ),
            (
                "Do not write a config file as a side effect",
                "forbid creating a config file while formatting",
            ),
        ]);

        let swift = builtin_partial(SWIFT_PARTIAL);
        assert!(
            !swift.contains("(or `swiftformat .`)"),
            "the swift guidelines partial must never offer `swift format` and \
             `swiftformat` as interchangeable alternatives — they are different \
             tools reading different config files"
        );
    }

    /// A package that depends on `github.com/airbnb/swift` gets one command
    /// running SwiftFormat and SwiftLint together. Both command lines carry
    /// `--allow-writing-to-package-directory`: the plugin always asks for write
    /// permission, so a bare `swift package format --lint` stops with a
    /// permission error in a non-interactive shell.
    #[test]
    fn swift_partial_documents_the_airbnb_plugin_commands() {
        assert_swift_partial_states(&[
            (
                "https://github.com/airbnb/swift",
                "show the dependency that supplies the Airbnb plugin",
            ),
            (
                "swift package --allow-writing-to-package-directory format --lint\n",
                "give the plugin check command, which needs the write permission too",
            ),
            (
                "swift package --allow-writing-to-package-directory format\n",
                "give the plugin non-interactive fix command",
            ),
            (
                "--target <Target>",
                "give the singular --target switch the plugin accepts",
            ),
            ("--paths Sources Tests", "list the plugin paths switch"),
            ("--exclude Tests", "list the plugin exclude switch"),
            (
                "--swift-version 6.2",
                "list the plugin Swift version switch",
            ),
            (
                "Fix mode exits non-zero only for a SwiftLint rule",
                "state which failures each mode reports through its exit code",
            ),
        ]);
    }

    /// Guidance that disagrees with a shipped gate walks an author into a
    /// finding from this project's own review.
    /// `builtin/validators/code-hygiene/rules/idioms-swift.md` runs swiftformat
    /// above a version floor with four options whose defaults all point the
    /// other way, and `builtin/validators/swift/rules/idioms.md` wants the
    /// annotated empty-collection literal that the Airbnb plugin's own
    /// `--property-types inferred` rewrites away.
    #[test]
    fn swift_partial_agrees_with_the_shipped_swift_tool_validators() {
        assert_swift_partial_states(&[
            (
                "--pattern-let inline",
                "pin the pattern-let option the idioms-swift gate measures",
            ),
            (
                "--short-optionals always",
                "pin the short-optionals option the idioms-swift gate measures",
            ),
            (
                "--single-line-for-each convert",
                "pin the single-line-for-each option the idioms-swift gate measures",
            ),
            (
                "--guard-like-if-statements convert",
                "pin the guard-like-if-statements option the idioms-swift gate measures",
            ),
            (
                "0.62.1",
                "state the swiftformat version floor idioms-swift enforces",
            ),
            (
                "--property-types inferred",
                "warn that the plugin rewrites annotated empty collections",
            ),
        ]);
    }
}
