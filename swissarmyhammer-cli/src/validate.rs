use anyhow::Result;
use colored::*;
use serde::Serialize;
use std::path::{Path, PathBuf};

use crate::cli::ValidateFormat;

// Local structs for validation
#[derive(Debug, Clone, serde::Deserialize)]
struct PromptArgument {
    name: String,
    // Fields used through Clone during mapping to main PromptArgument type
    #[allow(dead_code)]
    description: Option<String>,
    #[allow(dead_code)]
    required: bool,
    #[allow(dead_code)]
    default: Option<String>,
}

#[derive(Debug, Clone, serde::Deserialize)]
struct PromptFrontMatter {
    // Used for YAML deserialization but not directly accessed
    #[allow(dead_code)]
    title: String,
    #[allow(dead_code)]
    description: String,
    #[serde(default)]
    #[allow(dead_code)]
    arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone)]
struct Prompt {
    #[allow(dead_code)] // Only used during construction
    name: String,
    title: Option<String>,
    description: Option<String>,
    source_path: String,
    content: String,
    arguments: Vec<PromptArgument>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValidationLevel {
    Error,
    Warning,
    #[allow(dead_code)] // Available for future use
    Info,
}

#[derive(Debug, Clone)]
pub struct ValidationIssue {
    pub level: ValidationLevel,
    pub file_path: PathBuf,
    pub prompt_title: Option<String>,
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

        // Load all prompts using the centralized PromptResolver
        let mut library = swissarmyhammer::PromptLibrary::new();
        let mut resolver = swissarmyhammer::PromptResolver::new();
        resolver.load_all_prompts(&mut library)?;

        // Validate each loaded prompt
        let prompts = library.list()?;
        for prompt in prompts {
            result.files_checked += 1;

            // Store prompt title for error reporting
            let prompt_title = prompt
                .metadata
                .get("title")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| Some(prompt.name.clone()));

            // Validate template syntax with partials support
            self.validate_liquid_syntax_with_partials(
                &prompt,
                &library,
                prompt.source.as_ref().unwrap_or(&PathBuf::new()),
                &mut result,
                prompt_title.clone(),
            );

            // Create local prompt for field validation
            let local_prompt = Prompt {
                name: prompt.name.clone(),
                title: prompt
                    .metadata
                    .get("title")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string()),
                description: prompt.description.clone(),
                source_path: prompt
                    .source
                    .as_ref()
                    .map(|p| p.to_string_lossy().to_string())
                    .unwrap_or_default(),
                content: prompt.template.clone(),
                arguments: prompt
                    .arguments
                    .iter()
                    .map(|arg| PromptArgument {
                        name: arg.name.clone(),
                        description: arg.description.clone(),
                        required: arg.required,
                        default: arg.default.clone(),
                    })
                    .collect(),
            };

