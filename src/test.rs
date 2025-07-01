use anyhow::{anyhow, Result};
use colored::*;
use dialoguer::{Input, theme::ColorfulTheme};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use serde_json::Value;

use crate::cli::Commands;
use crate::prompts::{PromptLoader, PromptStorage, Prompt};

pub struct TestRunner {
    storage: PromptStorage,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            storage: PromptStorage::new(),
        }
    }

    pub async fn run(&mut self, command: &Commands) -> Result<i32> {
        if let Commands::Test { 
            prompt_name, 
            file, 
            arguments, 
            raw, 
            copy, 
            save, 
            debug 
        } = command {
            // Load all prompts first
            let mut loader = PromptLoader::new();
            loader.load_all()?;
            self.storage = loader.storage;

            // Get the prompt to test
            let prompt = self.get_prompt(prompt_name.as_deref(), file.as_deref())?;
            
            // Collect arguments
            let args = if arguments.is_empty() {
                // Interactive mode - but only if we're in a terminal
                if atty::is(atty::Stream::Stdin) {
                    self.collect_arguments_interactive(&prompt)?
                } else {
                    // Non-interactive mode when not in terminal (CI/testing)
                    self.collect_arguments_non_interactive(&prompt)?
                }
            } else {
                // Non-interactive mode
                self.parse_arguments(arguments)?
            };

            // Show debug information if requested
            if *debug {
                self.show_debug_info(&prompt, &args)?;
            }

            // Render the prompt with environment variables support
            let rendered = self.render_prompt_with_env(&prompt, &args)?;

            // Output the result
            self.output_result(&rendered, *raw, *copy, save.as_deref())?;

            Ok(0)
        } else {
            Err(anyhow!("Invalid command type"))
        }
    }

    fn get_prompt(&self, prompt_name: Option<&str>, file_path: Option<&str>) -> Result<Prompt> {
        match (prompt_name, file_path) {
            (Some(name), None) => {
                // Test by name
                self.storage.get(name)
                    .ok_or_else(|| anyhow!("Prompt '{}' not found", name))
            }
            (None, Some(path)) => {
                // Test from file
                let loader = PromptLoader::new();
                loader.load_prompt_from_file(Path::new(path))
            }
            (Some(_), Some(_)) => {
                Err(anyhow!("Cannot specify both prompt name and file path"))
            }
            (None, None) => {
                Err(anyhow!("Must specify either prompt name or file path"))
            }
        }
    }

    fn parse_arguments(&self, arguments: &[String]) -> Result<HashMap<String, Value>> {
        let mut args = HashMap::new();
        
        for arg in arguments {
            if let Some((key, value)) = arg.split_once('=') {
                args.insert(key.to_string(), Value::String(value.to_string()));
            } else {
                return Err(anyhow!("Invalid argument format: '{}'. Use key=value format", arg));
            }
        }
        
        Ok(args)
    }

    fn collect_arguments_interactive(&self, prompt: &Prompt) -> Result<HashMap<String, Value>> {
        let mut args = HashMap::new();
        let theme = ColorfulTheme::default();

        if prompt.arguments.is_empty() {
            println!("{}", "ℹ No arguments required for this prompt".blue());
            return Ok(args);
        }

        println!("{}", "📝 Please provide values for the following arguments:".bold().blue());
        println!();

        for arg in &prompt.arguments {
            let prompt_text = if arg.required {
                format!("{} (required): {}", arg.name.bold(), arg.description)
            } else {
                format!("{} (optional): {}", arg.name.bold(), arg.description)
            };

            loop {
                let mut input = Input::<String>::with_theme(&theme)
                    .with_prompt(&prompt_text);

                if let Some(default) = &arg.default {
                    input = input.default(default.clone()).show_default(true);
                }

                match input.interact_text() {
                    Ok(value) => {
                        if value.is_empty() && arg.required && arg.default.is_none() {
                            println!("{}", "❌ This argument is required".red());
                            continue;
                        }
                        
                        if !value.is_empty() {
                            args.insert(arg.name.clone(), Value::String(value));
                        }
                        break;
                    }
                    Err(e) => {
                        return Err(anyhow!("Failed to read input: {}", e));
                    }
                }
            }
        }

        println!();
        Ok(args)
    }

    fn collect_arguments_non_interactive(&self, prompt: &Prompt) -> Result<HashMap<String, Value>> {
        let mut args = HashMap::new();

        if prompt.arguments.is_empty() {
            return Ok(args);
        }

        // In non-interactive mode, only use default values for optional arguments
        // Required arguments without defaults will cause template to show undefined variable placeholders
        for arg in &prompt.arguments {
            if let Some(default) = &arg.default {
                args.insert(arg.name.clone(), Value::String(default.clone()));
            }
        }

        Ok(args)
    }

    fn show_debug_info(&self, prompt: &Prompt, args: &HashMap<String, Value>) -> Result<()> {
        println!("{}", "🔍 Debug Information".bold().yellow());
        println!("{}", "─".repeat(50));
        
        println!("{}", "📄 Prompt Details:".bold());
        println!("  Name: {}", prompt.name);
        if let Some(title) = &prompt.title {
            println!("  Title: {}", title);
        }
        if let Some(description) = &prompt.description {
            println!("  Description: {}", description);
        }
        println!("  Source: {}", prompt.source_path);
        println!();

        println!("{}", "📋 Template Content:".bold());
        for (i, line) in prompt.content.lines().enumerate() {
            println!("  {:3}: {}", i + 1, line.dimmed());
        }
        println!();

        println!("{}", "🔧 Arguments Provided:".bold());
        if args.is_empty() {
            println!("  {}", "None".dimmed());
        } else {
            for (key, value) in args {
                println!("  {} = {}", key.cyan(), value.as_str().unwrap_or("").green());
            }
        }
        println!();

        println!("{}", "⚙️ Template Processing:".bold());
        println!("  Engine: Liquid");
        println!("  Backward Compatibility: Enabled");
        println!();

        Ok(())
    }

    fn output_result(&self, rendered: &str, raw: bool, copy: bool, save_path: Option<&str>) -> Result<()> {
        if !raw {
            println!("{}", "✨ Rendered Prompt:".bold().green());
            println!("{}", "─".repeat(50));
        }

        println!("{}", rendered);

        if !raw {
            println!("{}", "─".repeat(50));
        }

        // Copy to clipboard if requested
        if copy {
            self.copy_to_clipboard(rendered)?;
            println!("{}", "📋 Copied to clipboard!".green());
        }

        // Save to file if requested
        if let Some(path) = save_path {
            fs::write(path, rendered)?;
            println!("{}", format!("💾 Saved to {}", path).green());
        }

        Ok(())
    }

    fn render_prompt_with_env(&self, prompt: &Prompt, args: &HashMap<String, Value>) -> Result<String> {
        use crate::template::LiquidEngine;
        
        let engine = LiquidEngine::new();
        
        // Convert our PromptArgument to template::TemplateArgument
        let template_args: Vec<crate::template::TemplateArgument> = prompt.arguments.iter()
            .map(|arg| crate::template::TemplateArgument {
                name: arg.name.clone(),
                description: Some(arg.description.clone()),
                required: arg.required,
                default_value: arg.default.clone(),
            })
            .collect();

        // Create a context that includes environment variables
        let mut context = args.clone();
        
        // Add all environment variables to the context
        let mut env_vars = std::collections::HashMap::new();
        for (key, value) in std::env::vars() {
            env_vars.insert(key, Value::String(value));
        }
        
        if !env_vars.is_empty() {
            context.insert("env".to_string(), Value::Object(env_vars.into_iter().collect()));
        }
        
        // In non-interactive mode, use basic processing (no validation) to allow backward compatibility
        // Only validate when we're in interactive mode or have explicit arguments
        if atty::is(atty::Stream::Stdin) || !args.is_empty() {
            engine.process_with_validation(&prompt.content, &context, &template_args)
        } else {
            engine.process(&prompt.content, &context)
        }
    }

    fn copy_to_clipboard(&self, content: &str) -> Result<()> {
        use arboard::Clipboard;
        
        let mut clipboard = Clipboard::new()
            .map_err(|e| anyhow!("Failed to access clipboard: {}", e))?;
        
        clipboard.set_text(content)
            .map_err(|e| anyhow!("Failed to copy to clipboard: {}", e))?;
        
        Ok(())
    }
}

impl Default for TestRunner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_creation() {
        let runner = TestRunner::new();
        assert!(runner.storage.is_empty());
    }

    #[test]
    fn test_parse_arguments() {
        let runner = TestRunner::new();
        let args = vec![
            "key1=value1".to_string(),
            "key2=value2".to_string(),
        ];
        
        let result = runner.parse_arguments(&args).unwrap();
        assert_eq!(result.get("key1"), Some(&Value::String("value1".to_string())));
        assert_eq!(result.get("key2"), Some(&Value::String("value2".to_string())));
    }

    #[test]
    fn test_parse_arguments_invalid_format() {
        let runner = TestRunner::new();
        let args = vec!["invalid".to_string()];
        
        let result = runner.parse_arguments(&args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid argument format"));
    }

    #[test]
    fn test_get_prompt_validation() {
        let runner = TestRunner::new();
        
        // Both name and file should error
        let result = runner.get_prompt(Some("test"), Some("test.md"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Cannot specify both"));
        
        // Neither name nor file should error
        let result = runner.get_prompt(None, None);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Must specify either"));
    }

    #[tokio::test]
    async fn test_run_with_invalid_command() {
        let mut runner = TestRunner::new();
        let result = runner.run(&Commands::Serve).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid command type"));
    }
}