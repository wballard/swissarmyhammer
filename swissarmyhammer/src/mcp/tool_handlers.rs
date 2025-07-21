//! Tool handlers for MCP operations

use super::responses::{
    create_error_response, create_issue_response, create_mark_complete_response,
    create_success_response,
};
use super::types::*;
use super::utils::validate_issue_name;
use crate::config::Config;
use crate::git::GitOperations;
use crate::issues::{Issue, IssueStorage};
use crate::Result;
use rmcp::model::*;
use rmcp::Error as McpError;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Tool handlers for MCP server operations
pub struct ToolHandlers {
    issue_storage: Arc<RwLock<Box<dyn IssueStorage>>>,
    git_ops: Arc<Mutex<Option<GitOperations>>>,
}

impl ToolHandlers {
    /// Create a new tool handlers instance with the given issue storage and git operations
    pub fn new(
        issue_storage: Arc<RwLock<Box<dyn IssueStorage>>>,
        git_ops: Arc<Mutex<Option<GitOperations>>>,
    ) -> Self {
        Self {
            issue_storage,
            git_ops,
        }
    }

    /// Handle the issue_create tool operation.
    ///
    /// Creates a new issue with auto-assigned number and stores it in the
    /// issues directory as a markdown file.
    ///
    /// # Arguments
    ///
    /// * `request` - The create issue request containing name and content
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_issue_create(
        &self,
        request: CreateIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::debug!("Creating issue: {:?}", request.name);

        // Validate issue name using shared validation logic, or use empty string for nameless issues
        let validated_name = match &request.name {
            Some(name) => validate_issue_name(name.as_str())?,
            None => String::new(), // Empty name for nameless issues - skip validation
        };

        let issue_storage = self.issue_storage.write().await;
        match issue_storage
            .create_issue(validated_name, request.content)
            .await
        {
            Ok(issue) => {
                tracing::info!("Created issue {}", issue.name);
                Ok(create_issue_response(&issue))
            }
            Err(crate::SwissArmyHammerError::IssueAlreadyExists(num)) => {
                tracing::warn!("Issue #{:06} already exists", num);
                Err(McpError::invalid_params(
                    format!("Issue #{num:06} already exists"),
                    None,
                ))
            }
            Err(e) => {
                tracing::error!("Failed to create issue: {}", e);
                Err(McpError::internal_error(
                    format!("Failed to create issue: {e}"),
                    None,
                ))
            }
        }
    }

    /// Handle the issue_mark_complete tool operation.
    ///
    /// Marks an issue as complete by moving it to the completed issues directory.
    ///
    /// # Arguments
    ///
    /// * `request` - The mark complete request containing the issue number
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_issue_mark_complete(
        &self,
        request: MarkCompleteRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let issue_storage = self.issue_storage.write().await;
        match issue_storage.mark_complete(request.name.as_str()).await {
            Ok(issue) => Ok(create_mark_complete_response(&issue)),
            Err(e) => Ok(create_error_response(format!(
                "Failed to mark issue complete: {e}"
            ))),
        }
    }

