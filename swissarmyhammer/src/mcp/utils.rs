//! Utility functions for MCP operations

use rmcp::Error as McpError;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// Convert a JSON map to a string map for template arguments
pub fn convert_prompt_arguments(arguments: &HashMap<String, Value>) -> HashMap<String, String> {
    arguments
        .iter()
        .map(|(k, v)| {
            let value_str = match v {
                Value::String(s) => s.clone(),
                _ => v.to_string(),
            };
            (k.clone(), value_str)
        })
        .collect()
}

/// Convert a JSON map to a string map
pub fn json_map_to_string_map(
    json_map: &serde_json::Map<String, Value>,
) -> HashMap<String, String> {
    json_map
        .iter()
        .map(|(k, v)| {
            let value_str = match v {
                Value::String(s) => s.clone(),
                _ => v.to_string(),
            };
            (k.clone(), value_str)
        })
        .collect()
}

/// Generate a JSON schema for a type that implements JsonSchema
pub fn generate_tool_schema<T>() -> Arc<serde_json::Map<String, Value>>
where
    T: schemars::JsonSchema,
{
    serde_json::to_value(schemars::schema_for!(T))
        .ok()
        .and_then(|v| v.as_object().map(|obj| Arc::new(obj.clone())))
        .unwrap_or_else(|| Arc::new(serde_json::Map::new()))
}

/// Validate and normalize an issue name according to MCP standards
///
/// This function performs comprehensive validation including:
/// - Empty/whitespace checks
/// - Length limits (max 100 characters)
/// - Invalid filesystem character checks
/// - Additional validation using the existing issues module
///
/// # Arguments
///
/// * `name` - The raw issue name to validate
///
/// # Returns
///
/// * `Result<String, McpError>` - The validated and trimmed name, or an error
pub fn validate_issue_name(name: &str) -> std::result::Result<String, McpError> {
    use crate::issues::validate_issue_name as validate_issue_name_internal;

    let trimmed = name.trim();

    if trimmed.is_empty() {
        return Err(McpError::invalid_params("Issue name cannot be empty", None));
    }

    if trimmed.len() > 100 {
        return Err(McpError::invalid_params(
            "Issue name too long (max 100 characters)",
            None,
        ));
    }

    // Check for invalid characters
    if trimmed.contains(['/', '\\', ':', '*', '?', '"', '<', '>', '|']) {
        return Err(McpError::invalid_params(
            "Issue name contains invalid characters",
            None,
        ));
    }

    // Use the existing validation function for additional checks
    validate_issue_name_internal(trimmed)
        .map_err(|e| McpError::invalid_params(format!("Invalid issue name: {}", e), None))?;

    Ok(trimmed.to_string())
}

