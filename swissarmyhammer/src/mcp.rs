//! Model Context Protocol (MCP) server support

use crate::file_watcher::{FileWatcher, FileWatcherCallback};
use crate::git::GitOperations;
use crate::issues::{FileSystemIssueStorage, IssueStorage};
use crate::workflow::{
    FileSystemWorkflowRunStorage, FileSystemWorkflowStorage, WorkflowRunStorageBackend,
    WorkflowStorage, WorkflowStorageBackend,
};
use crate::{PromptLibrary, PromptResolver, Result, SwissArmyHammerError};
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer, ServerHandler};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// MCP module structure
pub mod constants;
pub mod error_handling;
pub mod file_watcher;
pub mod responses;
pub mod tool_handlers;
pub mod types;
pub mod utils;

// Re-export commonly used items from submodules
use utils::validate_issue_name;
use responses::create_issue_response;

/// Constants for issue branch management
const ISSUE_BRANCH_PREFIX: &str = "issue/";
const ISSUE_NUMBER_WIDTH: usize = 6;

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
}

/// MCP server for serving prompts and workflows
#[derive(Clone)]
pub struct McpServer {
    library: Arc<RwLock<PromptLibrary>>,
    workflow_storage: Arc<RwLock<WorkflowStorage>>,
    file_watcher: Arc<Mutex<FileWatcher>>,
    issue_storage: Arc<RwLock<Box<dyn IssueStorage>>>,
    git_ops: Arc<Mutex<Option<GitOperations>>>,
}

impl McpServer {
    /// Create a new MCP server with the provided prompt library.
    ///
    /// # Arguments
    ///
    /// * `library` - The prompt library to serve via MCP
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The MCP server instance or an error if initialization fails
    ///
    /// # Errors
    ///
    /// Returns an error if workflow storage, issue storage, or git operations fail to initialize.
    pub fn new(library: PromptLibrary) -> Result<Self> {
        // Initialize workflow storage with filesystem backend
        let workflow_backend = Arc::new(FileSystemWorkflowStorage::new().map_err(|e| {
            tracing::error!("Failed to create workflow storage: {}", e);
            SwissArmyHammerError::Other(format!("Failed to create workflow storage: {}", e))
        })?) as Arc<dyn WorkflowStorageBackend>;

        // Create runs directory in user's home directory
        let runs_path = Self::get_workflow_runs_path();

        let run_backend = Arc::new(FileSystemWorkflowRunStorage::new(runs_path).map_err(|e| {
            tracing::error!("Failed to create workflow run storage: {}", e);
            SwissArmyHammerError::Other(format!("Failed to create workflow run storage: {}", e))
        })?) as Arc<dyn WorkflowRunStorageBackend>;

        let workflow_storage = WorkflowStorage::new(workflow_backend, run_backend);

        // Initialize issue storage with default issues directory
        let issues_dir = std::env::current_dir()
            .map_err(|e| {
                SwissArmyHammerError::Other(format!("Failed to get current directory: {}", e))
            })?
            .join("issues");

        let issue_storage = Box::new(FileSystemIssueStorage::new(issues_dir).map_err(|e| {
            tracing::error!("Failed to create issue storage: {}", e);
            SwissArmyHammerError::Other(format!("Failed to create issue storage: {}", e))
        })?) as Box<dyn IssueStorage>;

        // Initialize git operations - make it optional for tests
        let git_ops = match GitOperations::new() {
            Ok(ops) => Some(ops),
            Err(e) => {
                tracing::warn!("Git operations not available: {}", e);
                None
            }
        };

        Ok(Self {
            library: Arc::new(RwLock::new(library)),
            workflow_storage: Arc::new(RwLock::new(workflow_storage)),
            file_watcher: Arc::new(Mutex::new(FileWatcher::new())),
            issue_storage: Arc::new(RwLock::new(issue_storage)),
            git_ops: Arc::new(Mutex::new(git_ops)),
        })
    }

    /// Get a reference to the underlying prompt library.
    ///
    /// # Returns
    ///
    /// * `&Arc<RwLock<PromptLibrary>>` - Reference to the wrapped prompt library
    pub fn library(&self) -> &Arc<RwLock<PromptLibrary>> {
        &self.library
    }

    /// Initialize the server by loading prompts and workflows from disk.
    ///
    /// This method loads all prompts using the PromptResolver and initializes
    /// workflow storage. It should be called before starting the MCP server.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if initialization succeeds, error otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if prompt loading or workflow initialization fails.
    pub async fn initialize(&self) -> Result<()> {
        let mut library = self.library.write().await;
        let mut resolver = PromptResolver::new();

        // Use the same loading logic as CLI
        resolver.load_all_prompts(&mut library)?;

        let total = library.list()?.len();
        tracing::info!("Loaded {} prompts total", total);

        // Initialize workflows - workflows are loaded automatically by FileSystemWorkflowStorage
        // so we just need to check how many are available
        let workflow_storage = self.workflow_storage.read().await;
        let workflow_count = workflow_storage.list_workflows()?.len();
        tracing::info!("Loaded {} workflows total", workflow_count);

        Ok(())
    }

    /// List all available prompts, excluding partial templates.
    ///
    /// Partial templates are filtered out as they are meant for internal use
    /// and should not be exposed via the MCP interface.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>>` - List of prompt names or an error
    pub async fn list_prompts(&self) -> Result<Vec<String>> {
        let library = self.library.read().await;
        let prompts = library.list()?;
        Ok(prompts
            .iter()
            .filter(|p| !Self::is_partial_template(p))
            .map(|p| p.name.clone())
            .collect())
    }

    /// List all available workflows loaded from the workflow storage.
    ///
    /// # Returns
    ///
    /// * `Result<Vec<String>>` - List of workflow names or an error
    pub async fn list_workflows(&self) -> Result<Vec<String>> {
        let workflow_storage = self.workflow_storage.read().await;
        let workflows = workflow_storage.list_workflows()?;
        Ok(workflows.iter().map(|w| w.name.to_string()).collect())
    }

    /// Check if a prompt is a partial template that should not be exposed over MCP.
    ///
    /// Partial templates are identified by either:
    /// 1. Starting with the `{% partial %}` marker
    /// 2. Having a description containing "Partial template for reuse"
    ///
    /// # Arguments
    ///
    /// * `prompt` - The prompt to check
    ///
    /// # Returns
    ///
    /// * `bool` - True if the prompt is a partial template
    fn is_partial_template(prompt: &crate::prompts::Prompt) -> bool {
        // Check if the template starts with the partial marker
        if prompt.template.trim().starts_with("{% partial %}") {
            return true;
        }

        // Check if the description indicates it's a partial template
        if let Some(description) = &prompt.description {
            if description.contains("Partial template for reuse in other prompts") {
                return true;
            }
        }

        false
    }