    /// Handle the issue_all_complete tool operation.
    ///
    /// Provides comprehensive project status including all issues, completion statistics,
    /// and detailed insights for AI assistants to understand project health.
    ///
    /// # Arguments
    ///
    /// * `_request` - The all complete request (no parameters needed)
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result with comprehensive status
    pub async fn handle_issue_all_complete(
        &self,
        _request: AllCompleteRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let issue_storage = self.issue_storage.read().await;

        // Get all issues with comprehensive error handling
        let all_issues = match issue_storage.list_issues().await {
            Ok(issues) => issues,
            Err(e) => {
                let error_msg = match e.to_string() {
                    msg if msg.contains("permission") => {
                        "Permission denied: Unable to read issues directory. Check directory permissions.".to_string()
                    }
                    msg if msg.contains("No such file") => {
                        "Issues directory not found. The project may not have issue tracking initialized.".to_string()
                    }
                    _ => {
                        format!("Failed to check issue status: {e}")
                    }
                };

                return Ok(CallToolResult {
                    content: vec![Annotated::new(
                        RawContent::Text(RawTextContent {
                            text: error_msg.clone(),
                        }),
                        None,
                    )],
                    is_error: Some(true),
                });
            }
        };

        // Separate active and completed issues
        let mut active_issues = Vec::new();
        let mut completed_issues = Vec::new();

        for issue in all_issues {
            if issue.completed {
                completed_issues.push(issue);
            } else {
                active_issues.push(issue);
            }
        }

        // Calculate statistics
        let total_issues = active_issues.len() + completed_issues.len();
        let completed_count = completed_issues.len();
        let active_count = active_issues.len();
        let all_complete = active_count == 0 && total_issues > 0;

        let completion_percentage = if total_issues > 0 {
            (completed_count * 100) / total_issues
        } else {
            0
        };

        // Generate comprehensive response text
        let response_text = if total_issues == 0 {
            "📋 No issues found in the project\n\n✨ The project has no tracked issues. You can create issues using the `issue_create` tool.".to_string()
        } else if all_complete {
            format!(
                "🎉 All issues are complete!\n\n📊 Project Status:\n• Total Issues: {}\n• Completed: {} (100%)\n• Active: 0\n\n✅ Completed Issues:\n{}",
                total_issues,
                completed_count,
                completed_issues.iter()
                    .map(|issue| format!("• {}", issue.name))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        } else {
            let active_list = active_issues
                .iter()
                .map(|issue| format!("• {}", issue.name))
                .collect::<Vec<_>>()
                .join("\n");

            let completed_list = if completed_count > 0 {
                completed_issues
                    .iter()
                    .map(|issue| format!("• {}", issue.name))
                    .collect::<Vec<_>>()
                    .join("\n")
            } else {
                "  (none)".to_string()
            };

            format!(
                "⏳ Project has active issues ({completion_percentage}% complete)\n\n📊 Project Status:\n• Total Issues: {total_issues}\n• Completed: {completed_count} ({completion_percentage}%)\n• Active: {active_count}\n\n🔄 Active Issues:\n{active_list}\n\n✅ Completed Issues:\n{completed_list}"
            )
        };

        // Create comprehensive artifact with detailed data
        let _artifact = serde_json::json!({
            "action": "all_complete",
            "status": "success",
            "all_complete": all_complete,
            "statistics": {
                "total_issues": total_issues,
                "completed_count": completed_count,
                "active_count": active_count,
                "completion_percentage": completion_percentage
            },
            "issues": {
                "active": active_issues.iter().map(|issue| {
                    serde_json::json!({
                        "name": issue.name,
                        "file_path": issue.file_path.to_string_lossy()
                    })
                }).collect::<Vec<_>>(),
                "completed": completed_issues.iter().map(|issue| {
                    serde_json::json!({
                        "name": issue.name,
                        "file_path": issue.file_path.to_string_lossy()
                    })
                }).collect::<Vec<_>>()
            }
        });

        Ok(CallToolResult {
            content: vec![Annotated::new(
                RawContent::Text(RawTextContent {
                    text: response_text,
                }),
                None,
            )],
            is_error: Some(false),
        })
    }

    /// Handle the issue_update tool operation.
    ///
    /// Updates the content of an existing issue with new markdown content.
    ///
    /// # Arguments
    ///
    /// * `request` - The update request containing issue number and new content
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_issue_update(
        &self,
        request: UpdateIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let issue_storage = self.issue_storage.write().await;
        match issue_storage
            .update_issue(request.name.as_str(), request.content)
            .await
        {
            Ok(issue) => Ok(create_success_response(format!(
                "Updated issue {}",
                issue.name
            ))),
            Err(e) => Ok(create_error_response(format!(
                "Failed to update issue: {e}"
            ))),
        }
    }

    /// Handle the issue_current tool operation.
    ///
    /// Determines the current issue being worked on by checking the git branch name.
    /// If on an issue branch (starts with 'issue/'), returns the issue name.
    ///
    /// # Arguments
    ///
    /// * `_request` - The current issue request (no parameters needed)
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result with current issue info
    pub async fn handle_issue_current(
        &self,
        _request: CurrentIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let git_ops = self.git_ops.lock().await;
        match git_ops.as_ref() {
            Some(ops) => match ops.current_branch() {
                Ok(branch) => {
                    let config = Config::global();
                    if let Some(issue_name) = branch.strip_prefix(&config.issue_branch_prefix) {
                        Ok(create_success_response(format!(
                            "Currently working on issue: {issue_name}"
                        )))
                    } else {
                        Ok(create_success_response(format!(
                            "Not on an issue branch. Current branch: {branch}"
                        )))
                    }
                }
                Err(e) => Ok(create_error_response(format!(
                    "Failed to get current branch: {e}"
                ))),
            },
            None => Ok(create_error_response(
                "Git operations not available".to_string(),
            )),
        }
    }

    /// Handle the issue_work tool operation.
    ///
    /// Switches to a work branch for the specified issue. Creates a new branch
    /// with the format 'issue/{issue_number}_{issue_name}' if it doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `request` - The work request containing the issue number
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_issue_work(
        &self,
        request: WorkIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        // Get the issue to determine its number for branch naming
        let issue = match self.get_issue_or_error(&request.name).await {
            Ok(issue) => issue,
            Err(error_response) => return Ok(error_response),
        };

        // Create work branch with format: number_name
        let mut git_ops = self.git_ops.lock().await;
        let branch_name = format!("issue/{}", issue.name);

        match git_ops.as_mut() {
            Some(ops) => match ops.create_work_branch(&branch_name) {
                Ok(branch_name) => Ok(create_success_response(format!(
                    "Switched to work branch: {branch_name}"
                ))),
                Err(e) => {
                    // Check if this is an ABORT ERROR - if so, return it directly
                    if e.is_abort_error() {
                        let error_msg = e.abort_error_message().unwrap_or_else(|| e.to_string());
                        Ok(create_error_response(error_msg))
                    } else {
                        Ok(create_error_response(format!(
                            "Failed to create work branch: {e}"
                        )))
                    }
                }
            },
            None => Ok(create_error_response(
                "Git operations not available".to_string(),
            )),
        }
    }

    /// Handle the issue_merge tool operation.
    ///
    /// Merges the work branch for an issue back to the main branch.
    /// The branch name is determined from the issue number and name.
    ///
    /// # Arguments
    ///
    /// * `request` - The merge request containing the issue number
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_issue_merge(
        &self,
        request: MergeIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        // Get the issue to determine its details
        let issue = match self.get_issue_or_error(&request.name).await {
            Ok(issue) => issue,
            Err(error_response) => return Ok(error_response),
        };

        // Validate that the issue is completed before allowing merge
        if !issue.completed {
            return Ok(create_error_response(format!(
                "Issue '{}' must be completed before merging",
                request.name
            )));
        }

        // Check working directory is clean before merge
        let git_ops_guard = self.git_ops.lock().await;
        if let Some(git_ops) = git_ops_guard.as_ref() {
            if let Err(e) = self.check_working_directory_clean(git_ops).await {
                return Ok(create_error_response(format!(
                    "Working directory is not clean. Please commit or stash changes before merging: {e}"
                )));
            }
        }
        drop(git_ops_guard);

        // Merge branch
        let mut git_ops = self.git_ops.lock().await;
        let issue_name = issue.name.clone();

        match git_ops.as_mut() {
            Some(ops) => {
                // First merge the branch
                match ops.merge_issue_branch(&issue_name) {
                    Ok(_) => {
                        let mut success_message =
                            format!("Merged work branch for issue {issue_name} to main");

                        // Get commit information after successful merge
                        let commit_info = match ops.get_last_commit_info() {
                            Ok(info) => {
                                let parts: Vec<&str> = info.split('|').collect();
                                if parts.len() >= 4 {
                                    format!(
                                        "\n\nMerge commit: {}\nMessage: {}\nAuthor: {}\nDate: {}",
                                        &parts[0][..8], // First 8 chars of hash
                                        parts[1],
                                        parts[2],
                                        parts[3]
                                    )
                                } else {
                                    format!("\n\nMerge commit: {info}")
                                }
                            }
                            Err(_) => String::new(),
                        };

                        // If delete_branch is true, delete the branch after successful merge
                        if request.delete_branch {
                            let branch_name = Self::format_issue_branch_name(&issue_name);
                            match ops.delete_branch(&branch_name) {
                                Ok(_) => {
                                    success_message
                                        .push_str(&format!(" and deleted branch {branch_name}"));
                                }
                                Err(e) => {
                                    success_message
                                        .push_str(&format!(" but failed to delete branch: {e}"));
                                }
                            }
                        }

                        success_message.push_str(&commit_info);
                        Ok(create_success_response(success_message))
                    }
                    Err(e) => Ok(create_error_response(format!(
                        "Failed to merge branch: {e}"
                    ))),
                }
            }
            None => Ok(create_error_response(
                "Git operations not available".to_string(),
            )),
        }
    }

    /// Check if working directory is clean
    async fn check_working_directory_clean(&self, _git_ops: &GitOperations) -> Result<()> {
        use std::process::Command;

        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .output()
            .map_err(|e| {
                crate::SwissArmyHammerError::git_operation_failed(
                    "git status check",
                    &e.to_string(),
                )
            })?;

        let status = String::from_utf8_lossy(&output.stdout);

        if !status.trim().is_empty() {
            return Err(crate::SwissArmyHammerError::Other(
                "Working directory is not clean - there are uncommitted changes".to_string(),
            ));
        }

        Ok(())
    }

    /// Helper method to get an issue and handle errors consistently
    async fn get_issue_or_error(
        &self,
        issue_name: &IssueName,
    ) -> std::result::Result<Issue, CallToolResult> {
        let issue_storage = self.issue_storage.read().await;
        match issue_storage.get_issue(issue_name.as_str()).await {
            Ok(issue) => {
                drop(issue_storage);
                Ok(issue)
            }
            Err(e) => {
                drop(issue_storage);
                Err(create_error_response(format!(
                    "Failed to get issue '{issue_name}': {e}"
                )))
            }
        }
    }


    /// Helper method to format issue branch names consistently
    fn format_issue_branch_name(issue_name: &str) -> String {
        format!("issue/{issue_name}")
    }
}