/// Validate issue content comprehensively according to MCP standards
///
/// This function validates that issue content doesn't exceed size limits,
/// contains safe markdown, and doesn't include potentially dangerous content.
///
/// # Arguments
///
/// * `content` - The issue content to validate
///
/// # Returns
///
/// * `Result<(), McpError>` - Success or validation error
pub fn validate_issue_content_size(content: &str) -> std::result::Result<(), McpError> {
    // Configurable limits via environment variables with sensible defaults
    let max_content_size = std::env::var("SWISSARMYHAMMER_MAX_CONTENT_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(1024 * 1024); // 1MB default

    let max_content_lines = std::env::var("SWISSARMYHAMMER_MAX_CONTENT_LINES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(10000); // 10k lines default

    // Check content size in bytes
    if content.len() > max_content_size {
        return Err(McpError::invalid_params(
            format!(
                "Issue content too large: {} bytes (max {} bytes)",
                content.len(),
                max_content_size
            ),
            None,
        ));
    }

    // Check line count
    let line_count = content.lines().count();
    if line_count > max_content_lines {
        return Err(McpError::invalid_params(
            format!(
                "Issue content has too many lines: {} lines (max {} lines)",
                line_count, max_content_lines
            ),
            None,
        ));
    }

    // Validate against control characters (except common ones like tabs and newlines)
    for (line_num, line) in content.lines().enumerate() {
        for c in line.chars() {
            if c.is_control() && c != '\t' && c != '\n' && c != '\r' {
                return Err(McpError::invalid_params(
                    format!(
                        "Issue content contains invalid control characters on line {}: '{}'",
                        line_num + 1,
                        line.chars()
                            .map(|c| if c.is_control() { '�' } else { c })
                            .collect::<String>()
                    ),
                    None,
                ));
            }
        }
    }

    // Check for potentially dangerous HTML tags/XSS vectors
    validate_html_security(content)?;

    // Validate markdown structure
    validate_markdown_structure(content)?;

    Ok(())
}

/// Validate content against potential XSS vectors and dangerous HTML
/// This provides comprehensive HTML sanitization using regex patterns and context-aware validation
fn validate_html_security(content: &str) -> std::result::Result<(), McpError> {
    use regex::Regex;
    
    // Pattern for dangerous HTML tags (more comprehensive than simple string matching)
    static DANGEROUS_TAG_PATTERNS: &[&str] = &[
        r"<\s*script[^>]*>",
        r"<\s*iframe[^>]*>", 
        r"<\s*object[^>]*>",
        r"<\s*embed[^>]*>",
        r"<\s*link[^>]*>",
        r"<\s*style[^>]*>",
        r"<\s*meta[^>]*>",
        r"<\s*base[^>]*>",
        r"<\s*form[^>]*>",
        r"<\s*input[^>]*>",
        r"<\s*button[^>]*>",
        r"<\s*svg[^>]*>",
        r"<\s*math[^>]*>",
        r"<\s*details[^>]*>",
        r"<\s*dialog[^>]*>",
    ];

    // Pattern for dangerous protocols
    static DANGEROUS_PROTOCOLS: &[&str] = &[
        r"javascript\s*:",
        r"data\s*:",
        r"vbscript\s*:",
        r"file\s*:",
        r"ftp\s*:",
    ];

    // Pattern for event handlers (more comprehensive)
    static EVENT_HANDLER_PATTERNS: &[&str] = &[
        r"on\w+\s*=",
        r"@\w+\s*=", // Vue.js style events
        r"ng-\w+\s*=", // Angular style events
    ];

    // Pattern for dangerous attributes
    static DANGEROUS_ATTRIBUTE_PATTERNS: &[&str] = &[
        r"srcdoc\s*=",
        r"formaction\s*=",
        r"action\s*=",
        r"background\s*=",
        r"poster\s*=",
    ];

    let content_lower = content.to_lowercase();

    // Check for dangerous HTML tags
    for pattern in DANGEROUS_TAG_PATTERNS {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(&content_lower) {
                return Err(McpError::invalid_params(
                    format!(
                        "Issue content contains potentially dangerous HTML tag matching pattern: '{}'",
                        pattern
                    ),
                    None,
                ));
            }
        }
    }

    // Check for dangerous protocols
    for pattern in DANGEROUS_PROTOCOLS {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(&content_lower) {
                return Err(McpError::invalid_params(
                    format!(
                        "Issue content contains potentially dangerous protocol: '{}'",
                        pattern
                    ),
                    None,
                ));
            }
        }
    }

    // Check for event handlers
    for pattern in EVENT_HANDLER_PATTERNS {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(&content_lower) {
                return Err(McpError::invalid_params(
                    format!(
                        "Issue content contains potentially dangerous event handler: '{}'",
                        pattern
                    ),
                    None,
                ));
            }
        }
    }

    // Check for dangerous attributes
    for pattern in DANGEROUS_ATTRIBUTE_PATTERNS {
        if let Ok(regex) = Regex::new(pattern) {
            if regex.is_match(&content_lower) {
                return Err(McpError::invalid_params(
                    format!(
                        "Issue content contains potentially dangerous attribute: '{}'",
                        pattern
                    ),
                    None,
                ));
            }
        }
    }

    // Additional validation for encoded content
    validate_encoded_content(&content_lower)?;

    Ok(())
}

/// Validate against encoded malicious content
fn validate_encoded_content(content: &str) -> std::result::Result<(), McpError> {
    // Check for HTML entities that could be used to bypass validation
    let suspicious_entities = [
        "&#x6a;&#x61;&#x76;&#x61;&#x73;&#x63;&#x72;&#x69;&#x70;&#x74;", // javascript
        "&#106;&#97;&#118;&#97;&#115;&#99;&#114;&#105;&#112;&#116;", // javascript
        "&lt;script", // encoded script tags
        "&lt;iframe",
        "&lt;object",
        "%3cscript", // URL encoded script
        "%3ciframe",
        "%3cobject",
    ];

    for entity in &suspicious_entities {
        if content.contains(entity) {
            return Err(McpError::invalid_params(
                format!(
                    "Issue content contains potentially dangerous encoded content: '{}'",
                    entity
                ),
                None,
            ));
        }
    }

    Ok(())
}

/// Validate basic markdown structure
fn validate_markdown_structure(content: &str) -> std::result::Result<(), McpError> {
    // Check for balanced code blocks
    let mut code_block_count = 0;
    let mut in_code_block = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Check for code block markers
        if trimmed.starts_with("```") {
            if in_code_block {
                code_block_count -= 1;
                in_code_block = false;
            } else {
                code_block_count += 1;
                in_code_block = true;
            }
        }
    }

    if code_block_count > 0 {
        return Err(McpError::invalid_params(
            format!(
                "Issue content has {} unmatched code blocks (```). Each opening ``` must have a closing ```",
                code_block_count
            ),
            None,
        ));
    }

    Ok(())
}
