//! Tool handlers for MCP operations

use super::memo_types::*;
use super::responses::{
    create_error_response, create_issue_response, create_mark_complete_response,
    create_success_response,
};
use super::shared_utils::{McpErrorHandler, McpFormatter, McpValidation};
use super::types::*;
use super::utils::validate_issue_name;
#[cfg(not(test))]
use crate::common::rate_limiter::get_rate_limiter;
use crate::config::Config;
use crate::git::GitOperations;
use crate::issues::{Issue, IssueStorage};
use crate::memoranda::{MemoId, MemoStorage};
use rmcp::model::*;
use rmcp::Error as McpError;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Preview length for memo list operations (characters)
const MEMO_LIST_PREVIEW_LENGTH: usize = 100;

/// Preview length for memo search operations (characters)
const MEMO_SEARCH_PREVIEW_LENGTH: usize = 200;

/// Tool handlers for MCP server operations
#[derive(Clone)]
pub struct ToolHandlers {
    issue_storage: Arc<RwLock<Box<dyn IssueStorage>>>,
    git_ops: Arc<Mutex<Option<GitOperations>>>,
    memo_storage: Arc<RwLock<Box<dyn MemoStorage>>>,
}

impl ToolHandlers {
    /// Create a new tool handlers instance with the given issue storage, git operations, and memo storage
    pub fn new(
        issue_storage: Arc<RwLock<Box<dyn IssueStorage>>>,
        git_ops: Arc<Mutex<Option<GitOperations>>>,
        memo_storage: Arc<RwLock<Box<dyn MemoStorage>>>,
    ) -> Self {
        Self {
            issue_storage,
            git_ops,
            memo_storage,
        }
    }

    /// Format a memo preview with consistent formatting
    ///
    /// Creates a standardized preview format showing title, ID, timestamps, and content preview.
    ///
    /// # Arguments
    ///
    /// * `memo` - The memo to format
    /// * `preview_length` - Number of characters to include in content preview
    ///
    /// # Returns
    ///
    /// * `String` - Formatted memo preview
    fn format_memo_preview(memo: &crate::memoranda::Memo, preview_length: usize) -> String {
        format!(
            "• {} ({})\n  Created: {}\n  Updated: {}\n  Preview: {}",
            memo.title,
            memo.id,
            McpFormatter::format_timestamp(memo.created_at),
            McpFormatter::format_timestamp(memo.updated_at),
            McpFormatter::format_preview(&memo.content, preview_length)
        )
    }

    /// Check rate limit for an MCP operation
    ///
    /// Applies rate limiting to prevent DoS attacks. Uses client identification
    /// based on request context when available, falls back to a default client ID.
    ///
    /// # Arguments
    ///
    /// * `operation` - The operation being performed
    /// * `cost` - Token cost of the operation (default: 1, expensive: 2-5)
    /// * `client_id` - Optional client identifier (falls back to "unknown")
    ///
    /// # Returns
    ///
    /// * `Result<(), McpError>` - Ok if operation is allowed, error if rate limited
    fn check_rate_limit(
        &self,
        _operation: &str,
        _cost: u32,
        _client_id: Option<&str>,
    ) -> std::result::Result<(), McpError> {
        // Skip rate limiting in test environment
        #[cfg(test)]
        {
            return Ok(());
        }

        #[cfg(not(test))]
        {
            let client = _client_id.unwrap_or("unknown");
            let rate_limiter = get_rate_limiter();

            rate_limiter
                .check_rate_limit(client, _operation, _cost)
                .map_err(|e| {
                    tracing::warn!(
                        "Rate limit exceeded for client '{}', operation '{}': {}",
                        client,
                        _operation,
                        e
                    );
                    McpError::invalid_params(e.to_string(), None)
                })
        }
    }

