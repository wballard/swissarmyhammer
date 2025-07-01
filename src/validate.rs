use anyhow::Result;
use colored::*;
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::fs;

use crate::cli::ValidateFormat;
use crate::prompts::{PromptLoader, PromptStorage};
use crate::template::LiquidEngine;

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationLevel {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub level: ValidationLevel,
    pub file_path: PathBuf,
    pub line: Option<usize>,
    pub column: Option<usize>,
    pub message: String,
    pub suggestion: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub issues: Vec<ValidationIssue>,
    pub files_checked: usize,
    pub errors: usize,
    pub warnings: usize,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidationResult {
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            files_checked: 0,
            errors: 0,
            warnings: 0,
        }
    }

    pub fn add_issue(&mut self, issue: ValidationIssue) {
        match issue.level {
            ValidationLevel::Error => self.errors += 1,
            ValidationLevel::Warning => self.warnings += 1,
            ValidationLevel::Info => {}
        }
        self.issues.push(issue);
    }

    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    pub fn has_warnings(&self) -> bool {
        self.warnings > 0
    }
}

#[derive(Debug, Serialize)]
struct JsonValidationResult {
    files_checked: usize,
    errors: usize,
    warnings: usize,
    issues: Vec<JsonValidationIssue>,
}

#[derive(Debug, Serialize)]
struct JsonValidationIssue {
    level: String,
    file_path: String,
    line: Option<usize>,
    column: Option<usize>,
    message: String,
    suggestion: Option<String>,
}

pub struct Validator {
    quiet: bool,
}

impl Validator {
    pub fn new(quiet: bool) -> Self {
        Self { quiet }
    }

    pub fn validate_all(&self) -> Result<ValidationResult> {
        let mut result = ValidationResult::new();

        // Load all prompts from all sources
        let storage = PromptStorage::new();
        let mut loader = PromptLoader::new();
        loader.storage = storage.clone();
        loader.load_all()?;

        // Validate each loaded prompt
        for (_name, prompt) in storage.iter() {
            self.validate_prompt_data(&prompt, &mut result)?;
            result.files_checked += 1;
        }

        Ok(result)
    }