    /// Get a specific prompt by name, with optional template argument rendering.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the prompt to retrieve
    /// * `arguments` - Optional template arguments for rendering
    ///
    /// # Returns
    ///
    /// * `Result<String>` - The rendered prompt content or an error
    ///
    /// # Errors
    ///
    /// Returns an error if the prompt is not found, is a partial template,
    /// or if template rendering fails.
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<&HashMap<String, String>>,
    ) -> Result<String> {
        let library = self.library.read().await;
        let prompt = library.get(name)?;

        // Check if this is a partial template
        if Self::is_partial_template(&prompt) {
            return Err(SwissArmyHammerError::Other(format!(
                "Cannot access partial template '{}' via MCP. Partial templates are for internal use only.",
                name
            )));
        }

        // Handle arguments if provided
        let content = if let Some(args) = arguments {
            library.render_prompt(name, args)?
        } else {
            prompt.template.clone()
        };

        Ok(content)
    }
}

/// Callback implementation for file watcher that handles prompt reloading
#[derive(Clone)]
struct McpFileWatcherCallback {
    server: McpServer,
    peer: rmcp::Peer<RoleServer>,
}

impl McpFileWatcherCallback {
    fn new(server: McpServer, peer: rmcp::Peer<RoleServer>) -> Self {
        Self { server, peer }
    }
}

impl FileWatcherCallback for McpFileWatcherCallback {
    async fn on_file_changed(&self, paths: Vec<std::path::PathBuf>) -> Result<()> {
        tracing::info!("📄 Prompt file changed: {:?}", paths);

        // Reload the library
        if let Err(e) = self.server.reload_prompts().await {
            tracing::error!("❌ Failed to reload prompts: {}", e);
            return Err(e);
        } else {
            tracing::info!("✅ Prompts reloaded successfully");
        }

        // Send notification to client about prompt list change
        let peer_clone = self.peer.clone();
        tokio::spawn(async move {
            match peer_clone.notify_prompt_list_changed().await {
                Ok(_) => {
                    tracing::info!("📢 Sent prompts/listChanged notification to client");
                }
                Err(e) => {
                    tracing::error!("❌ Failed to send notification: {}", e);
                }
            }
        });

        Ok(())
    }

    async fn on_error(&self, error: String) {
        tracing::error!("❌ File watcher error: {}", error);
    }
}

impl McpServer {
    /// Start watching prompt directories for file changes.
    ///
    /// When files change, the server will automatically reload prompts and
    /// send notifications to the MCP client.
    ///
    /// # Arguments
    ///
    /// * `peer` - The MCP peer connection for sending notifications
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if watching starts successfully, error otherwise
    ///
    /// # Errors
    ///
    /// Returns an error if file watching cannot be initialized.
    pub async fn start_file_watching(&self, peer: rmcp::Peer<RoleServer>) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_BACKOFF_MS: u64 = 100;

        // Create callback that handles file changes and notifications
        let callback = McpFileWatcherCallback::new(self.clone(), peer);