    /// Handle memo operation errors consistently based on error type
    ///
    /// Uses the shared error handler for consistent error mapping across all MCP operations.
    ///
    /// # Arguments
    ///
    /// * `error` - The SwissArmyHammerError to handle
    /// * `operation` - Description of the operation that failed (for logging)
    ///
    /// # Returns
    ///
    /// * `McpError` - Appropriate MCP error response
    fn handle_memo_error(error: crate::error::SwissArmyHammerError, operation: &str) -> McpError {
        McpErrorHandler::handle_error(error, operation)
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
        // Apply rate limiting for issue creation
        self.check_rate_limit("issue_create", 1, None)?;

        tracing::debug!("Creating issue: {:?}", request.name);

        // Validate issue name using shared validation logic, or use empty string for nameless issues
        let validated_name = match &request.name {
            Some(name) => {
                McpValidation::validate_not_empty(name.as_str(), "issue name")
                    .map_err(|e| McpErrorHandler::handle_error(e, "validate issue name"))?;
                validate_issue_name(name.as_str())?
            }
            None => String::new(), // Empty name for nameless issues - skip validation
        };

        // Validate content is not empty
        McpValidation::validate_not_empty(&request.content, "issue content")
            .map_err(|e| McpErrorHandler::handle_error(e, "validate issue content"))?;

        let issue_storage = self.issue_storage.write().await;
        match issue_storage
            .create_issue(validated_name, request.content)
            .await
        {
            Ok(issue) => {
                tracing::info!("Created issue {}", issue.name);
                Ok(create_issue_response(&issue))
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "create issue")),
        }
    }

    /// Handle the issue_mark_complete tool operation.
    ///
    /// Marks an issue as complete by moving it to the completed issues directory.
    ///
    /// # Arguments
    ///
    /// * `request` - The mark complete request containing the issue name
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_issue_mark_complete(
        &self,
        request: MarkCompleteRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        // Validate issue name is not empty
        McpValidation::validate_not_empty(request.name.as_str(), "issue name")
            .map_err(|e| McpErrorHandler::handle_error(e, "validate issue name"))?;

        let issue_storage = self.issue_storage.write().await;
        match issue_storage.mark_complete(request.name.as_str()).await {
            Ok(issue) => {
                // After successfully marking issue complete, switch back to main branch
                // if we're currently on the issue branch and commit the changes
                let git_ops_guard = self.git_ops.lock().await;
                if let Some(git_ops) = git_ops_guard.as_ref() {
                    let expected_branch = format!("issue/{}", issue.name);
                    match git_ops.current_branch() {
                        Ok(current_branch) => {
                            // If we're on the issue branch, switch back to main
                            if current_branch == expected_branch {
                                match git_ops.main_branch() {
                                    Ok(main_branch) => {
                                        if let Err(e) = git_ops.checkout_branch(&main_branch) {
                                            tracing::warn!(
                                                "Failed to switch back to main branch after completing issue {}: {}",
                                                issue.name, e
                                            );
                                        }
                                    }
                                    Err(e) => {
                                        tracing::warn!(
                                            "Failed to determine main branch after completing issue {}: {}",
                                            issue.name, e
                                        );
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!(
                                "Failed to get current branch after completing issue {}: {}",
                                issue.name,
                                e
                            );
                        }
                    }
                }
                drop(git_ops_guard);

                Ok(create_mark_complete_response(&issue))
            }
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
    /// * `request` - The update request containing issue name and new content
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_issue_update(
        &self,
        request: UpdateIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        // Validate issue name and content
        McpValidation::validate_not_empty(request.name.as_str(), "issue name")
            .map_err(|e| McpErrorHandler::handle_error(e, "validate issue name"))?;
        McpValidation::validate_not_empty(&request.content, "issue content")
            .map_err(|e| McpErrorHandler::handle_error(e, "validate issue content"))?;

        let issue_storage = self.issue_storage.write().await;

        // Handle append mode by reading existing content first
        let final_content = if request.append {
            match issue_storage.get_issue(request.name.as_str()).await {
                Ok(existing_issue) => {
                    format!("{}\n{}", existing_issue.content, request.content)
                }
                Err(_) => request.content, // If can't read existing, just use new content
            }
        } else {
            request.content
        };

        match issue_storage
            .update_issue(request.name.as_str(), final_content)
            .await
        {
            Ok(issue) => Ok(create_success_response(format!(
                "Updated issue {} ({})",
                issue.name,
                if request.append {
                    "append mode"
                } else {
                    "replace mode"
                }
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
    /// * `request` - The work request containing the issue name
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
        let branch_name = issue.name.clone();

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
    /// The branch name is determined from the issue name and name.
    ///
    /// # Arguments
    ///
    /// * `request` - The merge request containing the issue name
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

        // Note: Removed working directory check to allow merge operations when issue completion
        // creates uncommitted changes. The git merge command itself will handle conflicts appropriately.

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
                    Err(e) => {
                        // Add debug output to understand what's failing
                        tracing::error!("Merge failed for issue '{}': {}", issue_name, e);
                        Ok(create_error_response(format!(
                            "Failed to merge branch: {e}"
                        )))
                    }
                }
            }
            None => Ok(create_error_response(
                "Git operations not available".to_string(),
            )),
        }
    }


    /// Handle the memo_create tool operation.
    ///
    /// Creates a new memo with the given title and content.
    ///
    /// # Arguments
    ///
    /// * `request` - The create memo request containing title and content
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_memo_create(
        &self,
        request: CreateMemoRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::debug!("Creating memo with title: {}", request.title);

        // Note: Both title and content can be empty - storage layer supports this

        let memo_storage = self.memo_storage.write().await;
        match memo_storage
            .create_memo(request.title, request.content)
            .await
        {
            Ok(memo) => {
                tracing::info!("Created memo {}", memo.id);
                Ok(create_success_response(format!(
                    "Successfully created memo '{}' with ID: {}\n\nTitle: {}\nContent: {}",
                    memo.title, memo.id, memo.title, memo.content
                )))
            }
            Err(e) => Err(Self::handle_memo_error(e, "create memo")),
        }
    }

    /// Handle the memo_get tool operation.
    ///
    /// Retrieves a memo by its ID.
    ///
    /// # Arguments
    ///
    /// * `request` - The get memo request containing the memo ID
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_memo_get(
        &self,
        request: GetMemoRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::debug!("Getting memo with ID: {}", request.id);

        let memo_id = match MemoId::from_string(request.id.clone()) {
            Ok(id) => id,
            Err(_) => {
                return Err(McpError::invalid_params(
                    format!("Invalid memo ID format: {}", request.id),
                    None,
                ))
            }
        };

        let memo_storage = self.memo_storage.read().await;
        match memo_storage.get_memo(&memo_id).await {
            Ok(memo) => {
                tracing::info!("Retrieved memo {}", memo.id);
                Ok(create_success_response(format!(
                    "Memo found:\n\nID: {}\nTitle: {}\nCreated: {}\nUpdated: {}\n\nContent:\n{}",
                    memo.id,
                    memo.title,
                    McpFormatter::format_timestamp(memo.created_at),
                    McpFormatter::format_timestamp(memo.updated_at),
                    memo.content
                )))
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "get memo")),
        }
    }

    /// Handle the memo_update tool operation.
    ///
    /// Updates a memo's content by its ID.
    ///
    /// # Arguments
    ///
    /// * `request` - The update memo request containing memo ID and new content
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_memo_update(
        &self,
        request: UpdateMemoRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::debug!("Updating memo with ID: {}", request.id);

        // Validate memo content using shared validation
        McpValidation::validate_not_empty(&request.content, "memo content")
            .map_err(|e| McpErrorHandler::handle_error(e, "validate memo content"))?;

        let memo_id = match MemoId::from_string(request.id.clone()) {
            Ok(id) => id,
            Err(_) => {
                return Err(McpError::invalid_params(
                    format!("Invalid memo ID format: {}", request.id),
                    None,
                ))
            }
        };

        let memo_storage = self.memo_storage.write().await;
        match memo_storage.update_memo(&memo_id, request.content).await {
            Ok(memo) => {
                tracing::info!("Updated memo {}", memo.id);
                Ok(create_success_response(format!(
                    "Successfully updated memo:\n\nID: {}\nTitle: {}\nUpdated: {}\n\nContent:\n{}",
                    memo.id,
                    memo.title,
                    McpFormatter::format_timestamp(memo.updated_at),
                    memo.content
                )))
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "update memo")),
        }
    }

    /// Handle the memo_delete tool operation.
    ///
    /// Deletes a memo by its ID.
    ///
    /// # Arguments
    ///
    /// * `request` - The delete memo request containing the memo ID
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_memo_delete(
        &self,
        request: DeleteMemoRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::debug!("Deleting memo with ID: {}", request.id);

        let memo_id = match MemoId::from_string(request.id.clone()) {
            Ok(id) => id,
            Err(_) => {
                return Err(McpError::invalid_params(
                    format!("Invalid memo ID format: {}", request.id),
                    None,
                ))
            }
        };

        let memo_storage = self.memo_storage.write().await;
        match memo_storage.delete_memo(&memo_id).await {
            Ok(()) => {
                tracing::info!("Deleted memo {}", request.id);
                Ok(create_success_response(format!(
                    "Successfully deleted memo with ID: {}",
                    request.id
                )))
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "delete memo")),
        }
    }

    /// Handle the memo_list tool operation.
    ///
    /// Lists all available memos.
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_memo_list(
        &self,
        _request: ListMemosRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::debug!("Listing all memos");

        let memo_storage = self.memo_storage.read().await;
        match memo_storage.list_memos().await {
            Ok(memos) => {
                tracing::info!("Retrieved {} memos", memos.len());
                if memos.is_empty() {
                    Ok(create_success_response("No memos found".to_string()))
                } else {
                    let memo_list = memos
                        .iter()
                        .map(|memo| Self::format_memo_preview(memo, MEMO_LIST_PREVIEW_LENGTH))
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    let summary =
                        McpFormatter::format_list_summary("memo", memos.len(), memos.len());
                    Ok(create_success_response(format!(
                        "{summary}:\n\n{memo_list}"
                    )))
                }
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "list memos")),
        }
    }

    /// Handle the memo_search tool operation.
    ///
    /// Searches memos by query string.
    ///
    /// # Arguments
    ///
    /// * `request` - The search memo request containing the search query
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_memo_search(
        &self,
        request: SearchMemosRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::debug!("Searching memos with query: {}", request.query);

        // Validate search query is not empty
        McpValidation::validate_not_empty(&request.query, "search query")
            .map_err(|e| McpErrorHandler::handle_error(e, "validate search query"))?;

        let memo_storage = self.memo_storage.read().await;
        match memo_storage.search_memos(&request.query).await {
            Ok(memos) => {
                tracing::info!("Search returned {} memos", memos.len());
                if memos.is_empty() {
                    Ok(create_success_response(format!(
                        "No memos found matching query: '{}'",
                        request.query
                    )))
                } else {
                    let memo_list = memos
                        .iter()
                        .map(|memo| Self::format_memo_preview(memo, MEMO_SEARCH_PREVIEW_LENGTH))
                        .collect::<Vec<_>>()
                        .join("\n\n");

                    Ok(create_success_response(format!(
                        "Found {} memo{} matching '{}':\n\n{}",
                        memos.len(),
                        if memos.len() == 1 { "" } else { "s" },
                        request.query,
                        memo_list
                    )))
                }
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "search memos")),
        }
    }

    /// Handle the memo_get_all_context tool operation.
    ///
    /// Gets all memo content formatted for AI context consumption.
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result
    pub async fn handle_memo_get_all_context(
        &self,
        _request: GetAllContextRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::debug!("Getting all memo context");

        let memo_storage = self.memo_storage.read().await;
        match memo_storage.list_memos().await {
            Ok(memos) => {
                tracing::info!("Retrieved {} memos for context", memos.len());
                if memos.is_empty() {
                    Ok(create_success_response("No memos available".to_string()))
                } else {
                    // Sort memos by updated_at descending (most recent first)
                    let mut sorted_memos = memos;
                    sorted_memos.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

                    let context = sorted_memos
                        .iter()
                        .map(|memo| {
                            format!(
                                "=== {} (ID: {}) ===\nCreated: {}\nUpdated: {}\n\n{}",
                                memo.title,
                                memo.id,
                                McpFormatter::format_timestamp(memo.created_at),
                                McpFormatter::format_timestamp(memo.updated_at),
                                memo.content
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(&format!("\n\n{}\n\n", "=".repeat(80)));

                    let memo_count = sorted_memos.len();
                    let plural_suffix = if memo_count == 1 { "" } else { "s" };
                    Ok(create_success_response(format!(
                        "All memo context ({memo_count} memo{plural_suffix}):\n\n{context}"
                    )))
                }
            }
            Err(e) => Err(McpErrorHandler::handle_error(e, "get memo context")),
        }
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

    /// Handle the issue_next tool operation.
    ///
    /// Gets the next issue to work on by returning the first pending issue alphabetically by name.
    ///
    /// # Arguments
    ///
    /// * `_request` - The next issue request (no parameters needed)
    ///
    /// # Returns
    ///
    /// A result containing the next issue name or an error message
    pub async fn handle_issue_next(
        &self,
        _request: NextIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let issue_storage = self.issue_storage.read().await;

        // Use the new get_next_issue method from storage
        match issue_storage.get_next_issue().await {
            Ok(Some(next_issue)) => Ok(CallToolResult {
                content: vec![Annotated::new(
                    RawContent::Text(RawTextContent {
                        text: format!("Next issue: {}", next_issue.name.as_str()),
                    }),
                    None,
                )],
                is_error: Some(false),
            }),
            Ok(None) => Ok(CallToolResult {
                content: vec![Annotated::new(
                    RawContent::Text(RawTextContent {
                        text: "No pending issues found. All issues are completed!".to_string(),
                    }),
                    None,
                )],
                is_error: Some(false),
            }),
            Err(e) => Ok(CallToolResult {
                content: vec![Annotated::new(
                    RawContent::Text(RawTextContent {
                        text: format!("Failed to get next issue: {e}"),
                    }),
                    None,
                )],
                is_error: Some(true),
            }),
        }
    }

    /// Helper method to format issue branch names consistently
    fn format_issue_branch_name(issue_name: &str) -> String {
        format!("issue/{issue_name}")
    }
}