    pub fn validate_path<P: AsRef<Path>>(&self, path: P) -> Result<ValidationResult> {
        let path = path.as_ref();
        let mut result = ValidationResult::new();

        if path.is_file() {
            if let Some(ext) = path.extension() {
                if ext == "md" {
                    self.validate_file(path, &mut result)?;
                    result.files_checked += 1;
                } else {
                    result.add_issue(ValidationIssue {
                        level: ValidationLevel::Warning,
                        file_path: path.to_path_buf(),
                        line: None,
                        column: None,
                        message: "Only .md files are supported for prompt validation".to_string(),
                        suggestion: Some("Ensure prompt files have .md extension".to_string()),
                    });
                }
            }
        } else if path.is_dir() {
            self.validate_directory(path, &mut result)?;
        } else {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: path.to_path_buf(),
                line: None,
                column: None,
                message: "Path does not exist or is not accessible".to_string(),
                suggestion: Some("Check the file path and permissions".to_string()),
            });
        }

        Ok(result)
    }

    fn validate_directory(&self, dir: &Path, result: &mut ValidationResult) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_file() {
                if let Some(ext) = path.extension() {
                    if ext == "md" {
                        self.validate_file(&path, result)?;
                        result.files_checked += 1;
                    }
                }
            } else if path.is_dir() {
                // Recursively validate subdirectories
                self.validate_directory(&path, result)?;
            }
        }
        Ok(())
    }

    fn validate_file(&self, file_path: &Path, result: &mut ValidationResult) -> Result<()> {
        // Read file content
        let content = match fs::read_to_string(file_path) {
            Ok(content) => content,
            Err(e) => {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    line: None,
                    column: None,
                    message: format!("Failed to read file: {}", e),
                    suggestion: Some("Check file permissions and encoding".to_string()),
                });
                return Ok(());
            }
        };

        // Check UTF-8 encoding (already done by read_to_string, but let's be explicit)
        self.validate_encoding(&content, file_path, result);

        // Check line endings
        self.validate_line_endings(&content, file_path, result);

        // Parse and validate front matter, and if successful, validate the full prompt
        match self.parse_and_validate_prompt(&content, file_path, result) {
            Ok(Some((front_matter, prompt_content))) => {
                // We successfully parsed the front matter, now validate template variables
                let arguments = front_matter.arguments;
                self.validate_template_variables(&prompt_content, &arguments, file_path, result);
                
                // Validate required fields
                if front_matter.title.is_empty() {
                    result.add_issue(ValidationIssue {
                        level: ValidationLevel::Error,
                        file_path: file_path.to_path_buf(),
                        line: None,
                        column: None,
                        message: "Missing required field: title".to_string(),
                        suggestion: Some("Add a title field to the YAML front matter".to_string()),
                    });
                }
                
                if front_matter.description.is_empty() {
                    result.add_issue(ValidationIssue {
                        level: ValidationLevel::Error,
                        file_path: file_path.to_path_buf(),
                        line: None,
                        column: None,
                        message: "Missing required field: description".to_string(),
                        suggestion: Some("Add a description field to the YAML front matter".to_string()),
                    });
                }
            }
            Ok(None) => {
                // Front matter validation failed, errors already added
            }
            Err(e) => {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    line: None,
                    column: None,
                    message: format!("Failed to parse prompt: {}", e),
                    suggestion: Some("Check file format and syntax".to_string()),
                });
            }
        }

        Ok(())
    }

    fn validate_prompt_data(&self, prompt: &crate::prompts::Prompt, result: &mut ValidationResult) -> Result<()> {
        let file_path = PathBuf::from(&prompt.source_path);

        // Check required fields
        if prompt.title.is_none() || prompt.title.as_ref().unwrap().is_empty() {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: file_path.clone(),
                line: None,
                column: None,
                message: "Missing required field: title".to_string(),
                suggestion: Some("Add a title field to the YAML front matter".to_string()),
            });
        }

        if prompt.description.is_none() || prompt.description.as_ref().unwrap().is_empty() {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: file_path.clone(),
                line: None,
                column: None,
                message: "Missing required field: description".to_string(),
                suggestion: Some("Add a description field to the YAML front matter".to_string()),
            });
        }

        // Validate template variables
        self.validate_template_variables(&prompt.content, &prompt.arguments, &file_path, result);

        Ok(())
    }

    fn validate_encoding(&self, content: &str, file_path: &Path, result: &mut ValidationResult) {
        // If we can read it as a string, it's valid UTF-8
        // Check for BOM
        if content.starts_with('\u{FEFF}') {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Warning,
                file_path: file_path.to_path_buf(),
                line: Some(1),
                column: Some(1),
                message: "File contains UTF-8 BOM".to_string(),
                suggestion: Some("Remove the BOM for better compatibility".to_string()),
            });
        }
    }

    fn validate_line_endings(&self, content: &str, file_path: &Path, result: &mut ValidationResult) {
        let has_crlf = content.contains("\r\n");
        let has_lf_only = content.contains('\n') && !content.contains("\r\n");
        
        if has_crlf && has_lf_only {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Warning,
                file_path: file_path.to_path_buf(),
                line: None,
                column: None,
                message: "Mixed line endings detected (both CRLF and LF)".to_string(),
                suggestion: Some("Use consistent line endings throughout the file".to_string()),
            });
        }
    }

    fn parse_and_validate_prompt(&self, content: &str, file_path: &Path, result: &mut ValidationResult) -> Result<Option<(crate::prompts::PromptFrontMatter, String)>> {
        if !content.starts_with("---") {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: file_path.to_path_buf(),
                line: Some(1),
                column: Some(1),
                message: "Missing YAML front matter delimiter".to_string(),
                suggestion: Some("Start file with '---' to begin YAML front matter".to_string()),
            });
            return Ok(None);
        }

        // Find the end of front matter
        let lines: Vec<&str> = content.lines().collect();
        let mut end_line = None;
        
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                end_line = Some(i);
                break;
            }
        }

        let end_line = match end_line {
            Some(line) => line,
            None => {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    line: Some(1),
                    column: Some(1),
                    message: "Missing closing YAML front matter delimiter".to_string(),
                    suggestion: Some("Add '---' to close the YAML front matter".to_string()),
                });
                return Ok(None);
            }
        };

        // Extract YAML and prompt content
        let yaml_content: String = lines[1..end_line].join("\n");
        let prompt_content: String = lines[end_line + 1..].join("\n");
        
        match serde_yaml::from_str::<crate::prompts::PromptFrontMatter>(&yaml_content) {
            Ok(front_matter) => {
                // YAML is valid, now check for common typos
                self.validate_yaml_fields(&yaml_content, file_path, result);
                Ok(Some((front_matter, prompt_content)))
            }
            Err(e) => {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    line: Some(e.location().map(|l| l.line()).unwrap_or(1)),
                    column: Some(e.location().map(|l| l.column()).unwrap_or(1)),
                    message: format!("YAML syntax error: {}", e),
                    suggestion: Some("Fix YAML syntax according to the error message".to_string()),
                });
                Ok(None)
            }
        }
    }


    fn validate_yaml_fields(&self, yaml_content: &str, file_path: &Path, result: &mut ValidationResult) {
        // Check for common typos in field names
        let common_typos = [
            ("titel", "title"),
            ("descripton", "description"),
            ("argumnets", "arguments"),
            ("requried", "required"),
        ];

        for line in yaml_content.lines() {
            for (typo, correct) in &common_typos {
                if line.contains(typo) {
                    result.add_issue(ValidationIssue {
                        level: ValidationLevel::Warning,
                        file_path: file_path.to_path_buf(),
                        line: None,
                        column: None,
                        message: format!("Possible typo: '{}' should be '{}'", typo, correct),
                        suggestion: Some(format!("Replace '{}' with '{}'", typo, correct)),
                    });
                }
            }
        }
    }

    fn validate_template_variables(&self, content: &str, arguments: &[crate::prompts::PromptArgument], file_path: &Path, result: &mut ValidationResult) {
        // First validate the Liquid template syntax
        self.validate_liquid_syntax(content, file_path, result);
        
        // Then validate variable usage
        self.validate_variable_usage(content, arguments, file_path, result);
    }

    fn validate_liquid_syntax(&self, content: &str, file_path: &Path, result: &mut ValidationResult) {
        let engine = LiquidEngine::new();
        
        // Try to parse the template with strict mode (no backward compatibility)
        // to catch Liquid syntax errors
        let empty_args = std::collections::HashMap::new();
        if let Err(e) = engine.process_with_compatibility(content, &empty_args, false) {
            let error_msg = e.to_string();
            
            // Only report actual syntax errors, not unknown variable errors
            if !error_msg.contains("Unknown variable") {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    line: None,
                    column: None,
                    message: format!("Liquid template syntax error: {}", error_msg),
                    suggestion: Some("Check Liquid template syntax and fix any errors".to_string()),
                });
            }
        }
    }

    fn validate_variable_usage(&self, content: &str, arguments: &[crate::prompts::PromptArgument], file_path: &Path, result: &mut ValidationResult) {
        use regex::Regex;
        
        // Enhanced regex to match various Liquid variable patterns
        let patterns = [
            // Simple variables: {{ variable }}
            r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}",
            // Variables with filters: {{ variable | filter }}
            r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\|",
            // Object properties: {{ object.property }}
            r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\.[a-zA-Z_][a-zA-Z0-9_]*",
            // Array access: {{ array[0] }}
            r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\[",
        ];
        
        let mut used_variables = std::collections::HashSet::new();
        
        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for captures in regex.captures_iter(content) {
                    if let Some(var_match) = captures.get(1) {
                        let var_name = var_match.as_str().trim();
                        // Skip 'env' as it's a special built-in object
                        if var_name != "env" {
                            used_variables.insert(var_name.to_string());
                        }
                    }
                }
            }
        }
        
        // Also check for loop variables in {% for %} statements
        let for_regex = Regex::new(r"\{\%\s*for\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+in\s+([a-zA-Z_][a-zA-Z0-9_]*)")
            .unwrap();
        for captures in for_regex.captures_iter(content) {
            if let Some(collection_match) = captures.get(2) {
                let collection_name = collection_match.as_str().trim();
                used_variables.insert(collection_name.to_string());
            }
        }

        // Check if all used variables are defined in arguments
        let defined_args: std::collections::HashSet<String> = arguments.iter()
            .map(|arg| arg.name.clone())
            .collect();

        for used_var in &used_variables {
            if !defined_args.contains(used_var) {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    line: None,
                    column: None,
                    message: format!("Undefined template variable: '{}'", used_var),
                    suggestion: Some(format!("Add '{}' to the arguments list or remove the template variable", used_var)),
                });
            }
        }

        // Check for unused arguments (warning)
        for arg in arguments {
            if !used_variables.contains(&arg.name) {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Warning,
                    file_path: file_path.to_path_buf(),
                    line: None,
                    column: None,
                    message: format!("Unused argument: '{}'", arg.name),
                    suggestion: Some(format!("Remove '{}' from arguments or use it in the template", arg.name)),
                });
            }
        }

        // Check if template has variables but no arguments defined
        if !used_variables.is_empty() && arguments.is_empty() {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Warning,
                file_path: file_path.to_path_buf(),
                line: None,
                column: None,
                message: "Template uses variables but no arguments are defined".to_string(),
                suggestion: Some("Define arguments for the template variables".to_string()),
            });
        }
    }

    pub fn print_results(&self, result: &ValidationResult, format: ValidateFormat) -> Result<()> {
        match format {
            ValidateFormat::Text => self.print_text_results(result),
            ValidateFormat::Json => self.print_json_results(result)?,
        }
        Ok(())
    }

    fn print_text_results(&self, result: &ValidationResult) {
        if result.issues.is_empty() {
            if !self.quiet {
                println!("{} All {} files validated successfully!", "✓".green(), result.files_checked);
            }
            return;
        }

        // Group issues by file
        let mut issues_by_file: std::collections::HashMap<PathBuf, Vec<&ValidationIssue>> = std::collections::HashMap::new();
        
        for issue in &result.issues {
            issues_by_file.entry(issue.file_path.clone()).or_default().push(issue);
        }

        // Print issues grouped by file
        for (file_path, issues) in issues_by_file {
            if !self.quiet {
                println!("\n{}", file_path.display().to_string().bold());
            }
            
            for issue in issues {
                let level_str = match issue.level {
                    ValidationLevel::Error => "ERROR".red(),
                    ValidationLevel::Warning => "WARN".yellow(),
                    ValidationLevel::Info => "INFO".blue(),
                };

                let location = if let (Some(line), Some(col)) = (issue.line, issue.column) {
                    format!("{}:{}", line, col)
                } else if let Some(line) = issue.line {
                    format!("{}", line)
                } else {
                    "-".to_string()
                };

                if self.quiet && issue.level != ValidationLevel::Error {
                    continue;
                }

                println!("  {} [{}] {}", level_str, location, issue.message);
                
                if !self.quiet {
                    if let Some(suggestion) = &issue.suggestion {
                        println!("    💡 {}", suggestion.dimmed());
                    }
                }
            }
        }

        if !self.quiet {
            println!("\n{}", "Summary:".bold());
            println!("  Files checked: {}", result.files_checked);
            if result.errors > 0 {
                println!("  Errors: {}", result.errors.to_string().red());
            }
            if result.warnings > 0 {
                println!("  Warnings: {}", result.warnings.to_string().yellow());
            }
            
            if result.has_errors() {
                println!("\n{} Validation failed with errors.", "✗".red());
            } else if result.has_warnings() {
                println!("\n{} Validation completed with warnings.", "⚠".yellow());
            } else {
                println!("\n{} Validation passed!", "✓".green());
            }
        }
    }

    fn print_json_results(&self, result: &ValidationResult) -> Result<()> {
        let json_issues: Vec<JsonValidationIssue> = result.issues.iter().map(|issue| {
            JsonValidationIssue {
                level: match issue.level {
                    ValidationLevel::Error => "error".to_string(),
                    ValidationLevel::Warning => "warning".to_string(),
                    ValidationLevel::Info => "info".to_string(),
                },
                file_path: issue.file_path.display().to_string(),
                line: issue.line,
                column: issue.column,
                message: issue.message.clone(),
                suggestion: issue.suggestion.clone(),
            }
        }).collect();

        let json_result = JsonValidationResult {
            files_checked: result.files_checked,
            errors: result.errors,
            warnings: result.warnings,
            issues: json_issues,
        };

        println!("{}", serde_json::to_string_pretty(&json_result)?);
        Ok(())
    }
}

