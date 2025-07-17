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

/// Validate issue content size according to MCP standards
///
/// This function validates that issue content doesn't exceed size limits
/// to prevent memory issues and ensure reasonable issue sizes.
///
/// # Arguments
///
/// * `content` - The issue content to validate
///
/// # Returns
///
/// * `Result<(), McpError>` - Success or validation error
pub fn validate_issue_content_size(content: &str) -> std::result::Result<(), McpError> {
    const MAX_CONTENT_SIZE: usize = 1024 * 1024; // 1MB limit
    const MAX_CONTENT_LINES: usize = 10000; // 10k lines limit

    // Check content size in bytes
    if content.len() > MAX_CONTENT_SIZE {
        return Err(McpError::invalid_params(
            format!(
                "Issue content too large: {} bytes (max {} bytes / 1MB)",
                content.len(),
                MAX_CONTENT_SIZE
            ),
            None,
        ));
    }

    // Check line count
    let line_count = content.lines().count();
    if line_count > MAX_CONTENT_LINES {
        return Err(McpError::invalid_params(
            format!(
                "Issue content has too many lines: {} lines (max {} lines)",
                line_count, MAX_CONTENT_LINES
            ),
            None,
        ));
    }

    Ok(())
}