            // Validate fields and variables (but skip liquid syntax since we did it above)
            self.validate_prompt_fields_and_variables(&local_prompt, &mut result, prompt_title)?;
        }

        Ok(result)
    }

    fn validate_prompt_fields_and_variables(
        &self,
        prompt: &Prompt,
        result: &mut ValidationResult,
        prompt_title: Option<String>,
    ) -> Result<()> {
        let file_path = PathBuf::from(&prompt.source_path);

        // Check if this is a partial template by looking at the description
        let is_partial = prompt
            .description
            .as_ref()
            .map(|desc| desc == "Partial template for reuse in other prompts")
            .unwrap_or(false);

        // Skip field validation for partial templates
        if !is_partial {
            // Check required fields
            if prompt.title.is_none() || prompt.title.as_ref().unwrap().is_empty() {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.clone(),
                    prompt_title: prompt_title.clone(),
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
                    prompt_title: prompt_title.clone(),
                    line: None,
                    column: None,
                    message: "Missing required field: description".to_string(),
                    suggestion: Some(
                        "Add a description field to the YAML front matter".to_string(),
                    ),
                });
            }
        }

        // Validate template variables (without liquid syntax validation)
        self.validate_variable_usage(
            &prompt.content,
            &prompt.arguments,
            &file_path,
            result,
            prompt_title,
        );

        Ok(())
    }

    #[allow(dead_code)]
    fn validate_encoding(&self, content: &str, file_path: &Path, result: &mut ValidationResult) {
        // If we can read it as a string, it's valid UTF-8
        // Check for BOM
        if content.starts_with('\u{FEFF}') {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Warning,
                file_path: file_path.to_path_buf(),
                prompt_title: None,
                line: Some(1),
                column: Some(1),
                message: "File contains UTF-8 BOM".to_string(),
                suggestion: Some("Remove the BOM for better compatibility".to_string()),
            });
        }
    }

    #[allow(dead_code)]
    fn validate_line_endings(
        &self,
        content: &str,
        file_path: &Path,
        result: &mut ValidationResult,
    ) {
        let has_crlf = content.contains("\r\n");
        let has_lf_only = content.contains('\n') && !content.contains("\r\n");

        if has_crlf && has_lf_only {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Warning,
                file_path: file_path.to_path_buf(),
                prompt_title: None,
                line: None,
                column: None,
                message: "Mixed line endings detected (both CRLF and LF)".to_string(),
                suggestion: Some("Use consistent line endings throughout the file".to_string()),
            });
        }
    }

    #[allow(dead_code)]
    fn parse_and_validate_prompt(
        &self,
        content: &str,
        file_path: &Path,
        result: &mut ValidationResult,
    ) -> Result<Option<(PromptFrontMatter, String)>> {
        if !content.starts_with("---") {
            let suggestion = if file_path
                .extension()
                .map(|e| e == "liquid")
                .unwrap_or(false)
            {
                "Start file with '---' to begin YAML front matter\n💡 Add {% partial %} to disable YAML front matter checking".to_string()
            } else {
                "Start file with '---' to begin YAML front matter".to_string()
            };

            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: file_path.to_path_buf(),
                prompt_title: None,
                line: Some(1),
                column: Some(1),
                message: "Missing YAML front matter delimiter".to_string(),
                suggestion: Some(suggestion),
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
                let suggestion = if file_path
                    .extension()
                    .map(|e| e == "liquid")
                    .unwrap_or(false)
                {
                    "Add '---' to close the YAML front matter\n💡 Add {% partial %} to disable YAML front matter checking".to_string()
                } else {
                    "Add '---' to close the YAML front matter".to_string()
                };

                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    prompt_title: None,
                    line: Some(1),
                    column: Some(1),
                    message: "Missing closing YAML front matter delimiter".to_string(),
                    suggestion: Some(suggestion),
                });
                return Ok(None);
            }
        };

        // Extract YAML and prompt content
        let yaml_content: String = lines[1..end_line].join("\n");
        let prompt_content: String = lines[end_line + 1..].join("\n");

        match serde_yaml::from_str::<PromptFrontMatter>(&yaml_content) {
            Ok(front_matter) => {
                // YAML is valid, now check for common typos
                self.validate_yaml_fields(&yaml_content, file_path, result);
                Ok(Some((front_matter, prompt_content)))
            }
            Err(e) => {
                let suggestion = if file_path
                    .extension()
                    .map(|e| e == "liquid")
                    .unwrap_or(false)
                {
                    "Fix YAML syntax according to the error message\n💡 Add {% partial %} to disable YAML front matter checking".to_string()
                } else {
                    "Fix YAML syntax according to the error message".to_string()
                };

                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    prompt_title: None,
                    line: Some(e.location().map(|l| l.line()).unwrap_or(1)),
                    column: Some(e.location().map(|l| l.column()).unwrap_or(1)),
                    message: format!("YAML syntax error: {}", e),
                    suggestion: Some(suggestion),
                });
                Ok(None)
            }
        }
    }

    #[allow(dead_code)]
    fn validate_yaml_fields(
        &self,
        yaml_content: &str,
        file_path: &Path,
        result: &mut ValidationResult,
    ) {
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
                        prompt_title: None,
                        line: None,
                        column: None,
                        message: format!("Possible typo: '{}' should be '{}'", typo, correct),
                        suggestion: Some(format!("Replace '{}' with '{}'", typo, correct)),
                    });
                }
            }
        }
    }

    #[allow(dead_code)]
    fn validate_template_variables(
        &self,
        content: &str,
        arguments: &[PromptArgument],
        file_path: &Path,
        result: &mut ValidationResult,
    ) {
        // First validate the Liquid template syntax
        self.validate_liquid_syntax(content, file_path, result);

        // Then validate variable usage
        self.validate_variable_usage(content, arguments, file_path, result, None);
    }

    fn validate_liquid_syntax(
        &self,
        content: &str,
        file_path: &Path,
        result: &mut ValidationResult,
    ) {
        use swissarmyhammer::TemplateEngine;

        let engine = TemplateEngine::new();

        // Try to parse the template to catch syntax errors
        let empty_args = std::collections::HashMap::new();
        if let Err(e) = engine.render(content, &empty_args) {
            let error_msg = e.to_string();

            // Only report actual syntax errors, not unknown variable errors
            if !error_msg.contains("Unknown variable") {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    prompt_title: None,
                    line: None,
                    column: None,
                    message: format!("Liquid template syntax error: {}", error_msg),
                    suggestion: Some("Check Liquid template syntax and fix any errors".to_string()),
                });
            }
        }
    }

    fn validate_liquid_syntax_with_partials(
        &self,
        prompt: &swissarmyhammer::Prompt,
        library: &swissarmyhammer::PromptLibrary,
        file_path: &Path,
        result: &mut ValidationResult,
        prompt_title: Option<String>,
    ) {
        // Try to render the template with partials support using the same path as test/serve
        let empty_args = std::collections::HashMap::new();

        // Use render_prompt which internally uses render_with_partials
        if let Err(e) = library.render_prompt(&prompt.name, &empty_args) {
            let error_msg = e.to_string();

            // Only report actual syntax errors, not unknown variable errors
            if !error_msg.contains("Unknown variable") && !error_msg.contains("Required argument") {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    prompt_title,
                    line: None,
                    column: None,
                    message: format!("Liquid template syntax error: {}", error_msg),
                    suggestion: Some(
                        "Check Liquid template syntax and partial references".to_string(),
                    ),
                });
            }
        }
    }

    fn validate_variable_usage(
        &self,
        content: &str,
        arguments: &[PromptArgument],
        file_path: &Path,
        result: &mut ValidationResult,
        prompt_title: Option<String>,
    ) {
        use regex::Regex;

        // Remove {% raw %} blocks from content before validation
        let raw_regex = Regex::new(r"(?s)\{%\s*raw\s*%\}.*?\{%\s*endraw\s*%\}").unwrap();
        let content_without_raw = raw_regex.replace_all(content, "");

        // Enhanced regex to match various Liquid variable patterns
        let patterns = [
            // Simple variables: {{ variable }}
            r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\}\}",
            // Variables with filters: {{ variable | filter }}
            r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\s*\|",
            // Variables as filter arguments: {{ "value" | filter: variable }}
            r"\|\s*[a-zA-Z_][a-zA-Z0-9_]*\s*:\s*([a-zA-Z_][a-zA-Z0-9_]*)",
            // Object properties: {{ object.property }}
            r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\.[a-zA-Z_][a-zA-Z0-9_]*",
            // Array access: {{ array[0] }}
            r"\{\{\s*([a-zA-Z_][a-zA-Z0-9_]*)\[",
            // Case statements: {% case variable %}
            r"\{\%\s*case\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\%\}",
            // If statements: {% if variable %}
            r"\{\%\s*if\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*[%}=<>!]",
            // Unless statements: {% unless variable %}
            r"\{\%\s*unless\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*[%}=<>!]",
            // Elsif statements: {% elsif variable %}
            r"\{\%\s*elsif\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*[%}=<>!]",
            // Variable comparisons: {% if variable == "value" %}
            r"\{\%\s*(?:if|elsif|unless)\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*[=<>!]",
            // Assignment statements: {% assign var = variable %}
            r"\{\%\s*assign\s+[a-zA-Z_][a-zA-Z0-9_]*\s*=\s*([a-zA-Z_][a-zA-Z0-9_]*)",
        ];

        let mut used_variables = std::collections::HashSet::new();

        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                for captures in regex.captures_iter(&content_without_raw) {
                    if let Some(var_match) = captures.get(1) {
                        let var_name = var_match.as_str().trim();
                        // Skip built-in Liquid objects and variables
                        let builtin_vars = ["env", "forloop", "tablerow", "paginate"];
                        if !builtin_vars.contains(&var_name) {
                            used_variables.insert(var_name.to_string());
                        }
                    }
                }
            }
        }

        // Find assigned variables with {% assign %} statements
        let assign_regex = Regex::new(r"\{\%\s*assign\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*=").unwrap();
        let mut assigned_variables = std::collections::HashSet::new();
        for captures in assign_regex.captures_iter(&content_without_raw) {
            if let Some(var_match) = captures.get(1) {
                assigned_variables.insert(var_match.as_str().trim().to_string());
            }
        }

        // Also check for loop variables in {% for %} statements
        let for_regex =
            Regex::new(r"\{\%\s*for\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+in\s+([a-zA-Z_][a-zA-Z0-9_]*)")
                .unwrap();
        for captures in for_regex.captures_iter(&content_without_raw) {
            if let Some(loop_var) = captures.get(1) {
                // The loop variable is defined by the for loop
                assigned_variables.insert(loop_var.as_str().trim().to_string());
            }
            if let Some(collection_match) = captures.get(2) {
                let collection_name = collection_match.as_str().trim();
                used_variables.insert(collection_name.to_string());
            }
        }

        // Also find variables from {% capture %} blocks
        let capture_regex =
            Regex::new(r"\{\%\s*capture\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*\%\}").unwrap();
        for captures in capture_regex.captures_iter(&content_without_raw) {
            if let Some(var_match) = captures.get(1) {
                assigned_variables.insert(var_match.as_str().trim().to_string());
            }
        }

        // Check if all used variables are defined in arguments
        let defined_args: std::collections::HashSet<String> =
            arguments.iter().map(|arg| arg.name.clone()).collect();

        for used_var in &used_variables {
            // Skip if this variable is defined within the template
            if assigned_variables.contains(used_var) {
                continue;
            }

            // Check if it's defined in arguments
            if !defined_args.contains(used_var) {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: file_path.to_path_buf(),
                    prompt_title: prompt_title.clone(),
                    line: None,
                    column: None,
                    message: format!("Undefined template variable: '{}'", used_var),
                    suggestion: Some(format!(
                        "Add '{}' to the arguments list or remove the template variable",
                        used_var
                    )),
                });
            }
        }

        // Check for unused arguments (warning)
        for arg in arguments {
            if !used_variables.contains(&arg.name) {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Warning,
                    file_path: file_path.to_path_buf(),
                    prompt_title: prompt_title.clone(),
                    line: None,
                    column: None,
                    message: format!("Unused argument: '{}'", arg.name),
                    suggestion: Some(format!(
                        "Remove '{}' from arguments or use it in the template",
                        arg.name
                    )),
                });
            }
        }

        // Check if template has variables but no arguments defined
        if !used_variables.is_empty() && arguments.is_empty() {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Warning,
                file_path: file_path.to_path_buf(),
                prompt_title,
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
                println!(
                    "{} All {} files validated successfully!",
                    "✓".green(),
                    result.files_checked
                );
            }
            return;
        }

        // Group issues by file
        let mut issues_by_file: std::collections::HashMap<PathBuf, Vec<&ValidationIssue>> =
            std::collections::HashMap::new();

        for issue in &result.issues {
            issues_by_file
                .entry(issue.file_path.clone())
                .or_default()
                .push(issue);
        }

        // Print issues grouped by file
        for (file_path, issues) in issues_by_file {
            if !self.quiet {
                // Get the prompt title from the first issue (all issues for a file should have the same title)
                let prompt_title = issues.first().and_then(|issue| issue.prompt_title.as_ref());

                if let Some(title) = prompt_title {
                    // Show the prompt title
                    println!("\n{}", title.bold());
                    // Show the file path in smaller text if it's a user prompt
                    if file_path.to_string_lossy() != ""
                        && !file_path.to_string_lossy().contains("PathBuf")
                    {
                        println!("  {}", file_path.display().to_string().dimmed());
                    }
                } else {
                    // Fallback to file path if no title
                    println!("\n{}", file_path.display().to_string().bold());
                }
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
        let json_issues: Vec<JsonValidationIssue> = result
            .issues
            .iter()
            .map(|issue| JsonValidationIssue {
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
            })
            .collect();

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

pub fn run_validate_command(quiet: bool, format: ValidateFormat) -> Result<i32> {
    let validator = Validator::new(quiet);

    // Always validate all prompts
    let result = validator.validate_all()?;

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
    // Note: Many tests have been temporarily disabled after simplifying the validate command
    // to always validate all prompts. These tests need to be rewritten to work with the new
    // simplified validation approach.

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
            prompt_title: Some("Test Prompt".to_string()),
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
            prompt_title: Some("Test Prompt".to_string()),
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
    fn test_validate_all_handles_partial_templates() {
        // This test verifies that .liquid files with {% partial %} marker
        // don't generate errors for missing title/description
        let validator = Validator::new(false);

        // Note: This test relies on the actual prompt loading mechanism
        // which will load test files from the test environment
        let result = validator.validate_all().unwrap();

        // Check that partial templates don't cause title/description errors
        let partial_errors = result
            .issues
            .iter()
            .filter(|issue| {
                issue.file_path.to_string_lossy().ends_with(".liquid")
                    && (issue.message.contains("Missing required field: title")
                        || issue
                            .message
                            .contains("Missing required field: description"))
            })
            .count();

        assert_eq!(partial_errors, 0,
            "Partial templates (with {{% partial %}} marker) should not have title/description errors");
    }
}