pub fn run_validate_command(
    path: Option<String>,
    all: bool,
    quiet: bool,
    format: ValidateFormat,
) -> Result<i32> {
    let validator = Validator::new(quiet);
    
    let result = if all {
        validator.validate_all()?
    } else if let Some(path) = path {
        validator.validate_path(&path)?
    } else {
        // If no path and not --all, default to current directory
        validator.validate_path(".")?
    };

    validator.print_results(&result, format)?;

    // Return appropriate exit code
    if result.has_errors() {
        Ok(2) // Errors
    } else if result.has_warnings() {
        Ok(1) // Warnings
    } else {
        Ok(0) // Success
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_validation_result_creation() {
        let result = ValidationResult::new();
        assert_eq!(result.files_checked, 0);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_validation_result_add_error() {
        let mut result = ValidationResult::new();
        let issue = ValidationIssue {
            level: ValidationLevel::Error,
            file_path: PathBuf::from("test.md"),
            line: Some(1),
            column: Some(1),
            message: "Test error".to_string(),
            suggestion: None,
        };
        
        result.add_issue(issue);
        assert_eq!(result.errors, 1);
        assert_eq!(result.warnings, 0);
        assert!(result.has_errors());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_validation_result_add_warning() {
        let mut result = ValidationResult::new();
        let issue = ValidationIssue {
            level: ValidationLevel::Warning,
            file_path: PathBuf::from("test.md"),
            line: Some(1),
            column: Some(1),
            message: "Test warning".to_string(),
            suggestion: None,
        };
        
        result.add_issue(issue);
        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 1);
        assert!(!result.has_errors());
        assert!(result.has_warnings());
    }

    #[test]
    fn test_validator_creation() {
        let validator = Validator::new(false);
        assert!(!validator.quiet);

        let quiet_validator = Validator::new(true);
        assert!(quiet_validator.quiet);
    }

    #[test]
    fn test_validate_nonexistent_path() {
        let validator = Validator::new(false);
        let result = validator.validate_path("/nonexistent/path").unwrap();
        
        assert_eq!(result.files_checked, 0);
        assert!(result.has_errors());
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_validate_non_markdown_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        fs::write(&file_path, "Hello world").unwrap();

        let validator = Validator::new(false);
        let result = validator.validate_path(&file_path).unwrap();
        
        assert_eq!(result.files_checked, 0);
        assert!(result.has_warnings());
        assert_eq!(result.warnings, 1);
    }

    #[test]
    fn test_validate_valid_markdown_file() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let content = r#"---
title: Test Prompt
description: A test prompt for validation
arguments:
  - name: topic
    description: The topic to discuss
    required: true
---

# Test Prompt

Please discuss {{topic}} in detail.
"#;
        fs::write(&file_path, content).unwrap();

        let validator = Validator::new(false);
        let result = validator.validate_path(&file_path).unwrap();
        
        assert_eq!(result.files_checked, 1);
        assert!(!result.has_errors());
        assert!(!result.has_warnings());
    }

    #[test]
    fn test_validate_missing_front_matter() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let content = "# Test Prompt\n\nThis is a test.";
        fs::write(&file_path, content).unwrap();

        let validator = Validator::new(false);
        let result = validator.validate_path(&file_path).unwrap();
        
        assert_eq!(result.files_checked, 1);
        assert!(result.has_errors());
        assert_eq!(result.errors, 1);
    }

    #[test]
    fn test_validate_invalid_yaml() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let content = r#"---
title: Test Prompt
description: [invalid yaml
---

# Test
"#;
        fs::write(&file_path, content).unwrap();

        let validator = Validator::new(false);
        let result = validator.validate_path(&file_path).unwrap();
        
        assert_eq!(result.files_checked, 1);
        assert!(result.has_errors());
    }

    #[test]
    fn test_validate_undefined_template_variable() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let content = r#"---
title: Test Prompt
description: A test prompt
arguments:
  - name: topic
    description: The topic
    required: true
---

# Test

Discuss {{topic}} and {{undefined_var}}.
"#;
        fs::write(&file_path, content).unwrap();

        let validator = Validator::new(false);
        let result = validator.validate_path(&file_path).unwrap();
        
        assert_eq!(result.files_checked, 1);
        assert!(result.has_errors());
        
        // Should have error for undefined variable
        let undefined_error = result.issues.iter()
            .find(|issue| issue.message.contains("undefined_var"));
        assert!(undefined_error.is_some());
    }

    #[test]
    fn test_validate_unused_argument() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let content = r#"---