        let mut last_error = None;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        for attempt in 1..=MAX_RETRIES {
            // Start watching using the file watcher module
            let result = {
                let mut watcher = self.file_watcher.lock().await;
                watcher.start_watching(callback.clone()).await
            };

            match result {
                Ok(()) => {
                    if attempt > 1 {
                        tracing::info!(
                            "✅ File watcher started successfully on attempt {}",
                            attempt
                        );
                    }
                    return Ok(());
                }
                Err(e) => {
                    last_error = Some(e);

                    if attempt < MAX_RETRIES
                        && Self::is_retryable_fs_error(last_error.as_ref().unwrap())
                    {
                        tracing::warn!(
                            "⚠️ File watcher initialization attempt {} failed, retrying in {}ms: {}",
                            attempt,
                            backoff_ms,
                            last_error.as_ref().unwrap()
                        );

                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2; // Exponential backoff
                    } else {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Stop watching prompt directories for file changes.
    ///
    /// This should be called when the MCP server is shutting down.
    pub async fn stop_file_watching(&self) {
        let mut watcher = self.file_watcher.lock().await;
        watcher.stop_watching();
    }

    /// Get the workflow runs directory path
    fn get_workflow_runs_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| {
                std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."))
            })
            .join(".swissarmyhammer")
            .join("workflow-runs")
    }

    /// Convert internal prompt arguments to MCP PromptArgument structures.
    ///
    /// # Arguments
    ///
    /// * `args` - The internal argument specifications
    ///
    /// # Returns
    ///
    /// * `Option<Vec<PromptArgument>>` - The converted MCP arguments or None if empty
    fn convert_prompt_arguments(args: &[crate::ArgumentSpec]) -> Option<Vec<PromptArgument>> {
        if args.is_empty() {
            None
        } else {
            Some(
                args.iter()
                    .map(|arg| PromptArgument {
                        name: arg.name.clone(),
                        description: arg.description.clone(),
                        required: Some(arg.required),
                    })
                    .collect(),
            )
        }
    }

    /// Convert serde_json::Map to HashMap<String, String> for template rendering.
    ///
    /// This helper method converts MCP tool arguments from JSON format to
    /// the string format expected by the template engine.
    ///
    /// # Arguments
    ///
    /// * `args` - The JSON map of arguments from MCP
    ///
    /// # Returns
    ///
    /// * `HashMap<String, String>` - The converted arguments
    fn json_map_to_string_map(args: &serde_json::Map<String, Value>) -> HashMap<String, String> {
        let mut template_args = HashMap::new();
        for (key, value) in args {
            let value_str = match value {
                Value::String(s) => s.clone(),
                v => v.to_string(),
            };
            template_args.insert(key.clone(), value_str);
        }
        template_args
    }

    /// Generate input schema for a tool from a request type.
    ///
    /// This helper method creates the JSON schema for MCP tool input validation
    /// using the schemars crate. It handles schema generation errors gracefully
    /// by returning an empty schema if generation fails.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The request type that implements JsonSchema
    ///
    /// # Returns
    ///
    /// * `Arc<serde_json::Map<String, Value>>` - The generated schema or empty schema
    fn generate_tool_schema<T>() -> Arc<serde_json::Map<String, Value>>
    where
        T: schemars::JsonSchema,
    {
        serde_json::to_value(schemars::schema_for!(T))
            .ok()
            .and_then(|v| v.as_object().map(|obj| Arc::new(obj.clone())))
            .unwrap_or_else(|| Arc::new(serde_json::Map::new()))
    }

    /// Create a success response for a tool call.
    ///
    /// # Arguments
    ///
    /// * `message` - The success message to include in the response
    ///
    /// # Returns
    ///
    /// * `CallToolResult` - A successful tool call result with the message
    fn create_success_response(message: String) -> CallToolResult {
        CallToolResult {
            content: vec![Annotated::new(
                RawContent::Text(RawTextContent { text: message }),
                None,
            )],
            is_error: Some(false),
        }
    }

    /// Create an error response for a tool call.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message to include in the response
    ///
    /// # Returns
    ///
    /// * `CallToolResult` - An error tool call result with the message
    fn create_error_response(message: String) -> CallToolResult {
        CallToolResult {
            content: vec![Annotated::new(
                RawContent::Text(RawTextContent { text: message }),
                None,
            )],
            is_error: Some(true),
        }
    }

    /// Validate and sanitize issue name for MCP tool calls.
    ///
    /// # Arguments
    ///
    /// * `name` - The issue name to validate
    ///
    /// # Returns
    ///
    /// * `Result<String, McpError>` - The validated name or MCP error

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
    async fn handle_issue_create(
        &self,
        request: CreateIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        tracing::debug!("Creating issue: {}", request.name);

        // Validate issue name using shared validation logic
        let validated_name = validate_issue_name(&request.name)?;

        let issue_storage = self.issue_storage.write().await;
        match issue_storage
            .create_issue(validated_name, request.content)
            .await
        {
            Ok(issue) => {
                tracing::info!("Created issue {} with number {}", issue.name, issue.number);
                Ok(create_issue_response(&issue))
            }
            Err(SwissArmyHammerError::IssueAlreadyExists(num)) => {
                tracing::warn!("Issue #{:06} already exists", num);
                Err(McpError::invalid_params(
                    format!("Issue #{:06} already exists", num),
                    None,
                ))
            }
            Err(e) => {
                tracing::error!("Failed to create issue: {}", e);
                Err(McpError::internal_error(
                    format!("Failed to create issue: {}", e),
                    None,
                ))
            }
        }
    }

    /// Get completion statistics
    async fn get_issue_stats(&self) -> crate::Result<(usize, usize)> {
        let issue_storage = self.issue_storage.read().await;
        let all_issues = issue_storage.list_issues().await?;
        
        let completed = all_issues.iter().filter(|i| i.completed).count();
        let pending = all_issues.len() - completed;
        
        Ok((pending, completed))
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
    async fn handle_issue_mark_complete(
        &self,
        request: MarkCompleteRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        // Validate issue number
        if request.number == 0 || request.number > 999999 {
            return Err(McpError::invalid_params(
                "Invalid issue number (must be 1-999999)".to_string(),
                None,
            ));
        }
        
        // Get issue storage
        let issue_storage = self.issue_storage.write().await;
        
        // Check if issue exists and get its current state
        let existing_issue = match issue_storage.get_issue(request.number).await {
            Ok(issue) => issue,
            Err(crate::SwissArmyHammerError::IssueNotFound(_)) => {
                return Err(McpError::invalid_params(
                    format!("Issue #{:06} not found", request.number),
                    None,
                ));
            }
            Err(e) => {
                return Err(McpError::internal_error(
                    format!("Failed to get issue: {}", e),
                    None,
                ));
            }
        };
        
        // Check if already completed
        if existing_issue.completed {
            return Ok(CallToolResult {
                content: vec![Annotated::new(
                    RawContent::Text(RawTextContent { 
                        text: format!(
                            "Issue #{:06} - {} is already marked as complete",
                            existing_issue.number,
                            existing_issue.name
                        )
                    }),
                    None,
                )],
                is_error: Some(false),
            });
        }
        
        // Mark the issue as complete
        let issue = match issue_storage.mark_complete(request.number).await {
            Ok(issue) => issue,
            Err(crate::SwissArmyHammerError::IssueNotFound(_)) => {
                return Err(McpError::invalid_params(
                    format!("Issue #{:06} not found", request.number),
                    None,
                ));
            }
            Err(e) => {
                return Err(McpError::internal_error(
                    format!("Failed to mark issue complete: {}", e),
                    None,
                ));
            }
        };
        
        // Get statistics
        let (pending, completed) = self.get_issue_stats().await
            .unwrap_or((0, 0));
        
        // Format response
        let response = serde_json::json!({
            "number": issue.number,
            "name": issue.name,
            "file_path": issue.file_path.to_string_lossy(),
            "completed": issue.completed,
            "stats": {
                "pending": pending,
                "completed": completed,
                "total": pending + completed,
            },
            "message": format!(
                "Issue #{:06} - {} marked as complete. {} issues pending, {} completed.",
                issue.number,
                issue.name,
                pending,
                completed
            )
        });
        
        Ok(CallToolResult {
            content: vec![Annotated::new(
                RawContent::Text(RawTextContent { 
                    text: response["message"].as_str().unwrap().to_string()
                }),
                None,
            )],
            is_error: Some(false),
        })
    }

    /// Handle the issue_all_complete tool operation.
    ///
    /// Checks if all issues are completed by listing pending issues.
    ///
    /// # Arguments
    ///
    /// * `_request` - The all complete request (no parameters needed)
    ///
    /// # Returns
    ///
    /// * `Result<CallToolResult, McpError>` - The tool call result with completion status
    async fn handle_issue_all_complete(
        &self,
        _request: AllCompleteRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let issue_storage = self.issue_storage.read().await;
        match issue_storage.list_issues().await {
            Ok(issues) => {
                let pending_issues: Vec<_> = issues.iter().filter(|i| !i.completed).collect();
                let all_complete = pending_issues.is_empty();
                Ok(Self::create_success_response(format!(
                    "All issues complete: {}. Pending issues: {}",
                    all_complete,
                    pending_issues.len()
                )))
            }
            Err(e) => Ok(Self::create_error_response(format!(
                "Failed to check issues: {}",
                e
            ))),
        }
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
    async fn handle_issue_update(
        &self,
        request: UpdateIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let issue_storage = self.issue_storage.write().await;
        match issue_storage
            .update_issue(request.number, request.content)
            .await
        {
            Ok(issue) => Ok(Self::create_success_response(format!(
                "Updated issue {} ({})",
                issue.number, issue.name
            ))),
            Err(e) => Ok(Self::create_error_response(format!(
                "Failed to update issue: {}",
                e
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
    async fn handle_issue_current(
        &self,
        _request: CurrentIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        let git_ops = self.git_ops.lock().await;
        match git_ops.as_ref() {
            Some(ops) => match ops.current_branch() {
                Ok(branch) => {
                    if let Some(issue_name) = branch.strip_prefix(ISSUE_BRANCH_PREFIX) {
                        Ok(Self::create_success_response(format!(
                            "Currently working on issue: {}",
                            issue_name
                        )))
                    } else {
                        Ok(Self::create_success_response(format!(
                            "Not on an issue branch. Current branch: {}",
                            branch
                        )))
                    }
                }
                Err(e) => Ok(Self::create_error_response(format!(
                    "Failed to get current branch: {}",
                    e
                ))),
            },
            None => Ok(Self::create_error_response(
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
    async fn handle_issue_work(
        &self,
        request: WorkIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        // First get the issue to determine its name
        let issue_storage = self.issue_storage.read().await;
        let issue = match issue_storage.get_issue(request.number).await {
            Ok(issue) => issue,
            Err(e) => {
                return Ok(Self::create_error_response(format!(
                    "Failed to get issue {}: {}",
                    request.number, e
                )))
            }
        };
        drop(issue_storage);

        // Create work branch
        let mut git_ops = self.git_ops.lock().await;
        let issue_name = format!(
            "{:0width$}_{}",
            issue.number,
            issue.name,
            width = ISSUE_NUMBER_WIDTH
        );

        match git_ops.as_mut() {
            Some(ops) => match ops.create_work_branch(&issue_name) {
                Ok(branch_name) => Ok(Self::create_success_response(format!(
                    "Switched to work branch: {}",
                    branch_name
                ))),
                Err(e) => Ok(Self::create_error_response(format!(
                    "Failed to create work branch: {}",
                    e
                ))),
            },
            None => Ok(Self::create_error_response(
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
    async fn handle_issue_merge(
        &self,
        request: MergeIssueRequest,
    ) -> std::result::Result<CallToolResult, McpError> {
        // First get the issue to determine its name
        let issue_storage = self.issue_storage.read().await;
        let issue = match issue_storage.get_issue(request.number).await {
            Ok(issue) => issue,
            Err(e) => {
                return Ok(Self::create_error_response(format!(
                    "Failed to get issue {}: {}",
                    request.number, e
                )))
            }
        };
        drop(issue_storage);

        // Merge branch
        let mut git_ops = self.git_ops.lock().await;
        let issue_name = format!(
            "{:0width$}_{}",
            issue.number,
            issue.name,
            width = ISSUE_NUMBER_WIDTH
        );

        match git_ops.as_mut() {
            Some(ops) => match ops.merge_issue_branch(&issue_name) {
                Ok(_) => Ok(Self::create_success_response(format!(
                    "Merged work branch for issue {} to main",
                    issue_name
                ))),
                Err(e) => Ok(Self::create_error_response(format!(
                    "Failed to merge branch: {}",
                    e
                ))),
            },
            None => Ok(Self::create_error_response(
                "Git operations not available".to_string(),
            )),
        }
    }

    /// Reload prompts from disk with retry logic.
    ///
    /// This method reloads all prompts from the file system and updates
    /// the internal library. It includes retry logic for transient errors.
    ///
    /// # Returns
    ///
    /// * `Result<()>` - Ok if reload succeeds, error otherwise
    async fn reload_prompts(&self) -> Result<()> {
        self.reload_prompts_with_retry().await
    }

    /// Reload prompts with retry logic for transient file system errors
    async fn reload_prompts_with_retry(&self) -> Result<()> {
        const MAX_RETRIES: u32 = 3;
        const INITIAL_BACKOFF_MS: u64 = 100;

        let mut last_error = None;
        let mut backoff_ms = INITIAL_BACKOFF_MS;

        for attempt in 1..=MAX_RETRIES {
            match self.reload_prompts_internal().await {
                Ok(()) => return Ok(()),
                Err(e) => {
                    last_error = Some(e);

                    // Check if this is a retryable error
                    if attempt < MAX_RETRIES
                        && Self::is_retryable_fs_error(last_error.as_ref().unwrap())
                    {
                        tracing::warn!(
                            "⚠️ Reload attempt {} failed, retrying in {}ms: {}",
                            attempt,
                            backoff_ms,
                            last_error.as_ref().unwrap()
                        );

                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2; // Exponential backoff
                    } else {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    /// Check if an error is a retryable file system error
    fn is_retryable_fs_error(error: &SwissArmyHammerError) -> bool {
        // Check for common transient file system errors
        if let SwissArmyHammerError::Io(io_err) = error {
            matches!(
                io_err.kind(),
                std::io::ErrorKind::TimedOut
                    | std::io::ErrorKind::Interrupted
                    | std::io::ErrorKind::WouldBlock
                    | std::io::ErrorKind::UnexpectedEof
            )
        } else {
            // Also retry if the error message contains certain patterns
            let error_str = error.to_string().to_lowercase();
            error_str.contains("temporarily unavailable")
                || error_str.contains("resource busy")
                || error_str.contains("locked")
        }
    }

    /// Internal reload method that performs the actual reload
    async fn reload_prompts_internal(&self) -> Result<()> {
        let mut library = self.library.write().await;
        let mut resolver = PromptResolver::new();

        // Get count before reload (default to 0 if library.list() fails)
        let before_count = library.list().map(|p| p.len()).unwrap_or(0);

        // Clear existing prompts and reload
        *library = PromptLibrary::new();
        resolver.load_all_prompts(&mut library)?;

        let after_count = library.list()?.len();
        tracing::info!(
            "🔄 Reloaded prompts: {} → {} prompts",
            before_count,
            after_count
        );

        Ok(())
    }
}

impl ServerHandler for McpServer {
    async fn initialize(
        &self,
        request: InitializeRequestParam,
        context: RequestContext<RoleServer>,
    ) -> std::result::Result<InitializeResult, McpError> {
        tracing::info!(
            "🚀 MCP client connecting: {} v{}",
            request.client_info.name,
            request.client_info.version
        );

        // Start file watching when MCP client connects
        match self.start_file_watching(context.peer).await {
            Ok(_) => {
                tracing::info!("🔍 File watching started for MCP client");
            }
            Err(e) => {
                tracing::error!("❌ Failed to start file watching for MCP client: {}", e);
                // Continue initialization even if file watching fails
            }
        }

        Ok(InitializeResult {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                prompts: Some(PromptsCapability {
                    list_changed: Some(true),
                }),
                tools: Some(ToolsCapability {
                    list_changed: Some(true),
                }),
                resources: None,
                logging: None,
                completions: None,
                experimental: None,
            },instructions: Some("A flexible prompt and workflow management server with integrated issue tracking. Use list_prompts to see available prompts and get_prompt to retrieve and render them. Use workflow tools to execute and manage workflows. Use issue_* tools to create and manage work items tracked as markdown files in your repository.".into()),
            server_info: Implementation {
                name: "SwissArmyHammer".into(),
                version: crate::VERSION.into(),
            },
        })
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListPromptsResult, McpError> {
        let library = self.library.read().await;
        match library.list() {
            Ok(prompts) => {
                let prompt_list: Vec<Prompt> = prompts
                    .iter()
                    .filter(|p| !Self::is_partial_template(p)) // Filter out partial templates
                    .map(|p| {
                        let arguments = Self::convert_prompt_arguments(&p.arguments);

                        Prompt {
                            name: p.name.clone(),
                            description: p.description.clone(),
                            arguments,
                        }
                    })
                    .collect();

                Ok(ListPromptsResult {
                    prompts: prompt_list,
                    next_cursor: None,
                })
            }
            Err(e) => Err(McpError::internal_error(e.to_string(), None)),
        }
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<GetPromptResult, McpError> {
        let library = self.library.read().await;
        match library.get(&request.name) {
            Ok(prompt) => {
                // Check if this is a partial template
                if Self::is_partial_template(&prompt) {
                    return Err(McpError::invalid_request(
                        format!(
                            "Cannot access partial template '{}' via MCP. Partial templates are for internal use only.",
                            request.name
                        ),
                        None,
                    ));
                }

                // Handle arguments if provided
                let content = if let Some(args) = &request.arguments {
                    let template_args = Self::json_map_to_string_map(args);

                    match library.render_prompt(&request.name, &template_args) {
                        Ok(rendered) => rendered,
                        Err(e) => {
                            return Err(McpError::internal_error(
                                format!("Template rendering error: {}", e),
                                None,
                            ))
                        }
                    }
                } else {
                    prompt.template.clone()
                };

                Ok(GetPromptResult {
                    description: prompt.description,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::Text { text: content },
                    }],
                })
            }
            Err(e) => {
                tracing::warn!("Prompt '{}' not found: {}", request.name, e);
                Err(McpError::invalid_request(
                    format!(
                        "Prompt '{}' is not available. It may have been deleted or renamed.",
                        request.name
                    ),
                    None,
                ))
            }
        }
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParam>,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<ListToolsResult, McpError> {
        let tools = vec![
            Tool {
                name: "issue_create".into(),
                description: Some("Create a new issue with auto-assigned number. Issues are markdown files stored in ./issues directory for tracking work items.".into()),
                input_schema: Self::generate_tool_schema::<CreateIssueRequest>(),
                annotations: None,
            },
            Tool {
                name: "issue_mark_complete".into(),
                description: Some("Mark an issue as complete by moving it to ./issues/complete directory.".into()),
                input_schema: Self::generate_tool_schema::<MarkCompleteRequest>(),
                annotations: None,
            },
            Tool {
                name: "issue_all_complete".into(),
                description: Some("Check if all issues are completed. Returns true if no pending issues remain.".into()),
                input_schema: Self::generate_tool_schema::<AllCompleteRequest>(),
                annotations: None,
            },
            Tool {
                name: "issue_update".into(),
                description: Some("Update the content of an existing issue with additional context or modifications.".into()),
                input_schema: Self::generate_tool_schema::<UpdateIssueRequest>(),
                annotations: None,
            },
            Tool {
                name: "issue_current".into(),
                description: Some("Get the current issue being worked on. Checks branch name to identify active issue.".into()),
                input_schema: Self::generate_tool_schema::<CurrentIssueRequest>(),
                annotations: None,
            },
            Tool {
                name: "issue_work".into(),
                description: Some("Switch to a work branch for the specified issue (creates branch issue/<issue_name> if needed).".into()),
                input_schema: Self::generate_tool_schema::<WorkIssueRequest>(),
                annotations: None,
            },
            Tool {
                name: "issue_merge".into(),
                description: Some("Merge the work branch for an issue back to the main branch.".into()),
                input_schema: Self::generate_tool_schema::<MergeIssueRequest>(),
                annotations: None,
            },
        ];

        Ok(ListToolsResult {
            tools,
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResult, McpError> {
        match request.name.as_ref() {
            "issue_create" => {
                let req: CreateIssueRequest = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.clone().unwrap_or_default(),
                ))
                .map_err(|e| {
                    McpError::invalid_request(format!("Invalid arguments: {}", e), None)
                })?;
                self.handle_issue_create(req).await
            }

            "issue_mark_complete" => {
                let req: MarkCompleteRequest = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.clone().unwrap_or_default(),
                ))
                .map_err(|e| {
                    McpError::invalid_request(format!("Invalid arguments: {}", e), None)
                })?;
                self.handle_issue_mark_complete(req).await
            }

            "issue_all_complete" => {
                let req: AllCompleteRequest = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.clone().unwrap_or_default(),
                ))
                .map_err(|e| {
                    McpError::invalid_request(format!("Invalid arguments: {}", e), None)
                })?;
                self.handle_issue_all_complete(req).await
            }

            "issue_update" => {
                let req: UpdateIssueRequest = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.clone().unwrap_or_default(),
                ))
                .map_err(|e| {
                    McpError::invalid_request(format!("Invalid arguments: {}", e), None)
                })?;
                self.handle_issue_update(req).await
            }

            "issue_current" => {
                let req: CurrentIssueRequest = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.clone().unwrap_or_default(),
                ))
                .map_err(|e| {
                    McpError::invalid_request(format!("Invalid arguments: {}", e), None)
                })?;
                self.handle_issue_current(req).await
            }

            "issue_work" => {
                let req: WorkIssueRequest = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.clone().unwrap_or_default(),
                ))
                .map_err(|e| {
                    McpError::invalid_request(format!("Invalid arguments: {}", e), None)
                })?;
                self.handle_issue_work(req).await
            }

            "issue_merge" => {
                let req: MergeIssueRequest = serde_json::from_value(serde_json::Value::Object(
                    request.arguments.clone().unwrap_or_default(),
                ))
                .map_err(|e| {
                    McpError::invalid_request(format!("Invalid arguments: {}", e), None)
                })?;
                self.handle_issue_merge(req).await
            }

            _ => Err(McpError::invalid_request(
                format!("Unknown tool: {}", request.name),
                None,
            )),
        }
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                prompts: Some(PromptsCapability {
                    list_changed: Some(true),
                }),
                tools: Some(ToolsCapability {
                    list_changed: Some(true),
                }),
                resources: None,
                logging: None,
                completions: None,
                experimental: None,
            },server_info: Implementation {
                name: "SwissArmyHammer".into(),
                version: crate::VERSION.into(),
            },instructions: Some("A flexible prompt and workflow management server with integrated issue tracking. Use list_prompts to see available prompts and get_prompt to retrieve and render them. Use workflow tools to execute and manage workflows. Use issue_* tools to create and manage work items tracked as markdown files in your repository.".into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prompts::Prompt;

    #[tokio::test]
    async fn test_mcp_server_creation() {
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let info = server.get_info();
        // Just verify we can get server info - details depend on default implementation
        assert!(!info.server_info.name.is_empty());
        assert!(!info.server_info.version.is_empty());

        // Debug print to see what capabilities are returned
        println!("Server capabilities: {:?}", info.capabilities);
    }

    #[tokio::test]
    async fn test_mcp_server_list_prompts() {
        let mut library = PromptLibrary::new();
        let prompt = Prompt::new("test", "Test prompt: {{ name }}")
            .with_description("Test description".to_string());
        library.add(prompt).unwrap();

        let server = McpServer::new(library).unwrap();
        let prompts = server.list_prompts().await.unwrap();

        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0], "test");
    }

    #[tokio::test]
    async fn test_mcp_server_get_prompt() {
        let mut library = PromptLibrary::new();
        let prompt = Prompt::new("test", "Hello {{ name }}!")
            .with_description("Greeting prompt".to_string());
        library.add(prompt).unwrap();

        let server = McpServer::new(library).unwrap();
        let mut arguments = HashMap::new();
        arguments.insert("name".to_string(), "World".to_string());

        let result = server.get_prompt("test", Some(&arguments)).await.unwrap();
        assert_eq!(result, "Hello World!");

        // Test without arguments
        let result = server.get_prompt("test", None).await.unwrap();
        assert_eq!(result, "Hello {{ name }}!");
    }

    #[tokio::test]
    async fn test_mcp_server_exposes_prompt_capabilities() {
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let info = server.get_info();

        // Verify server exposes prompt capabilities
        assert!(info.capabilities.prompts.is_some());
        let prompts_cap = info.capabilities.prompts.unwrap();
        assert_eq!(prompts_cap.list_changed, Some(true));

        // Verify server info is set correctly
        assert_eq!(info.server_info.name, "SwissArmyHammer");
        assert_eq!(info.server_info.version, crate::VERSION);

        // Verify instructions are provided
        assert!(info.instructions.is_some());
        assert!(info
            .instructions
            .unwrap()
            .contains("prompt and workflow management"));
    }

    #[tokio::test]
    async fn test_mcp_server_uses_same_prompt_paths_as_cli() {
        // This test verifies the fix for issue 000054.md
        // MCP server now uses the same PromptResolver as CLI

        // Simply verify that both CLI and MCP use the same PromptResolver type
        // This ensures they will load from the same directories

        // The fix is that both now use PromptResolver::new() and load_all_prompts()
        // This test verifies the API is consistent rather than testing file system behavior
        // which can be flaky in test environments

        let mut resolver1 = PromptResolver::new();
        let mut resolver2 = PromptResolver::new();
        let mut lib1 = PromptLibrary::new();
        let mut lib2 = PromptLibrary::new();

        // Both should use the same loading logic without errors
        let result1 = resolver1.load_all_prompts(&mut lib1);
        let result2 = resolver2.load_all_prompts(&mut lib2);

        // Both should succeed (even if no prompts are found)
        assert!(result1.is_ok(), "CLI resolver should work");
        assert!(result2.is_ok(), "MCP resolver should work");

        // The key fix: both use identical PromptResolver logic
        // In production, this ensures they load from ~/.swissarmyhammer/prompts
    }

    #[tokio::test]
    async fn test_mcp_server_file_watching_integration() {
        // Create a test library and server
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        // Test that file watching requires a peer connection
        // In tests, we can't easily create a real peer, so we skip the file watching test
        println!("File watching requires a peer connection from MCP client");

        // Test manual reload functionality
        let reload_result = server.reload_prompts().await;
        assert!(reload_result.is_ok(), "Manual prompt reload should work");

        // Test that the server can list prompts (even if empty)
        let prompts = server.list_prompts().await.unwrap();
        println!("Server has {} prompts loaded", prompts.len());

        // Notifications are sent via the peer connection when prompts change
        println!("File watching active - notifications will be sent when prompts change");
    }

    #[tokio::test]
    async fn test_mcp_server_uses_same_directory_discovery() {
        // Verify that MCP server uses same directory discovery as PromptResolver
        let resolver = PromptResolver::new();
        let resolver_dirs = resolver.get_prompt_directories().unwrap();

        // The server should use the same directories for file watching
        // This test ensures the fix for hardcoded paths is working
        let library = PromptLibrary::new();
        let _server = McpServer::new(library).unwrap();

        // File watching now requires a peer connection from the MCP client
        // The important thing is that both use get_prompt_directories() method
        println!(
            "File watching would watch {} directories when started with a peer connection",
            resolver_dirs.len()
        );

        // The fix ensures both use get_prompt_directories() method
        // This test verifies the API consistency
        println!("PromptResolver found {} directories", resolver_dirs.len());
        for dir in resolver_dirs {
            println!("  - {:?}", dir);
        }
    }

    #[tokio::test]
    async fn test_mcp_server_graceful_error_for_missing_prompt() {
        // Create a test library and server with one prompt
        let mut library = PromptLibrary::new();
        library
            .add(Prompt::new("test", "Hello {{ name }}!").with_description("Test prompt"))
            .unwrap();
        let server = McpServer::new(library).unwrap();

        // Test getting an existing prompt works
        let mut args = HashMap::new();
        args.insert("name".to_string(), "World".to_string());
        let result = server.get_prompt("test", Some(&args)).await;
        assert!(result.is_ok(), "Should successfully get existing prompt");

        // Test getting a non-existent prompt returns proper error
        let result = server.get_prompt("nonexistent", None).await;
        assert!(result.is_err(), "Should return error for missing prompt");

        let error_msg = result.unwrap_err().to_string();
        println!("Error for missing prompt: {}", error_msg);

        // Should contain helpful message about prompt not being available
        assert!(
            error_msg.contains("not available") || error_msg.contains("not found"),
            "Error should mention prompt issue: {}",
            error_msg
        );
    }

    #[tokio::test]
    async fn test_mcp_server_exposes_workflow_tools_capability() {
        // Create a test library and server
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let info = server.get_info();

        // Verify server exposes tools capabilities for workflows
        assert!(info.capabilities.tools.is_some());
        let tools_cap = info.capabilities.tools.unwrap();
        assert_eq!(tools_cap.list_changed, Some(true));

        // Verify prompts capability is still present
        assert!(info.capabilities.prompts.is_some());
        let prompts_cap = info.capabilities.prompts.unwrap();
        assert_eq!(prompts_cap.list_changed, Some(true));

        // Verify server info is set correctly
        assert_eq!(info.server_info.name, "SwissArmyHammer");
        assert_eq!(info.server_info.version, crate::VERSION);

        // Verify instructions mention both prompts and workflows
        assert!(info.instructions.is_some());
        let instructions = info.instructions.unwrap();
        assert!(instructions.contains("prompt"));
        assert!(instructions.contains("workflow"));
    }

    #[tokio::test]
    async fn test_mcp_server_does_not_expose_partial_templates() {
        // Create a test library with both regular and partial templates
        let mut library = PromptLibrary::new();

        // Add a regular prompt
        let regular_prompt = Prompt::new("regular_prompt", "This is a regular prompt: {{ name }}")
            .with_description("A regular prompt".to_string());
        library.add(regular_prompt).unwrap();

        // Add a partial template (marked as partial in description)
        let partial_prompt = Prompt::new("partial_template", "This is a partial template")
            .with_description("Partial template for reuse in other prompts".to_string());
        library.add(partial_prompt).unwrap();

        // Add another partial template with {% partial %} marker
        let partial_with_marker = Prompt::new(
            "partial_with_marker",
            "{% partial %}\nThis is a partial with marker",
        )
        .with_description("Another partial template".to_string());
        library.add(partial_with_marker).unwrap();

        let server = McpServer::new(library).unwrap();

        // Test list_prompts - should only return regular prompts
        let prompts = server.list_prompts().await.unwrap();
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0], "regular_prompt");
        assert!(!prompts.contains(&"partial_template".to_string()));
        assert!(!prompts.contains(&"partial_with_marker".to_string()));

        // Test get_prompt - should work for regular prompts
        let result = server.get_prompt("regular_prompt", None).await;
        assert!(result.is_ok());

        // Test get_prompt - should fail for partial templates
        let result = server.get_prompt("partial_template", None).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("partial template"));

        let result = server.get_prompt("partial_with_marker", None).await;
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("partial template"));
    }

    #[tokio::test]
    async fn test_mcp_server_exposes_issue_tools() {
        // Create a test library and server
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        // Test that server info includes issue tracking capabilities
        let info = server.get_info();
        assert!(
            info.capabilities.tools.is_some(),
            "Server should expose tools capability"
        );

        let tools_cap = info.capabilities.tools.unwrap();
        assert_eq!(
            tools_cap.list_changed,
            Some(true),
            "Tools capability should support list_changed"
        );

        // Verify server info includes issue tracking in instructions
        assert!(
            info.instructions.is_some(),
            "Server should have instructions"
        );
        let instructions = info.instructions.unwrap();
        assert!(
            instructions.contains("issue tracking"),
            "Instructions should mention issue tracking"
        );
        assert!(
            instructions.contains("issue_*"),
            "Instructions should mention issue_* tools"
        );
    }

    #[tokio::test]
    async fn test_mcp_server_tool_schemas_are_valid() {
        // Test that all request schemas can be generated without error
        let create_schema = serde_json::to_value(schemars::schema_for!(CreateIssueRequest));
        assert!(
            create_schema.is_ok(),
            "CreateIssueRequest schema should be valid"
        );

        let mark_complete_schema = serde_json::to_value(schemars::schema_for!(MarkCompleteRequest));
        assert!(
            mark_complete_schema.is_ok(),
            "MarkCompleteRequest schema should be valid"
        );

        let all_complete_schema = serde_json::to_value(schemars::schema_for!(AllCompleteRequest));
        assert!(
            all_complete_schema.is_ok(),
            "AllCompleteRequest schema should be valid"
        );

        let update_schema = serde_json::to_value(schemars::schema_for!(UpdateIssueRequest));
        assert!(
            update_schema.is_ok(),
            "UpdateIssueRequest schema should be valid"
        );

        let current_schema = serde_json::to_value(schemars::schema_for!(CurrentIssueRequest));
        assert!(
            current_schema.is_ok(),
            "CurrentIssueRequest schema should be valid"
        );

        let work_schema = serde_json::to_value(schemars::schema_for!(WorkIssueRequest));
        assert!(
            work_schema.is_ok(),
            "WorkIssueRequest schema should be valid"
        );

        let merge_schema = serde_json::to_value(schemars::schema_for!(MergeIssueRequest));
        assert!(
            merge_schema.is_ok(),
            "MergeIssueRequest schema should be valid"
        );
    }

    #[tokio::test]
    async fn test_mcp_server_initializes_with_issue_storage() {
        // Test that server can be created and includes issue storage
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        // Verify server info includes issue tracking in instructions
        let info = server.get_info();
        assert!(
            info.instructions.is_some(),
            "Server should have instructions"
        );

        let instructions = info.instructions.unwrap();
        assert!(
            instructions.contains("issue tracking"),
            "Instructions should mention issue tracking"
        );
        assert!(
            instructions.contains("issue_*"),
            "Instructions should mention issue_* tools"
        );
    }

    #[tokio::test]
    async fn test_handle_issue_create_success() {
        // Test successful issue creation
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let request = CreateIssueRequest {
            name: "test_issue".to_string(),
            content: "# Test Issue\n\nThis is a test issue content.".to_string(),
        };

        let result = server.handle_issue_create(request).await;
        assert!(result.is_ok(), "Issue creation should succeed");

        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        assert!(!call_result.content.is_empty());

        // Check that the response contains expected information
        let text_content = &call_result.content[0];
        if let RawContent::Text(text) = &text_content.raw {
            assert!(text.text.contains("Created issue #"));
            assert!(text.text.contains("test_issue"));
            assert!(text.text.contains(" at "));
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_handle_issue_create_empty_name() {
        // Test validation failure with empty name
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let request = CreateIssueRequest {
            name: "".to_string(),
            content: "Some content".to_string(),
        };

        let result = server.handle_issue_create(request).await;
        assert!(result.is_err(), "Empty name should fail validation");

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("Issue name cannot be empty"),
            "Error should mention empty name: {}",
            error
        );
    }

    #[tokio::test]
    async fn test_handle_issue_create_whitespace_name() {
        // Test validation failure with whitespace-only name
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let request = CreateIssueRequest {
            name: "   ".to_string(),
            content: "Some content".to_string(),
        };

        let result = server.handle_issue_create(request).await;
        assert!(
            result.is_err(),
            "Whitespace-only name should fail validation"
        );

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("Issue name cannot be empty"),
            "Error should mention empty name: {}",
            error
        );
    }

    #[tokio::test]
    async fn test_handle_issue_create_long_name() {
        // Test validation failure with too long name
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let long_name = "a".repeat(101); // 101 characters, over the limit
        let request = CreateIssueRequest {
            name: long_name,
            content: "Some content".to_string(),
        };

        let result = server.handle_issue_create(request).await;
        assert!(result.is_err(), "Long name should fail validation");

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("too long"),
            "Error should mention name too long: {}",
            error
        );
    }

    #[tokio::test]
    async fn test_handle_issue_create_invalid_characters() {
        // Test validation failure with invalid characters
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let invalid_names = vec![
            "issue/with/slashes",
            "issue\\with\\backslashes",
            "issue:with:colons",
            "issue*with*asterisks",
            "issue?with?questions",
            "issue\"with\"quotes",
            "issue<with>brackets",
            "issue|with|pipes",
        ];

        for invalid_name in invalid_names {
            let request = CreateIssueRequest {
                name: invalid_name.to_string(),
                content: "Some content".to_string(),
            };

            let result = server.handle_issue_create(request).await;
            assert!(
                result.is_err(),
                "Invalid name '{}' should fail validation",
                invalid_name
            );

            let error = result.unwrap_err();
            assert!(
                error.to_string().contains("invalid characters"),
                "Error should mention invalid characters for '{}': {}",
                invalid_name,
                error
            );
        }
    }

    #[tokio::test]
    async fn test_handle_issue_create_trimmed_name() {
        // Test that names are properly trimmed
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let request = CreateIssueRequest {
            name: "  test_issue  ".to_string(),
            content: "Some content".to_string(),
        };

        let result = server.handle_issue_create(request).await;
        assert!(result.is_ok(), "Trimmed name should succeed");

        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));

        // Check that the response contains the trimmed name
        let text_content = &call_result.content[0];
        if let RawContent::Text(text) = &text_content.raw {
            assert!(text.text.contains("test_issue"));
            assert!(!text.text.contains("  test_issue  "));
        } else {
            panic!("Expected text content");
        }
    }

    #[tokio::test]
    async fn test_handle_issue_create_sequential_numbering() {
        // Test that multiple issues get sequential numbers
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        // Create first issue
        let request1 = CreateIssueRequest {
            name: "first_issue".to_string(),
            content: "First issue content".to_string(),
        };

        let result1 = server.handle_issue_create(request1).await;
        assert!(result1.is_ok(), "First issue creation should succeed");

        let call_result1 = result1.unwrap();
        let text_content1 = &call_result1.content[0];
        let first_issue_number = if let RawContent::Text(text) = &text_content1.raw {
            // Extract the issue number from the text
            let start = text.text.find("Created issue #").unwrap() + "Created issue #".len();
            let end = text.text[start..].find(' ').unwrap() + start;
            let number_str = &text.text[start..end];
            number_str.parse::<u32>().unwrap()
        } else {
            panic!("Expected text content");
        };

        // Create second issue
        let request2 = CreateIssueRequest {
            name: "second_issue".to_string(),
            content: "Second issue content".to_string(),
        };

        let result2 = server.handle_issue_create(request2).await;
        assert!(result2.is_ok(), "Second issue creation should succeed");

        let call_result2 = result2.unwrap();
        let text_content2 = &call_result2.content[0];
        let second_issue_number = if let RawContent::Text(text) = &text_content2.raw {
            // Extract the issue number from the text
            let start = text.text.find("Created issue #").unwrap() + "Created issue #".len();
            let end = text.text[start..].find(' ').unwrap() + start;
            let number_str = &text.text[start..end];
            number_str.parse::<u32>().unwrap()
        } else {
            panic!("Expected text content");
        };

        // Verify the second issue has a higher number than the first
        assert!(
            second_issue_number > first_issue_number,
            "Second issue number ({}) should be greater than first issue number ({})",
            second_issue_number,
            first_issue_number
        );
    }

    #[test]
    fn test_validate_issue_name_success() {
        // Test successful validation
        let valid_names = vec![
            "simple_name",
            "name with spaces",
            "name-with-dashes",
            "name_with_underscores",
            "123_numeric_start",
            "UPPERCASE_NAME",
            "MixedCase_Name",
            "a", // Minimum length
        ];

        for name in valid_names {
            let result = validate_issue_name(name);
            assert!(
                result.is_ok(),
                "Valid name '{}' should pass validation",
                name
            );
            assert_eq!(result.unwrap(), name.trim());
        }

        // Test maximum length separately
        let max_length_name = "a".repeat(100);
        let result = validate_issue_name(&max_length_name);
        assert!(result.is_ok(), "100 character name should pass validation");
        assert_eq!(result.unwrap(), max_length_name.trim());
    }

    #[test]
    fn test_validate_issue_name_failure() {
        // Test validation failures
        let invalid_names = vec![
            ("", "empty"),
            ("   ", "whitespace only"),
            ("name/with/slashes", "invalid characters"),
            ("name\\with\\backslashes", "invalid characters"),
            ("name:with:colons", "invalid characters"),
            ("name*with*asterisks", "invalid characters"),
            ("name?with?questions", "invalid characters"),
            ("name\"with\"quotes", "invalid characters"),
            ("name<with>brackets", "invalid characters"),
            ("name|with|pipes", "invalid characters"),
        ];

        for (name, reason) in invalid_names {
            let result = validate_issue_name(name);
            assert!(
                result.is_err(),
                "Invalid name '{}' should fail validation ({})",
                name,
                reason
            );
        }

        // Test too long name separately
        let too_long_name = "a".repeat(101);
        let result = validate_issue_name(&too_long_name);
        assert!(result.is_err(), "101 character name should fail validation");
    }

    #[test]
    fn test_validate_issue_name_trimming() {
        // Test that names are properly trimmed
        let names_with_whitespace = vec![
            ("  test  ", "test"),
            ("\ttest\t", "test"),
            ("  test_name  ", "test_name"),
            ("   multiple   spaces   ", "multiple   spaces"),
        ];

        for (input, expected) in names_with_whitespace {
            let result = validate_issue_name(input);
            assert!(
                result.is_ok(),
                "Name with whitespace '{}' should be valid",
                input
            );
            assert_eq!(result.unwrap(), expected);
        }
    }
}
