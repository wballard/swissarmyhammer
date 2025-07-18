//! Request and response types for MCP operations, along with constants

use serde::Deserialize;
use std::collections::HashMap;
use crate::config::Config;

// Type safety wrapper types

/// A wrapper type for issue numbers to prevent mixing up different ID types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct IssueNumber(pub u32);

impl IssueNumber {
    /// Create a new issue number after validation
    pub fn new(number: u32) -> Result<Self, String> {
        let config = Config::global();
        if !(config.min_issue_number..=config.max_issue_number).contains(&number) {
            return Err(format!(
                "Issue number {} is out of valid range ({}-{})",
                number, config.min_issue_number, config.max_issue_number
            ));
        }
        Ok(IssueNumber(number))
    }

    /// Get the inner u32 value
    pub fn get(&self) -> u32 {
        self.0
    }

    /// Create from u32 without validation (use with caution)
    pub fn from_u32_unchecked(number: u32) -> Self {
        IssueNumber(number)
    }
}

impl std::fmt::Display for IssueNumber {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let config = Config::global();
        write!(f, "{:0width$}", self.0, width = config.issue_number_digits)
    }
}

impl From<IssueNumber> for u32 {
    fn from(issue_number: IssueNumber) -> Self {
        issue_number.0
    }
}

/// A wrapper type for issue names to prevent mixing up different string types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Deserialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct IssueName(pub String);

impl IssueName {
    /// Create a new issue name after validation
    pub fn new(name: String) -> Result<Self, String> {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            return Err("Issue name cannot be empty".to_string());
        }
        if trimmed.len() > 100 {
            return Err("Issue name cannot exceed 100 characters".to_string());
        }
        
        // Check for invalid characters
        if trimmed.contains('/') || trimmed.contains('\\') || trimmed.contains('\0') {
            return Err("Issue name contains invalid characters".to_string());
        }
        
        Ok(IssueName(trimmed.to_string()))
    }

    /// Get the inner string value
    pub fn get(&self) -> &str {
        &self.0
    }

    /// Create from string without validation (use with caution)
    pub fn from_string_unchecked(name: String) -> Self {
        IssueName(name)
    }
}

impl std::fmt::Display for IssueName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<IssueName> for String {
    fn from(issue_name: IssueName) -> Self {
        issue_name.0
    }
}

impl AsRef<str> for IssueName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Request structure for getting a prompt
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct GetPromptRequest {
    /// Name of the prompt to retrieve
    pub name: String,
    /// Optional arguments for template rendering
    #[serde(default)]
    pub arguments: HashMap<String, String>,
}

/// Request structure for listing prompts
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct ListPromptsRequest {
    /// Optional filter by category
    pub category: Option<String>,
}

/// Request to create a new issue
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CreateIssueRequest {
    /// Name of the issue (will be used in filename)
    pub name: IssueName,
    /// Markdown content of the issue
    pub content: String,
}

/// Request to mark an issue as complete
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarkCompleteRequest {
    /// Issue number to mark as complete
    pub number: IssueNumber,
}

/// Request to check if all issues are complete
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct AllCompleteRequest {
    // No parameters needed
}

/// Request to update an issue
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct UpdateIssueRequest {
    /// Issue number to update
    pub number: IssueNumber,
    /// New markdown content for the issue
    pub content: String,
}

/// Request to get current issue
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct CurrentIssueRequest {
    /// Which branch to check (optional, defaults to current)
    pub branch: Option<String>,
}

/// Request to work on an issue
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct WorkIssueRequest {
    /// Issue number to work on
    pub number: IssueNumber,
}

/// Request to merge an issue
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeIssueRequest {
    /// Issue number to merge
    pub number: IssueNumber,
    /// Whether to delete the branch after merging (default: false)
    #[serde(default)]
    pub delete_branch: bool,
}