title: Test Prompt
description: A test prompt
arguments:
  - name: topic
    description: The topic
    required: true
  - name: unused_arg
    description: Not used
    required: false
---

# Test

Discuss {{topic}}.
"#;
        fs::write(&file_path, content).unwrap();

        let validator = Validator::new(false);
        let result = validator.validate_path(&file_path).unwrap();
        
        assert_eq!(result.files_checked, 1);
        assert!(!result.has_errors());
        assert!(result.has_warnings());
        
        // Should have warning for unused argument
        let unused_warning = result.issues.iter()
            .find(|issue| issue.message.contains("unused_arg"));
        assert!(unused_warning.is_some());
    }

    #[test]
    fn test_run_validate_command_nonexistent() {
        let exit_code = run_validate_command(
            Some("/nonexistent/path".to_string()),
            false,
            true,
            ValidateFormat::Text,
        ).unwrap();
        
        assert_eq!(exit_code, 2); // Should return error exit code
    }

    #[test]
    fn test_run_validate_command_success() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.md");
        let content = r#"---
title: Test Prompt
description: A test prompt
arguments:
  - name: topic
    description: The topic
    required: true
---

# Test

Discuss {{topic}}.
"#;
        fs::write(&file_path, content).unwrap();

        let exit_code = run_validate_command(
            Some(file_path.to_string_lossy().to_string()),
            false,
            true,
            ValidateFormat::Text,
        ).unwrap();
        
        assert_eq!(exit_code, 0); // Should return success exit code
    }
}