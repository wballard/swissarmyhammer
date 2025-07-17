//! Request and response types for MCP operations

use serde::Deserialize;
use std::collections::HashMap;

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
    pub name: String,
    /// Markdown content of the issue
    pub content: String,
}

/// Request to mark an issue as complete
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MarkCompleteRequest {
    /// Issue number to mark as complete
    pub number: u32,
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
    pub number: u32,
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
    pub number: u32,
}

/// Request to merge an issue
#[derive(Debug, Deserialize, schemars::JsonSchema)]
pub struct MergeIssueRequest {
    /// Issue number to merge
    pub number: u32,
    /// Whether to delete the branch after merging (default: false)
    #[serde(default)]
    pub delete_branch: bool,
}
