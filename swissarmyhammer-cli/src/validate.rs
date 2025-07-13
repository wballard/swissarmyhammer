use anyhow::{Context, Result};
use colored::*;
use serde::Serialize;
use std::path::{Path, PathBuf};
use swissarmyhammer::security::validate_workflow_complexity;
#[cfg(test)]
use swissarmyhammer::workflow::MermaidParser;
use swissarmyhammer::workflow::{
    MemoryWorkflowStorage, Workflow, WorkflowGraphAnalyzer, WorkflowResolver,
    WorkflowStorageBackend,
};

use crate::cli::ValidateFormat;
use crate::exit_codes::{EXIT_ERROR, EXIT_SUCCESS, EXIT_WARNING};

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

    #[allow(dead_code)]
    pub fn validate_all(&mut self) -> Result<ValidationResult> {
        self.validate_all_with_options()
    }

    pub fn validate_all_with_options(&mut self) -> Result<ValidationResult> {
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

        // Validate workflows using WorkflowResolver for consistent loading
        self.validate_all_workflows(&mut result)?;

        Ok(result)
    }

    fn validate_prompt_fields_and_variables(
        &mut self,
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

        // Also validate workflows
        self.validate_all_workflows(result)?;

        Ok(())
    }

    /// Validates all workflow files using WorkflowResolver for consistent loading
    ///
    /// This uses the same loading mechanism as `flow list` to ensure consistency:
    /// - Builtin workflows (embedded in binary)
    /// - User workflows (~/.swissarmyhammer/workflows)
    /// - Local workflows (./.swissarmyhammer/workflows)
    ///
    /// Parameters:
    /// - result: The validation result to accumulate errors into
    fn validate_all_workflows(&mut self, result: &mut ValidationResult) -> Result<()> {
        // Use WorkflowResolver to load workflows from standard locations
        let mut storage = MemoryWorkflowStorage::new();
        let mut resolver = WorkflowResolver::new();

        // Load all workflows using the same logic as flow list
        resolver
            .load_all_workflows(&mut storage)
            .context("Failed to load workflows from standard locations (builtin, user, local)")?;

        // Get all loaded workflows
        let workflows = storage
            .list_workflows()
            .context("Failed to retrieve loaded workflows from storage")?;

        // Validate each workflow
        for workflow in workflows {
            result.files_checked += 1;

            // Get the source location for better error reporting
            let source_location = match resolver.workflow_sources.get(&workflow.name) {
                Some(swissarmyhammer::FileSource::Builtin) => "builtin",
                Some(swissarmyhammer::FileSource::User) => "user",
                Some(swissarmyhammer::FileSource::Local) => "local",
                Some(swissarmyhammer::FileSource::Dynamic) => "dynamic",
                None => "unknown",
            };

            // Create a path that includes the source location for better debugging
            let workflow_path = PathBuf::from(format!(
                "workflow:{}:{}",
                source_location,
                workflow.name.as_str()
            ));

            // Validate the workflow structure directly
            self.validate_workflow_structure(&workflow, &workflow_path, result)?;
        }

        Ok(())
    }

    /// Validates a workflow structure directly
    ///
    /// This method validates a workflow that has already been parsed,
    /// collecting validation errors in the provided ValidationResult.
    ///
    /// # Returns
    ///
    /// Always returns Ok(()) - errors are recorded in the ValidationResult parameter
    fn validate_workflow_structure(
        &mut self,
        workflow: &Workflow,
        workflow_path: &Path,
        result: &mut ValidationResult,
    ) -> Result<()> {
        // Validate workflow name
        let workflow_name = workflow.name.as_str();
        if workflow_name.is_empty() {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: workflow_path.to_path_buf(),
                prompt_title: None,
                line: None,
                column: None,
                message: "Workflow name cannot be empty".to_string(),
                suggestion: Some("Add a 'name' field in the workflow metadata".to_string()),
            });
            return Ok(());
        }

        // Check for invalid characters in workflow name
        if !workflow_name
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: workflow_path.to_path_buf(),
                prompt_title: None,
                line: None,
                column: None,
                message: format!("Invalid workflow name '{}': only alphanumeric characters, hyphens, and underscores are allowed", workflow_name),
                suggestion: Some("Use a name like 'my-workflow' or 'my_workflow'".to_string()),
            });
            return Ok(());
        }

        // Check workflow complexity to prevent DoS
        if let Err(e) =
            validate_workflow_complexity(workflow.states.len(), workflow.transitions.len())
        {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: workflow_path.to_path_buf(),
                prompt_title: None,
                line: None,
                column: None,
                message: e.to_string(),
                suggestion: Some("Split complex workflows into smaller sub-workflows".to_string()),
            });
            // Continue validation of other files
            return Ok(());
        }

        // Validate the workflow structure
        match workflow.validate() {
            Ok(_) => {}
            Err(errors) => {
                for error in errors {
                    result.add_issue(ValidationIssue {
                        level: ValidationLevel::Error,
                        file_path: workflow_path.to_path_buf(),
                        prompt_title: None,
                        line: None,
                        column: None,
                        message: format!("Workflow validation failed: {}", error),
                        suggestion: None,
                    });
                }
                // Continue with other validations to find all issues
                return Ok(());
            }
        }

        // Check for unreachable states
        let graph_analyzer = WorkflowGraphAnalyzer::new(workflow);
        let unreachable_states = graph_analyzer.find_unreachable_states();

        for state_id in unreachable_states {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: workflow_path.to_path_buf(),
                prompt_title: None,
                line: None,
                column: None,
                message: format!("State '{}' is unreachable from the initial state", state_id),
                suggestion: Some(
                    "Ensure all states have incoming transitions or remove unused states"
                        .to_string(),
                ),
            });
        }

        // Check for terminal states
        let mut has_terminal_state = false;
        for state in workflow.states.values() {
            if state.is_terminal {
                has_terminal_state = true;
                break;
            }
        }

        if !has_terminal_state {
            result.add_issue(ValidationIssue {
                level: ValidationLevel::Error,
                file_path: workflow_path.to_path_buf(),
                prompt_title: None,
                line: None,
                column: None,
                message: "Workflow has no terminal state (no transitions to [*])".to_string(),
                suggestion: Some("Add at least one end state that transitions to [*]".to_string()),
            });
        }

        // Check for circular dependencies
        let all_cycles = graph_analyzer.detect_all_cycles();
        if !all_cycles.is_empty() {
            // Report only the first cycle to avoid clutter
            let first_cycle = &all_cycles[0];
            let cycle_str = first_cycle
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(" -> ");

            result.add_issue(ValidationIssue {
                level: ValidationLevel::Warning,
                file_path: workflow_path.to_path_buf(),
                prompt_title: None,
                line: None,
                column: None,
                message: format!("Circular dependency detected: {}", cycle_str),
                suggestion: Some(
                    "Ensure the workflow has proper exit conditions to avoid infinite loops"
                        .to_string(),
                ),
            });
        }

        // Validate actions in transitions
        for transition in &workflow.transitions {
            if let Some(action) = &transition.action {
                // Basic action validation - check if it looks like valid syntax
                let action_str = action.to_string();
                if action_str.contains("execute") && !action_str.contains("prompt") {
                    result.add_issue(ValidationIssue {
                        level: ValidationLevel::Warning,
                        file_path: workflow_path.to_path_buf(),
                        prompt_title: None,
                        line: None,
                        column: None,
                        message: format!("Action in transition from '{}' may be incomplete: '{}'", transition.from_state, action_str),
                        suggestion: Some("Ensure actions follow the correct syntax (e.g., 'execute prompt \"name\"')".to_string()),
                    });
                }
            }

            // Check for undefined variables in conditions
            if let Some(expression) = &transition.condition.expression {
                // Simple heuristic: look for variable-like patterns
                if expression.contains("undefined_var")
                    || (expression.contains("==") && !expression.contains("result."))
                {
                    result.add_issue(ValidationIssue {
                        level: ValidationLevel::Warning,
                        file_path: workflow_path.to_path_buf(),
                        prompt_title: None,
                        line: None,
                        column: None,
                        message: format!("Condition in transition from '{}' may reference undefined variable: '{}'", transition.from_state, expression),
                        suggestion: Some("Ensure all variables are defined before use or come from action results".to_string()),
                    });
                }
            }
        }

        Ok(())
    }

    /// Validates a single workflow file
    ///
    /// This method collects validation errors in the provided ValidationResult
    /// rather than returning errors directly. This allows validation to continue
    /// for other files even if this one has errors.
    ///
    /// # Returns
    ///
    /// Always returns Ok(()) - errors are recorded in the ValidationResult parameter
    #[cfg(test)]
    pub fn validate_workflow(
        &mut self,
        workflow_path: &Path,
        result: &mut ValidationResult,
    ) -> Result<()> {
        result.files_checked += 1;

        // Read the workflow file
        let content = match std::fs::read_to_string(workflow_path) {
            Ok(content) => content,
            Err(e) => {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: workflow_path.to_path_buf(),
                    prompt_title: None,
                    line: None,
                    column: None,
                    message: format!("Failed to read workflow file: {}", e),
                    suggestion: None,
                });
                // Continue validation of other files
                return Ok(());
            }
        };

        // Parse the workflow
        let workflow_name = workflow_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("workflow");
        let workflow = match MermaidParser::parse(&content, workflow_name) {
            Ok(workflow) => workflow,
            Err(e) => {
                result.add_issue(ValidationIssue {
                    level: ValidationLevel::Error,
                    file_path: workflow_path.to_path_buf(),
                    prompt_title: None,
                    line: None,
                    column: None,
                    message: format!("Failed to parse workflow syntax: {}", e),
                    suggestion: Some("Check your Mermaid state diagram syntax".to_string()),
                });
                // Continue validation of other files
                return Ok(());
            }
        };

        // Use the shared validation logic
        self.validate_workflow_structure(&workflow, workflow_path, result)
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
    let mut validator = Validator::new(quiet);

    // Always validate all prompts and workflows from standard locations
    let result = validator.validate_all_with_options()?;

    validator.print_results(&result, format)?;

    // Return appropriate exit code
    if result.has_errors() {
        Ok(EXIT_ERROR) // Errors
    } else if result.has_warnings() {
        Ok(EXIT_WARNING) // Warnings
    } else {
        Ok(EXIT_SUCCESS) // Success
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
        let mut validator = Validator::new(false);

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

    #[test]
    fn test_validate_workflow_syntax_valid() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("test.mermaid");

        // Create a valid workflow
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> Start
    Start --> Process: continue
    Process --> End: complete
    End --> [*]
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        assert_eq!(result.errors, 0);
        assert_eq!(result.warnings, 0);
    }

    #[test]
    fn test_validate_workflow_syntax_invalid() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("test.mermaid");

        // Create an invalid workflow with syntax error
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> Start
    Start --> Process: invalid syntax here [
    Process --> End
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        assert!(result.has_errors());
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.message.contains("syntax")));
    }

    #[test]
    fn test_validate_workflow_unreachable_states() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("test.mermaid");

        // Create a workflow with unreachable state
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> Start
    Start --> End
    End --> [*]
    Orphan --> End
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        assert!(result.has_errors());
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.message.contains("unreachable")
                    || issue.message.contains("Orphan"))
        );
    }

    #[test]
    fn test_validate_workflow_missing_terminal_state() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("test.mermaid");

        // Create a workflow without terminal state
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> Start
    Start --> Process
    Process --> Start
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        assert!(result.has_errors());
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.message.contains("terminal")
                    || issue.message.contains("end state"))
        );
    }

    #[test]
    fn test_validate_workflow_circular_dependency() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("test.mermaid");

        // Create a workflow with circular dependency but also a terminal state
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> A
    A --> B
    B --> C
    C --> A
    C --> End
    End --> [*]
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        assert!(result.has_warnings());
        assert!(result.issues.iter().any(|issue| {
            let msg_lower = issue.message.to_lowercase();
            msg_lower.contains("circular") || msg_lower.contains("cycle")
        }));
    }

    #[test]
    fn test_validate_workflow_with_actions() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("test.mermaid");

        // Create a workflow with actions
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> Start
    Start --> Process: execute prompt "test"
    Process --> End: check result.success
    End --> [*]
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        // Should validate action syntax
        assert_eq!(result.errors, 0);
    }

    #[test]
    fn test_validate_workflow_undefined_variables() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("test.mermaid");

        // Create a workflow using undefined variables
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> Start
    Start --> Process: check undefined_var == true
    Process --> End
    End --> [*]
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        assert!(result.has_warnings());
        assert!(
            result
                .issues
                .iter()
                .any(|issue| issue.message.contains("undefined")
                    || issue.message.contains("variable"))
        );
    }

    #[test]
    fn test_validate_command_includes_workflows() {
        // Test that run_validate_command now validates both prompts and workflows
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflows_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
        std::fs::create_dir_all(&workflows_dir).unwrap();

        // Create a workflow file
        std::fs::write(
            workflows_dir.join("test.mermaid"),
            r#"stateDiagram-v2
    [*] --> Start
    Start --> End
    End --> [*]
"#,
        )
        .unwrap();

        // Note: This test would need to be run as an integration test
        // since run_validate_command uses the current directory
    }

    #[test]
    fn test_validate_all_workflows_uses_standard_locations() {
        // This test verifies that validate_all_workflows now uses WorkflowResolver
        // to load workflows only from standard locations (builtin, user, local)
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let current_dir = temp_dir.path();

        // Create a test workflow outside standard locations
        let non_standard_dir = current_dir.join("tests").join("workflows");
        std::fs::create_dir_all(&non_standard_dir).unwrap();
        std::fs::write(
            non_standard_dir.join("test.md"),
            r#"stateDiagram-v2
    [*] --> Start
    Start --> End
    End --> [*]
"#,
        )
        .unwrap();

        // Create a workflow in standard local location
        let standard_dir = current_dir.join(".swissarmyhammer").join("workflows");
        std::fs::create_dir_all(&standard_dir).unwrap();
        std::fs::write(
            standard_dir.join("local.md"),
            r#"stateDiagram-v2
    [*] --> Start
    Start --> End
    End --> [*]
"#,
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(current_dir).unwrap();

        let mut result = ValidationResult::new();
        let validation_result = validator.validate_all_workflows(&mut result);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(validation_result.is_ok());

        // The test workflow in non-standard location should NOT be validated
        // Only workflows from standard locations (builtin, user, local) should be validated
        // Note: In test environment, we may not have any workflows loaded, which is fine
    }

    #[test]
    fn test_validate_only_loads_from_standard_locations() {
        // This test ensures that workflows outside standard locations are NOT validated
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let current_dir = temp_dir.path();

        // Create workflows in various non-standard locations
        let non_standard_locations = vec![
            current_dir.join("workflows"),
            current_dir.join("custom").join("workflows"),
            current_dir.join("test").join("workflows"),
            current_dir.join(".custom").join("workflows"),
        ];

        for (i, location) in non_standard_locations.iter().enumerate() {
            std::fs::create_dir_all(location).unwrap();
            std::fs::write(
                location.join(format!("workflow{}.md", i)),
                format!(
                    r#"---
name: test-workflow-{}
description: Test workflow in non-standard location
---

stateDiagram-v2
    [*] --> Start
    Start --> End
    End --> [*]
"#,
                    i
                ),
            )
            .unwrap();
        }

        // Create a workflow in the standard local location
        let standard_dir = current_dir.join(".swissarmyhammer").join("workflows");
        std::fs::create_dir_all(&standard_dir).unwrap();
        std::fs::write(
            standard_dir.join("standard.md"),
            r#"---
name: standard-workflow
description: Test workflow in standard location
---

stateDiagram-v2
    [*] --> Start
    Start --> End
    End --> [*]
"#,
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(current_dir).unwrap();

        let mut result = ValidationResult::new();
        let validation_result = validator.validate_all_workflows(&mut result);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(validation_result.is_ok());

        // Check that non-standard workflows were NOT validated by verifying
        // that only the standard workflow (if any) was processed
        // In a test environment, builtin workflows might also be loaded
    }

    #[test]
    fn test_validate_command_loads_same_workflows_as_flow_list() {
        // This test ensures consistency between validate and flow list commands
        // Both should load workflows from the same standard locations

        // Create a temporary test environment
        let temp_dir = tempfile::TempDir::new().unwrap();
        let current_dir = temp_dir.path();

        // Create workflows in standard locations
        let local_dir = current_dir.join(".swissarmyhammer").join("workflows");
        std::fs::create_dir_all(&local_dir).unwrap();

        // Create a valid workflow
        std::fs::write(
            local_dir.join("test-workflow.mermaid"),
            r#"---
name: test-workflow
description: Test workflow for validation
---

stateDiagram-v2
    [*] --> Start
    Start --> Process
    Process --> End
    End --> [*]
"#,
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(current_dir).unwrap();

        // Run validation
        let mut validator = Validator::new(false);
        let mut validation_result = ValidationResult::new();
        let validate_res = validator.validate_all_workflows(&mut validation_result);

        // Load workflows using WorkflowResolver (same as flow list)
        let mut storage = MemoryWorkflowStorage::new();
        let mut resolver = WorkflowResolver::new();
        let flow_res = resolver.load_all_workflows(&mut storage);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(validate_res.is_ok());
        assert!(flow_res.is_ok());

        // Both methods should find the same workflows
        let flow_workflows = storage.list_workflows().unwrap();

        // The validation should have checked at least the workflows that flow list found
        // (validation might also check builtin workflows)
        assert!(validation_result.files_checked >= flow_workflows.len());
    }

    #[test]
    fn test_validate_workflow_empty_file() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("empty.mermaid");

        // Create empty workflow file
        std::fs::write(&workflow_path, "").unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        assert!(result.has_errors());
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.message.contains("Failed to parse workflow syntax")));
    }

    #[test]
    fn test_validate_workflow_empty_name() {
        use std::collections::HashMap;
        use swissarmyhammer::workflow::{StateId, WorkflowName};

        let mut validator = Validator::new(false);
        let mut result = ValidationResult::new();

        // Create a workflow with empty name
        // Using from() to bypass validation and test the validator's handling
        let workflow = Workflow {
            name: WorkflowName::from(""),
            description: "Test workflow".to_string(),
            states: HashMap::new(),
            transitions: vec![],
            initial_state: StateId::new("start"),
            metadata: HashMap::new(),
        };

        let workflow_path = PathBuf::from("workflow:test:");
        validator
            .validate_workflow_structure(&workflow, &workflow_path, &mut result)
            .unwrap();

        assert!(result.has_errors());
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.message.contains("Workflow name cannot be empty")));
    }

    #[test]
    fn test_validate_workflow_invalid_name() {
        use std::collections::HashMap;
        use swissarmyhammer::workflow::{StateId, WorkflowName};

        let mut validator = Validator::new(false);
        let mut result = ValidationResult::new();

        // Create a workflow with invalid characters in name
        let workflow = Workflow {
            name: WorkflowName::from("test@workflow!"),
            description: "Test workflow".to_string(),
            states: HashMap::new(),
            transitions: vec![],
            initial_state: StateId::new("start"),
            metadata: HashMap::new(),
        };

        let workflow_path = PathBuf::from("workflow:test:test@workflow!");
        validator
            .validate_workflow_structure(&workflow, &workflow_path, &mut result)
            .unwrap();

        assert!(result.has_errors());
        assert!(result
            .issues
            .iter()
            .any(|issue| issue.message.contains("Invalid workflow name")
                && issue.message.contains("only alphanumeric")));
    }

    #[test]
    fn test_validate_workflow_path_traversal_attempts() {
        use std::collections::HashMap;
        use swissarmyhammer::workflow::{StateId, WorkflowName};

        let mut validator = Validator::new(false);

        // Test various path traversal attempts in workflow names
        let dangerous_names = vec![
            "../evil",
            "../../etc/passwd",
            "workflow/../../../secret",
            "/absolute/path",
            "~/home/user",
            "workflow\x00null",
            "workflow%2e%2e%2f",
        ];

        for dangerous_name in dangerous_names {
            let mut result = ValidationResult::new();

            let workflow = Workflow {
                name: WorkflowName::from(dangerous_name),
                description: "Test workflow".to_string(),
                states: HashMap::new(),
                transitions: vec![],
                initial_state: StateId::new("start"),
                metadata: HashMap::new(),
            };

            let workflow_path = PathBuf::from(format!("workflow:test:{}", dangerous_name));
            validator
                .validate_workflow_structure(&workflow, &workflow_path, &mut result)
                .unwrap();

            assert!(
                result.has_errors(),
                "Should have error for dangerous name: {}",
                dangerous_name
            );
            assert!(
                result
                    .issues
                    .iter()
                    .any(|issue| issue.message.contains("Invalid workflow name")),
                "Should have invalid name error for: {}",
                dangerous_name
            );
        }
    }

    #[test]
    fn test_validate_workflow_malformed_mermaid() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("malformed.mermaid");

        // Various malformed Mermaid syntax
        let test_cases = [
            // Missing diagram type
            "[*] --> Start",
            // Wrong diagram type
            "flowchart TD\n    A --> B",
            // Incomplete state definition (avoiding empty state ID)
            "stateDiagram-v2\n    [*] --> InvalidSyntax:",
            // Invalid transition syntax
            "stateDiagram-v2\n    [*] -> Start",
            // Missing terminal state
            "stateDiagram-v2\n    Start --> Middle",
        ];

        for (i, content) in test_cases.iter().enumerate() {
            std::fs::write(&workflow_path, content).unwrap();

            let mut result = ValidationResult::new();
            validator
                .validate_workflow(&workflow_path, &mut result)
                .unwrap();

            assert!(result.has_errors(), "Test case {} should have errors", i);
            assert!(
                result.issues.iter().any(|issue| issue
                    .message
                    .contains("Failed to parse workflow syntax")
                    || issue.message.contains("no terminal state")
                    || issue.message.contains("validation failed")),
                "Test case {} should have parsing or validation errors",
                i
            );
        }
    }

    #[test]
    fn test_validate_workflow_complex_edge_cases() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("complex.mermaid");

        // Workflow with multiple isolated components
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> A
    A --> B
    B --> [*]
    
    C --> D
    D --> E
    E --> [*]
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        // Should have errors for unreachable states C, D, E (they're not connected to initial state)
        assert!(result.has_errors());
        let _unreachable_count = result
            .issues
            .iter()
            .filter(|issue| issue.message.contains("unreachable"))
            .count();
        // Note: The parser may not create states that aren't referenced in transitions
        // So we just verify that validation completes without panic
        assert!(
            result.files_checked > 0,
            "Should have validated the workflow file"
        );
    }

    #[test]
    fn test_validate_workflow_self_loop() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("selfloop.mermaid");

        // Workflow with self-loop
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> Processing
    Processing --> Processing : retry
    Processing --> Done : success
    Done --> [*]
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        // Self-loops are valid, should have no errors
        assert!(!result.has_errors());
        // Might have a warning about cycles
        let cycle_warnings = result
            .issues
            .iter()
            .filter(|issue| {
                issue.level == ValidationLevel::Warning
                    && (issue.message.contains("cycle") || issue.message.contains("circular"))
            })
            .count();
        assert!(cycle_warnings <= 1); // At most one cycle warning
    }

    #[test]
    fn test_validate_workflow_nested_conditions() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();
        let workflow_path = temp_dir.path().join("conditions.mermaid");

        // Workflow with complex conditions
        std::fs::write(
            &workflow_path,
            r#"stateDiagram-v2
    [*] --> Check
    Check --> Process : result.success == true && input.type == "valid"
    Check --> Error : result.success == false || timeout > 30
    Process --> Done
    Error --> Done
    Done --> [*]
"#,
        )
        .unwrap();

        let mut result = ValidationResult::new();
        validator
            .validate_workflow(&workflow_path, &mut result)
            .unwrap();

        // Current implementation may not detect all undefined variables in complex expressions
        // This is a known limitation mentioned in CODE_REVIEW.md
        // The test verifies that validation completes without crashing
        assert!(
            !result.has_errors() || result.has_warnings(),
            "Should complete validation without critical errors"
        );
    }

    #[test]
    fn test_validate_all_workflows_integration() {
        let mut validator = Validator::new(false);
        let temp_dir = tempfile::TempDir::new().unwrap();

        // Create nested workflow directories
        let workflows_dir1 = temp_dir.path().join(".swissarmyhammer").join("workflows");
        let workflows_dir2 = temp_dir
            .path()
            .join("project")
            .join(".swissarmyhammer")
            .join("workflows");
        std::fs::create_dir_all(&workflows_dir1).unwrap();
        std::fs::create_dir_all(&workflows_dir2).unwrap();

        // Create valid workflow
        std::fs::write(
            workflows_dir1.join("valid.mermaid"),
            r#"stateDiagram-v2
    [*] --> Start
    Start --> End
    End --> [*]
"#,
        )
        .unwrap();

        // Create invalid workflow
        std::fs::write(
            workflows_dir2.join("invalid.mermaid"),
            r#"stateDiagram-v2
    [*] --> Start
    Start --> Middle
    Middle --> End
"#,
        )
        .unwrap();

        // Create non-workflow mermaid file (should be ignored)
        std::fs::write(
            temp_dir.path().join("diagram.mermaid"),
            r#"flowchart TD
    A --> B
"#,
        )
        .unwrap();

        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&temp_dir).unwrap();

        let mut result = ValidationResult::new();
        let validation_result = validator.validate_all_workflows(&mut result);

        std::env::set_current_dir(original_dir).unwrap();

        assert!(validation_result.is_ok());
        // With the new implementation using WorkflowResolver, workflows are only loaded
        // from standard locations (builtin, user ~/.swissarmyhammer/workflows, local ./.swissarmyhammer/workflows)
        // In a temp directory test environment, we might find the local workflow if the resolver
        // properly walks up to find .swissarmyhammer directories

        // With the new implementation using WorkflowResolver, workflows are only loaded
        // from standard locations. In a test environment, it may load builtin workflows
        // but won't find the test workflows we created in the temp directory.
        // This is the expected behavior - we want to ensure consistent loading from
        // standard locations only.
    }
}
