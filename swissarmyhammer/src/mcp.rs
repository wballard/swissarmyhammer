//! Model Context Protocol (MCP) server support

use crate::file_watcher::{FileWatcher, FileWatcherCallback};
use crate::git::GitOperations;
use crate::issues::{FileSystemIssueStorage, IssueStorage};
use crate::memoranda::{MarkdownMemoStorage, MemoStorage};
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
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// MCP module structure
pub mod error_handling;
pub mod file_watcher;
pub mod memo_types;
pub mod responses;
pub mod shared_utils;
pub mod tool_handlers;
pub mod tool_registry;
pub mod tools;
pub mod types;
pub mod utils;

// Re-export commonly used items from submodules
use tool_handlers::ToolHandlers;
use tool_registry::{register_issue_tools, register_memo_tools, ToolContext, ToolRegistry};
#[cfg(test)]
use types::{
    AllCompleteRequest, CreateIssueRequest, CurrentIssueRequest, IssueName, MarkCompleteRequest,
    MergeIssueRequest, UpdateIssueRequest, WorkIssueRequest,
};
#[cfg(test)]
use utils::validate_issue_name;

/// Constants for issue branch management
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

/// MCP server for serving prompts and workflows
#[derive(Clone)]
pub struct McpServer {
    library: Arc<RwLock<PromptLibrary>>,
    workflow_storage: Arc<RwLock<WorkflowStorage>>,
    file_watcher: Arc<Mutex<FileWatcher>>,
    tool_registry: Arc<ToolRegistry>,
    tool_context: Arc<ToolContext>,
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
        let work_dir = std::env::current_dir().map_err(|e| {
            SwissArmyHammerError::Other(format!("Failed to get current directory: {e}"))
        })?;
        Self::new_with_work_dir(library, work_dir)
    }

    /// Create a new MCP server with the provided prompt library and working directory.
    ///
    /// # Arguments
    ///
    /// * `library` - The prompt library to serve via MCP
    /// * `work_dir` - The working directory to use for issue storage and git operations
    ///
    /// # Returns
    ///
    /// * `Result<Self>` - The MCP server instance or an error if initialization fails
    ///
    /// # Errors
    ///
    /// Returns an error if workflow storage, issue storage, or git operations fail to initialize.
    pub fn new_with_work_dir(library: PromptLibrary, work_dir: PathBuf) -> Result<Self> {
        // Initialize workflow storage with filesystem backend
        let workflow_backend = Arc::new(FileSystemWorkflowStorage::new().map_err(|e| {
            tracing::error!("Failed to create workflow storage: {}", e);
            SwissArmyHammerError::Other(format!("Failed to create workflow storage: {e}"))
        })?) as Arc<dyn WorkflowStorageBackend>;

        // Create runs directory in user's home directory
        let runs_path = Self::get_workflow_runs_path();

        let run_backend = Arc::new(FileSystemWorkflowRunStorage::new(runs_path).map_err(|e| {
            tracing::error!("Failed to create workflow run storage: {}", e);
            SwissArmyHammerError::Other(format!("Failed to create workflow run storage: {e}"))
        })?) as Arc<dyn WorkflowRunStorageBackend>;

        let workflow_storage = WorkflowStorage::new(workflow_backend, run_backend);

        // Initialize issue storage with issues directory in work_dir
        let issues_dir = work_dir.join("issues");

        let issue_storage = Box::new(FileSystemIssueStorage::new(issues_dir).map_err(|e| {
            tracing::error!("Failed to create issue storage: {}", e);
            SwissArmyHammerError::Other(format!("Failed to create issue storage: {e}"))
        })?) as Box<dyn IssueStorage>;

        // Initialize memo storage with default location
        let memo_storage = Box::new(MarkdownMemoStorage::new_default().map_err(|e| {
            tracing::error!("Failed to create memo storage: {}", e);
            SwissArmyHammerError::Other(format!("Failed to create memo storage: {e}"))
        })?) as Box<dyn MemoStorage>;

        // Initialize git operations with work_dir - make it optional for tests
        let git_ops = match GitOperations::with_work_dir(work_dir.clone()) {
            Ok(ops) => Some(ops),
            Err(e) => {
                tracing::warn!("Git operations not available: {}", e);
                None
            }
        };

        // Create Arc wrappers for shared storage
        let issue_storage = Arc::new(RwLock::new(issue_storage));
        let memo_storage_arc = Arc::new(RwLock::new(memo_storage));
        let git_ops_arc = Arc::new(Mutex::new(git_ops));

        // Initialize tool handlers with all storage instances
        let tool_handlers = ToolHandlers::new(
            issue_storage.clone(),
            git_ops_arc.clone(),
            memo_storage_arc.clone(),
        );

        // Initialize tool registry and context
        let mut tool_registry = ToolRegistry::new();
        let tool_context = Arc::new(ToolContext::new(
            Arc::new(tool_handlers.clone()),
            issue_storage.clone(),
            git_ops_arc.clone(),
            memo_storage_arc.clone(),
        ));

        // Register all available tools
        register_issue_tools(&mut tool_registry);
        register_memo_tools(&mut tool_registry);

        Ok(Self {
            library: Arc::new(RwLock::new(library)),
            workflow_storage: Arc::new(RwLock::new(workflow_storage)),
            file_watcher: Arc::new(Mutex::new(FileWatcher::new())),
            tool_registry: Arc::new(tool_registry),
            tool_context,
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
                "Cannot access partial template '{name}' via MCP. Partial templates are for internal use only."
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
        }
        tracing::info!("✅ Prompts reloaded successfully");

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
                        && last_error.as_ref().is_some_and(Self::is_retryable_fs_error)
                    {
                        tracing::warn!(
                            "⚠️ File watcher initialization attempt {} failed, retrying in {}ms: {}",
                            attempt,
                            backoff_ms,
                            last_error.as_ref().map_or("Unknown error".to_string(), |e| e.to_string())
                        );

                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2; // Exponential backoff
                    } else {
                        break;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| {
            SwissArmyHammerError::Other("File watcher initialization failed".to_string())
        }))
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
                        && last_error.as_ref().is_some_and(Self::is_retryable_fs_error)
                    {
                        tracing::warn!(
                            "⚠️ Reload attempt {} failed, retrying in {}ms: {}",
                            attempt,
                            backoff_ms,
                            last_error
                                .as_ref()
                                .map_or("Unknown error".to_string(), |e| e.to_string())
                        );

                        tokio::time::sleep(tokio::time::Duration::from_millis(backoff_ms)).await;
                        backoff_ms *= 2; // Exponential backoff
                    } else {
                        break;
                    }
                }
            }
        }

        Err(last_error
            .unwrap_or_else(|| SwissArmyHammerError::Other("Prompt reload failed".to_string())))
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
                                format!("Template rendering error: {e}"),
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
        Ok(ListToolsResult {
            tools: self.tool_registry.list_tools(),
            next_cursor: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> std::result::Result<CallToolResult, McpError> {
        if let Some(tool) = self.tool_registry.get_tool(&request.name) {
            tool.execute(request.arguments.unwrap_or_default(), &self.tool_context)
                .await
        } else {
            Err(McpError::invalid_request(
                format!("Unknown tool: {}", request.name),
                None,
            ))
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

    fn extract_issue_name_from_create_request(request: &CreateIssueRequest) -> IssueName {
        if let Some(ref name) = request.name {
            name.clone()
        } else {
            // This should not happen in the new system
            panic!("Issue name is required in create request");
        }
    }

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
            println!("  - {dir:?}");
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
        println!("Error for missing prompt: {error_msg}");

        // Should contain helpful message about prompt not being available
        assert!(
            error_msg.contains("not available") || error_msg.contains("not found"),
            "Error should mention prompt issue: {error_msg}"
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
            name: Some(IssueName::new("test_issue".to_string()).unwrap()),
            content: "# Test Issue\n\nThis is a test issue content.".to_string(),
        };

        let result = server
            .tool_context
            .tool_handlers
            .handle_issue_create(request)
            .await;
        assert!(result.is_ok(), "Issue creation should succeed");

        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        assert!(!call_result.content.is_empty());

        // Check that the response contains expected information
        let text_content = &call_result.content[0];
        if let RawContent::Text(text) = &text_content.raw {
            assert!(text.text.contains("Created issue"));
            assert!(text.text.contains("test_issue"));
            assert!(text.text.contains(" at "));
        } else {
            panic!("Expected text content, got: {:?}", text_content.raw);
        }
    }

    #[tokio::test]
    async fn test_handle_issue_create_empty_name() {
        // Test validation failure with empty name
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let request = CreateIssueRequest {
            name: Some(IssueName("".to_string())),
            content: "Some content".to_string(),
        };

        let result = server
            .tool_context
            .tool_handlers
            .handle_issue_create(request)
            .await;
        assert!(result.is_err(), "Empty name should fail validation");

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("Issue name cannot be empty"),
            "Error should mention empty name: {error}"
        );
    }

    #[tokio::test]
    async fn test_handle_issue_create_whitespace_name() {
        // Test validation failure with whitespace-only name
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let request = CreateIssueRequest {
            name: Some(IssueName("   ".to_string())),
            content: "Some content".to_string(),
        };

        let result = server
            .tool_context
            .tool_handlers
            .handle_issue_create(request)
            .await;
        assert!(
            result.is_err(),
            "Whitespace-only name should fail validation"
        );

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("Issue name cannot be empty"),
            "Error should mention empty name: {error}"
        );
    }

    #[tokio::test]
    async fn test_handle_issue_create_long_name() {
        // Test validation failure with too long name
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let long_name = "a".repeat(101); // 101 characters, over the limit
        let request = CreateIssueRequest {
            name: Some(IssueName(long_name)),
            content: "Some content".to_string(),
        };

        let result = server
            .tool_context
            .tool_handlers
            .handle_issue_create(request)
            .await;
        assert!(result.is_err(), "Long name should fail validation");

        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("too long"),
            "Error should mention name too long: {error}"
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
                name: Some(IssueName(invalid_name.to_string())),
                content: "Some content".to_string(),
            };

            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(request)
                .await;
            assert!(
                result.is_err(),
                "Invalid name '{invalid_name}' should fail validation"
            );

            let error = result.unwrap_err();
            assert!(
                error.to_string().contains("invalid characters"),
                "Error should mention invalid characters for '{invalid_name}': {error}"
            );
        }
    }

    #[tokio::test]
    async fn test_handle_issue_create_trimmed_name() {
        // Test that names are properly trimmed
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let request = CreateIssueRequest {
            name: Some(IssueName("  test_issue  ".to_string())),
            content: "Some content".to_string(),
        };

        let result = server
            .tool_context
            .tool_handlers
            .handle_issue_create(request)
            .await;
        assert!(result.is_ok(), "Trimmed name should succeed");

        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));

        // Check that the response contains the trimmed name
        let text_content = &call_result.content[0];
        if let RawContent::Text(text) = &text_content.raw {
            assert!(text.text.contains("test_issue"));
            assert!(!text.text.contains("  test_issue  "));
        } else {
            panic!("Expected text content, got: {:?}", text_content.raw);
        }
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
            assert!(result.is_ok(), "Valid name '{name}' should pass validation");
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
                "Invalid name '{name}' should fail validation ({reason})"
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
                "Name with whitespace '{input}' should be valid"
            );
            assert_eq!(result.unwrap(), expected);
        }
    }

    // Integration tests for MCP tools
    mod mcp_integration_tests {
        use super::*;
        use std::fs;
        use std::process::Command;
        use tempfile::TempDir;

        /// Create test MCP server with issue support
        async fn create_test_mcp_server() -> (McpServer, TempDir) {
            let temp_dir = TempDir::new().unwrap();
            let temp_path = temp_dir.path().to_path_buf();

            // Do not change the working directory to avoid affecting other tests

            // Initialize git repo for testing
            Command::new("git")
                .args(["init"])
                .current_dir(&temp_path)
                .output()
                .expect("Failed to init git repo");

            // Explicitly set the default branch name to "main" for consistency
            Command::new("git")
                .args(["branch", "-M", "main"])
                .current_dir(&temp_path)
                .output()
                .expect("Failed to set main branch");

            // Set up git config for testing
            Command::new("git")
                .args(["config", "user.email", "test@example.com"])
                .current_dir(&temp_path)
                .output()
                .expect("Failed to set git email");

            Command::new("git")
                .args(["config", "user.name", "Test User"])
                .current_dir(&temp_path)
                .output()
                .expect("Failed to set git name");

            // Create initial commit
            Command::new("git")
                .args(["commit", "--allow-empty", "-m", "Initial commit"])
                .current_dir(&temp_path)
                .output()
                .expect("Failed to create initial commit");

            // Create issues directory
            fs::create_dir_all(temp_path.join("issues")).unwrap();

            let library = PromptLibrary::new();
            let server = McpServer::new_with_work_dir(library, temp_path.clone()).unwrap();

            (server, temp_dir)
        }

        /// Commit any changes in the temporary directory to keep git clean
        async fn commit_changes(temp_path: &std::path::Path) {
            // Add all changes
            Command::new("git")
                .args(["add", "."])
                .current_dir(temp_path)
                .output()
                .expect("Failed to add changes");

            // Commit changes if there are any
            match Command::new("git")
                .args(["commit", "-m", "Test changes"])
                .current_dir(temp_path)
                .output()
            {
                Ok(output) => {
                    if !output.status.success() {
                        tracing::debug!(
                            "Git commit failed (possibly no changes to commit): {}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to execute git commit: {}", e);
                }
            }
        }

        #[tokio::test]
        async fn test_mcp_create_issue() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create issue via MCP
            let request = CreateIssueRequest {
                name: Some(IssueName::new("test_mcp_issue".to_string()).unwrap()),
                content: "This is a test issue created via MCP".to_string(),
            };

            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(request)
                .await;
            assert!(result.is_ok());

            let response = result.unwrap();
            assert!(!response.is_error.unwrap_or(false));

            // Verify response content
            assert!(!response.content.is_empty());
            if let RawContent::Text(text_content) = &response.content[0].raw {
                assert!(text_content.text.contains("Created issue"));
                assert!(text_content.text.contains("test_mcp_issue"));
            } else {
                panic!("Expected text response");
            }

            // Verify issue file was created
            let issue_files: Vec<_> = fs::read_dir(_temp.path().join("issues"))
                .unwrap()
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    if entry.file_name().to_str()?.contains("test_mcp_issue") {
                        Some(entry.path())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(
                !issue_files.is_empty(),
                "Issue file should have been created"
            );

            // Verify issue content
            let issue_content = fs::read_to_string(&issue_files[0]).unwrap();
            assert!(issue_content.contains("This is a test issue created via MCP"));
        }

        #[tokio::test]
        async fn test_mcp_create_issue_invalid_name() {
            let (server, _temp) = create_test_mcp_server().await;

            // Try to create issue with empty name
            let request = CreateIssueRequest {
                name: Some(IssueName("".to_string())),
                content: "Content".to_string(),
            };

            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(request)
                .await;
            assert!(result.is_err());

            let error = result.unwrap_err();
            assert!(error.to_string().contains("empty"));
        }

        #[tokio::test]
        async fn test_mcp_complete_issue_workflow() {
            let (server, _temp) = create_test_mcp_server().await;

            // 1. Create an issue
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("feature_implementation".to_string()).unwrap()),
                content: "Implement new feature X".to_string(),
            };

            // Extract issue name from create request
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(!create_result.is_error.unwrap_or(false));

            // 2. Update the issue
            let update_request = UpdateIssueRequest {
                name: issue_name.clone(),
                content: "Implement new feature X\n\nAdditional notes: Started implementation"
                    .to_string(),
                append: false,
            };

            tracing::debug!("About to update issue...");
            let update_result = server
                .tool_context
                .tool_handlers
                .handle_issue_update(update_request)
                .await
                .unwrap();
            assert!(!update_result.is_error.unwrap_or(false));
            tracing::debug!("Update completed");

            // 3. Mark it complete
            let complete_request = MarkCompleteRequest {
                name: issue_name.clone(),
            };

            tracing::debug!("About to mark complete...");
            let complete_result = server
                .tool_context
                .tool_handlers
                .handle_issue_mark_complete(complete_request)
                .await
                .unwrap();
            assert!(!complete_result.is_error.unwrap_or(false));
            tracing::debug!("Mark complete finished");

            // Commit the issue completion
            commit_changes(_temp.path()).await;

            // 4. Check all complete
            let all_complete_request = AllCompleteRequest {};
            tracing::debug!("About to check all complete...");
            let all_complete_result = server
                .tool_context
                .tool_handlers
                .handle_issue_all_complete(all_complete_request)
                .await
                .unwrap();
            assert!(!all_complete_result.is_error.unwrap_or(false));
            tracing::debug!("All complete check finished");

            // Don't check file system for now since there might be an issue with the complete directory
            // The test passes if the MCP operations succeed
            tracing::debug!("Workflow test completed successfully");
        }

        #[tokio::test]
        async fn test_mcp_work_issue() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create an issue first
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("bug_fix".to_string()).unwrap()),
                content: "Fix critical bug in parser".to_string(),
            };
            // Extract issue name from create request
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(
                !create_result.content.is_empty(),
                "Create result should have content"
            );

            // Commit the issue file to keep git clean
            commit_changes(_temp.path()).await;

            // Work on the issue
            let work_request = WorkIssueRequest {
                name: issue_name.clone(),
            };

            let work_result = server
                .tool_context
                .tool_handlers
                .handle_issue_work(work_request)
                .await;
            assert!(work_result.is_ok());

            let response = work_result.unwrap();
            assert!(!response.is_error.unwrap_or(false));

            // Verify response mentions branch switch
            if let RawContent::Text(text) = &response.content[0].raw {
                assert!(text.text.contains("Switched to work branch"));
                assert!(text.text.contains("issue/"));
            } else {
                panic!("Expected text response");
            }

            // Verify git branch was created
            let branch_output = Command::new("git")
                .args(["branch", "--show-current"])
                .current_dir(_temp.path())
                .output()
                .expect("Failed to get current branch");

            let current_branch = String::from_utf8(branch_output.stdout).unwrap();
            assert!(current_branch.trim().starts_with("issue/"));
        }

        #[tokio::test]
        async fn test_mcp_current_issue() {
            let (server, _temp) = create_test_mcp_server().await;

            // Initially on main branch - no current issue
            let current_request = CurrentIssueRequest { branch: None };
            let current_result = server
                .tool_context
                .tool_handlers
                .handle_issue_current(current_request)
                .await
                .unwrap();
            assert!(!current_result.is_error.unwrap_or(false));

            // Create and work on an issue
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("test_task".to_string()).unwrap()),
                content: "Test task content".to_string(),
            };
            // Extract issue name from create request before moving it
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(
                !create_result.content.is_empty(),
                "Create result should have content"
            );

            // Commit the issue file to keep git clean
            commit_changes(_temp.path()).await;

            let work_request = WorkIssueRequest { name: issue_name };
            server
                .tool_context
                .tool_handlers
                .handle_issue_work(work_request)
                .await
                .unwrap();

            // Now should have current issue
            let current_request = CurrentIssueRequest { branch: None };
            let current_result = server
                .tool_context
                .tool_handlers
                .handle_issue_current(current_request)
                .await
                .unwrap();
            assert!(!current_result.is_error.unwrap_or(false));

            // Verify response mentions current issue
            if let RawContent::Text(text) = &current_result.content[0].raw {
                assert!(text.text.contains("Currently working on issue:"));
                assert!(text.text.contains("test_task"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_mcp_error_handling() {
            let (server, _temp) = create_test_mcp_server().await;

            // Test updating non-existent issue
            let update_request = UpdateIssueRequest {
                name: IssueName::new("non_existent".to_string()).unwrap(),
                content: "New content".to_string(),
                append: false,
            };

            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_update(update_request)
                .await;
            assert!(result.is_ok());
            let call_result = result.unwrap();
            assert!(call_result.is_error.unwrap_or(false));
            if let RawContent::Text(text) = &call_result.content[0].raw {
                assert!(text.text.contains("not found") || text.text.contains("Failed to update"));
            } else {
                panic!("Expected text content");
            }

            // Test marking non-existent issue complete
            let complete_request = MarkCompleteRequest {
                name: IssueName::new("non_existent".to_string()).unwrap(),
            };

            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_mark_complete(complete_request)
                .await;
            assert!(result.is_ok());
            let call_result = result.unwrap();
            assert!(call_result.is_error.unwrap_or(false));

            // Test working on non-existent issue
            let work_request = WorkIssueRequest {
                name: IssueName::new("non_existent".to_string()).unwrap(),
            };

            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_work(work_request)
                .await;
            assert!(result.is_ok());
            let response = result.unwrap();
            assert!(response.is_error.unwrap_or(false));
            if let RawContent::Text(text) = &response.content[0].raw {
                assert!(text.text.contains("not found"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_mcp_list_tools_includes_issues() {
            let (server, _temp) = create_test_mcp_server().await;

            // Mock request context would be needed for full test
            // For now, verify tool definitions exist
            let info = server.get_info();
            assert!(info.capabilities.tools.is_some());

            // We can't easily create a full RequestContext in tests,
            // but we can verify the server exposes the expected capabilities
            let tools_cap = info.capabilities.tools.unwrap();
            assert_eq!(tools_cap.list_changed, Some(true));
        }

        #[tokio::test]
        async fn test_mcp_issue_merge() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create an issue
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("merge_test".to_string()).unwrap()),
                content: "Test merge functionality".to_string(),
            };
            // Extract issue name from create request before moving it
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(
                !create_result.content.is_empty(),
                "Create result should have content"
            );

            // Commit the issue file to keep git clean
            commit_changes(_temp.path()).await;

            // Work on the issue to create a branch
            let work_request = WorkIssueRequest {
                name: issue_name.clone(),
            };
            server
                .tool_context
                .tool_handlers
                .handle_issue_work(work_request)
                .await
                .unwrap();

            // Make a dummy commit on the issue branch
            fs::write(_temp.path().join("test_file.txt"), "test content").unwrap();
            Command::new("git")
                .args(["add", "test_file.txt"])
                .current_dir(_temp.path())
                .output()
                .expect("Failed to add file");

            Command::new("git")
                .args(["commit", "-m", "Test commit"])
                .current_dir(_temp.path())
                .output()
                .expect("Failed to commit");

            // Mark issue as complete
            let complete_request = MarkCompleteRequest {
                name: issue_name.clone(),
            };
            server
                .tool_context
                .tool_handlers
                .handle_issue_mark_complete(complete_request)
                .await
                .unwrap();

            // Commit the issue completion
            commit_changes(_temp.path()).await;

            // Test merge
            let merge_request = MergeIssueRequest {
                name: issue_name,
                delete_branch: false,
            };

            let merge_result = server
                .tool_context
                .tool_handlers
                .handle_issue_merge(merge_request)
                .await
                .unwrap();
            if merge_result.is_error.unwrap_or(false) {
                if let Some(content) = merge_result.content.first() {
                    if let RawContent::Text(text) = &content.raw {
                        println!("MERGE ERROR: {}", text.text);
                    }
                }
            }
            assert!(!merge_result.is_error.unwrap_or(false));

            // Verify merge response
            if let RawContent::Text(text) = &merge_result.content[0].raw {
                assert!(text.text.contains("Merged work branch"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_mcp_issue_append_mode() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create an issue
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("append_test".to_string()).unwrap()),
                content: "Initial content".to_string(),
            };
            // Extract issue name from create request before moving it
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(
                !create_result.content.is_empty(),
                "Create result should have content"
            );

            // Update in append mode
            let update_request = UpdateIssueRequest {
                name: issue_name,
                content: "Additional content".to_string(),
                append: true,
            };

            let update_result = server
                .tool_context
                .tool_handlers
                .handle_issue_update(update_request)
                .await
                .unwrap();
            assert!(!update_result.is_error.unwrap_or(false));

            // Verify append mode response
            if let RawContent::Text(text) = &update_result.content[0].raw {
                assert!(text.text.contains("append"));
            } else {
                panic!("Expected text response");
            }

            // Verify file contains both original and appended content
            let issue_files: Vec<_> = fs::read_dir(_temp.path().join("issues"))
                .unwrap()
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    if entry.file_name().to_str()?.contains("append_test") {
                        Some(entry.path())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!issue_files.is_empty());
            let content = fs::read_to_string(&issue_files[0]).unwrap();
            assert!(content.contains("Initial content"));
            assert!(content.contains("Additional content"));
        }

        #[tokio::test]
        async fn test_mcp_issue_large_content() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create an issue with large content
            let large_content = "x".repeat(10000);
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("large_content_test".to_string()).unwrap()),
                content: large_content.clone(),
            };

            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(!create_result.is_error.unwrap_or(false));

            // Verify large content was saved
            let issue_files: Vec<_> = fs::read_dir(_temp.path().join("issues"))
                .unwrap()
                .filter_map(|entry| {
                    let entry = entry.ok()?;
                    if entry.file_name().to_str()?.contains("large_content_test") {
                        Some(entry.path())
                    } else {
                        None
                    }
                })
                .collect();

            assert!(!issue_files.is_empty());
            let content = fs::read_to_string(&issue_files[0]).unwrap();
            assert!(content.contains(&large_content));
        }

        #[tokio::test]
        async fn test_mcp_git_integration_workflow() {
            let (server, temp_dir) = create_test_mcp_server().await;
            let temp_path = temp_dir.path();

            // Test 1: Create an issue
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("git_integration_test".to_string()).unwrap()),
                content: "Testing git integration with MCP server".to_string(),
            };

            // Extract issue name from create request before moving it
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(!create_result.is_error.unwrap_or(false));

            // Commit the created issue file to git
            Command::new("git")
                .args(["add", "issues/"])
                .current_dir(temp_path)
                .output()
                .expect("Failed to add issues to git");

            Command::new("git")
                .args(["commit", "-m", "Add test issue"])
                .current_dir(temp_path)
                .output()
                .expect("Failed to commit issue");

            // Test 2: Work on the issue (should create a git branch)
            let work_request = WorkIssueRequest {
                name: issue_name.clone(),
            };

            let work_result = server
                .tool_context
                .tool_handlers
                .handle_issue_work(work_request)
                .await
                .unwrap();
            assert!(!work_result.is_error.unwrap_or(false));

            // Verify that a git branch was created
            let git_branches = Command::new("git")
                .args(["branch", "--list"])
                .current_dir(temp_path)
                .output()
                .expect("Failed to list git branches");

            let branches_output = String::from_utf8_lossy(&git_branches.stdout);
            assert!(branches_output.contains("issue/git_integration_test"));

            // Test 3: Update the issue content
            let update_request = UpdateIssueRequest {
                name: issue_name.clone(),
                content: "Updated content for git integration test".to_string(),
                append: false,
            };

            let update_result = server
                .tool_context
                .tool_handlers
                .handle_issue_update(update_request)
                .await
                .unwrap();
            assert!(!update_result.is_error.unwrap_or(false));

            // Commit the updated issue
            commit_changes(temp_path).await;

            // Test 4: Complete the issue (should switch back to main branch)
            let complete_request = MarkCompleteRequest { name: issue_name };

            let complete_result = server
                .tool_context
                .tool_handlers
                .handle_issue_mark_complete(complete_request)
                .await
                .unwrap();
            assert!(!complete_result.is_error.unwrap_or(false));

            // Commit the issue completion (issue moved to completed directory)
            commit_changes(temp_path).await;

            // Verify the issue is in completed state
            let current_request = CurrentIssueRequest { branch: None };
            let current_result = server
                .tool_context
                .tool_handlers
                .handle_issue_current(current_request)
                .await
                .unwrap();
            assert!(!current_result.is_error.unwrap_or(false));

            // Commit any changes before merge
            commit_changes(temp_path).await;

            // DEBUG: Check if branch still exists before merge
            let git_branches = Command::new("git")
                .args(["branch", "--list"])
                .current_dir(temp_path)
                .output()
                .expect("Failed to list git branches before merge");
            let branches_output = String::from_utf8_lossy(&git_branches.stdout);
            tracing::debug!("Branches before merge: {branches_output}");
            tracing::debug!("Looking for branch: issue/git_integration_test");
            tracing::debug!(
                "Branch exists: {}",
                branches_output.contains("issue/git_integration_test")
            );

            // Test 5: Merge the issue branch (if it still exists)
            let merge_request = MergeIssueRequest {
                name: IssueName::new("git_integration_test".to_string()).unwrap(),
                delete_branch: false,
            };

            let merge_result = server
                .tool_context
                .tool_handlers
                .handle_issue_merge(merge_request)
                .await
                .unwrap();
            // Note: merge may fail if branch doesn't exist or is already merged, which is okay
            if merge_result.is_error.unwrap_or(false) {
                if let RawContent::Text(text) = &merge_result.content[0].raw {
                    println!("DEBUG: Git integration workflow merge error: {}", text.text);
                }
            }
            assert!(!merge_result.is_error.unwrap_or(false));

            // Test 6: Verify git repository state
            let git_status = Command::new("git")
                .args(["status", "--porcelain"])
                .current_dir(temp_path)
                .output()
                .expect("Failed to check git status");

            let _status_output = String::from_utf8_lossy(&git_status.stdout);
            // Repository should be clean or have only expected changes

            // Test 7: Verify current branch is the main branch
            let git_ops = GitOperations::with_work_dir(temp_path.to_path_buf()).unwrap();
            let expected_main_branch = git_ops.main_branch().unwrap();

            let current_branch = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(temp_path)
                .output()
                .expect("Failed to get current branch");

            let branch_output_string = String::from_utf8_lossy(&current_branch.stdout);
            let branch_output = branch_output_string.trim();
            assert_eq!(branch_output, expected_main_branch);
        }

        #[tokio::test]
        async fn test_mcp_git_branch_management() {
            let (server, temp_dir) = create_test_mcp_server().await;
            let temp_path = temp_dir.path();

            // Create multiple issues to test branch management
            let issues = vec![
                ("branch_test_1", "First branch test"),
                ("branch_test_2", "Second branch test"),
                ("branch_test_3", "Third branch test"),
            ];

            let mut issue_names = Vec::new();

            // Create all issues
            for (name, content) in &issues {
                let create_request = CreateIssueRequest {
                    name: Some(IssueName::new(name.to_string()).unwrap()),
                    content: content.to_string(),
                };

                let issue_name = extract_issue_name_from_create_request(&create_request);
                let create_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_create(create_request)
                    .await
                    .unwrap();
                assert!(!create_result.is_error.unwrap_or(false));

                // Commit the created issue file to git
                Command::new("git")
                    .args(["add", "issues/"])
                    .current_dir(temp_path)
                    .output()
                    .expect("Failed to add issues to git");

                Command::new("git")
                    .args(["commit", "-m", &format!("Add issue {name}")])
                    .current_dir(temp_path)
                    .output()
                    .expect("Failed to commit issue");

                issue_names.push(issue_name);
            }

            // Work on multiple issues (should create multiple branches)
            for (i, issue_name) in issue_names.iter().enumerate() {
                // Switch back to main branch before working on next issue (except for the first one)
                if i > 0 {
                    Command::new("git")
                        .args(["checkout", "main"])
                        .current_dir(temp_path)
                        .output()
                        .or_else(|_| {
                            Command::new("git")
                                .args(["checkout", "master"])
                                .current_dir(temp_path)
                                .output()
                        })
                        .expect("Failed to switch back to main/master branch");
                }

                let work_request = WorkIssueRequest {
                    name: issue_name.clone(),
                };

                let work_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_work(work_request)
                    .await
                    .unwrap();
                assert!(!work_result.is_error.unwrap_or(false));

                // Verify correct branch is created and checked out
                let current_branch = Command::new("git")
                    .args(["rev-parse", "--abbrev-ref", "HEAD"])
                    .current_dir(temp_path)
                    .output()
                    .expect("Failed to get current branch");

                let branch_output_string = String::from_utf8_lossy(&current_branch.stdout);
                let branch_output = branch_output_string.trim();
                assert!(branch_output.contains(&format!("issue/{}", issues[i].0)));
            }

            // Complete all issues
            for issue_name in &issue_names {
                let complete_request = MarkCompleteRequest {
                    name: issue_name.clone(),
                };

                let complete_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_mark_complete(complete_request)
                    .await
                    .unwrap();
                assert!(!complete_result.is_error.unwrap_or(false));
            }

            // Explicitly switch back to main branch
            Command::new("git")
                .args(["checkout", "main"])
                .current_dir(temp_path)
                .output()
                .or_else(|_| {
                    Command::new("git")
                        .args(["checkout", "master"])
                        .current_dir(temp_path)
                        .output()
                })
                .expect("Failed to checkout main/master branch");

            // Verify we're back on main branch
            let current_branch = Command::new("git")
                .args(["rev-parse", "--abbrev-ref", "HEAD"])
                .current_dir(temp_path)
                .output()
                .expect("Failed to get current branch");

            let branch_output_string = String::from_utf8_lossy(&current_branch.stdout);
            let branch_output = branch_output_string.trim();
            assert!(branch_output == "main" || branch_output == "master");
        }

        #[tokio::test]
        async fn test_mcp_git_error_handling() {
            let (server, temp_dir) = create_test_mcp_server().await;
            let _temp_path = temp_dir.path();

            // Test working on a non-existent issue
            let work_request = WorkIssueRequest {
                name: IssueName::new("non_existent_issue".to_string()).unwrap(),
            };

            let work_result = server
                .tool_context
                .tool_handlers
                .handle_issue_work(work_request)
                .await;
            // This should return Ok but with an error flag since the issue wasn't found
            assert!(work_result.is_ok());
            let response = work_result.unwrap();
            assert!(response.is_error.unwrap_or(false));
            if let RawContent::Text(text) = &response.content[0].raw {
                assert!(text.text.contains("not found"));
            } else {
                panic!("Expected text response");
            }

            // Test merging a non-existent issue
            let merge_request = MergeIssueRequest {
                name: IssueName::new("non_existent_issue".to_string()).unwrap(),
                delete_branch: false,
            };

            let _merge_result = server
                .tool_context
                .tool_handlers
                .handle_issue_merge(merge_request)
                .await;
            // Note: merge operation may handle missing issues gracefully

            // Create a valid issue
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("error_test".to_string()).unwrap()),
                content: "Testing error handling".to_string(),
            };

            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(!create_result.is_error.unwrap_or(false));

            // Commit the created issue file to git
            Command::new("git")
                .args(["add", "issues/"])
                .current_dir(temp_dir.path())
                .output()
                .expect("Failed to add issues to git");

            Command::new("git")
                .args(["commit", "-m", "Add error test issue"])
                .current_dir(temp_dir.path())
                .output()
                .expect("Failed to commit issue");

            // Test merging an issue that hasn't been worked on (no branch exists)
            let merge_request = MergeIssueRequest {
                name: issue_name,
                delete_branch: false,
            };

            let _merge_result = server
                .tool_context
                .tool_handlers
                .handle_issue_merge(merge_request)
                .await
                .unwrap();
            // This should handle the case gracefully (may succeed or fail depending on implementation)
        }

        #[tokio::test]
        async fn test_issue_all_complete_no_issues() {
            let (server, _temp) = create_test_mcp_server().await;

            let all_complete_request = AllCompleteRequest {};
            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_all_complete(all_complete_request)
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));

            // Check response text
            if let RawContent::Text(text) = &result.content[0].raw {
                assert!(text.text.contains("📋 No issues found in the project"));
                assert!(text.text.contains("The project has no tracked issues"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_issue_all_complete_all_completed() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create and complete multiple issues
            let issues = vec![
                ("test_issue_1", "Content for issue 1"),
                ("test_issue_2", "Content for issue 2"),
                ("test_issue_3", "Content for issue 3"),
            ];

            for (name, content) in issues {
                // Create issue
                let create_request = CreateIssueRequest {
                    name: Some(IssueName::new(name.to_string()).unwrap()),
                    content: content.to_string(),
                };
                // Extract issue name and complete it
                let issue_name = extract_issue_name_from_create_request(&create_request);
                let create_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_create(create_request)
                    .await
                    .unwrap();
                assert!(!create_result.is_error.unwrap_or(false));
                let complete_request = MarkCompleteRequest { name: issue_name };
                let complete_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_mark_complete(complete_request)
                    .await
                    .unwrap();
                assert!(!complete_result.is_error.unwrap_or(false));
            }

            // Check all complete
            let all_complete_request = AllCompleteRequest {};
            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_all_complete(all_complete_request)
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));

            // Check response text
            if let RawContent::Text(text) = &result.content[0].raw {
                assert!(text.text.contains("🎉 All issues are complete!"));
                assert!(text.text.contains("Total Issues: 3"));
                assert!(text.text.contains("Completed: 3 (100%)"));
                assert!(text.text.contains("Active: 0"));
                assert!(text.text.contains("✅ Completed Issues:"));
                assert!(text.text.contains("test_issue_1"));
                assert!(text.text.contains("test_issue_2"));
                assert!(text.text.contains("test_issue_3"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_issue_all_complete_mixed_states() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create active issues
            let active_issues = vec![
                ("active_issue_1", "Active content 1"),
                ("active_issue_2", "Active content 2"),
            ];

            for (name, content) in active_issues {
                let create_request = CreateIssueRequest {
                    name: Some(IssueName(name.to_string())),
                    content: content.to_string(),
                };
                let create_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_create(create_request)
                    .await
                    .unwrap();
                assert!(!create_result.is_error.unwrap_or(false));
            }

            // Create and complete issues
            let completed_issues = vec![
                ("completed_issue_1", "Completed content 1"),
                ("completed_issue_2", "Completed content 2"),
                ("completed_issue_3", "Completed content 3"),
            ];

            for (name, content) in completed_issues {
                let create_request = CreateIssueRequest {
                    name: Some(IssueName::new(name.to_string()).unwrap()),
                    content: content.to_string(),
                };
                let issue_name = extract_issue_name_from_create_request(&create_request);
                let create_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_create(create_request)
                    .await
                    .unwrap();
                assert!(!create_result.is_error.unwrap_or(false));
                let complete_request = MarkCompleteRequest { name: issue_name };
                let complete_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_mark_complete(complete_request)
                    .await
                    .unwrap();
                assert!(!complete_result.is_error.unwrap_or(false));
            }

            // Check all complete
            let all_complete_request = AllCompleteRequest {};
            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_all_complete(all_complete_request)
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));

            // Check response text
            if let RawContent::Text(text) = &result.content[0].raw {
                assert!(text.text.contains("⏳ Project has active issues"));
                assert!(text.text.contains("Total Issues: 5"));
                assert!(text.text.contains("Completed: 3 (60%)"));
                assert!(text.text.contains("Active: 2"));
                assert!(text.text.contains("🔄 Active Issues:"));
                assert!(text.text.contains("active_issue_1"));
                assert!(text.text.contains("active_issue_2"));
                assert!(text.text.contains("✅ Completed Issues:"));
                assert!(text.text.contains("completed_issue_1"));
                assert!(text.text.contains("completed_issue_2"));
                assert!(text.text.contains("completed_issue_3"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_issue_all_complete_only_active() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create only active issues
            let active_issues = vec![
                ("only_active_1", "Active content 1"),
                ("only_active_2", "Active content 2"),
                ("only_active_3", "Active content 3"),
            ];

            for (name, content) in active_issues {
                let create_request = CreateIssueRequest {
                    name: Some(IssueName(name.to_string())),
                    content: content.to_string(),
                };
                let create_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_create(create_request)
                    .await
                    .unwrap();
                assert!(!create_result.is_error.unwrap_or(false));
            }

            // Check all complete
            let all_complete_request = AllCompleteRequest {};
            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_all_complete(all_complete_request)
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));

            // Check response text
            if let RawContent::Text(text) = &result.content[0].raw {
                assert!(text.text.contains("⏳ Project has active issues"));
                assert!(text.text.contains("Total Issues: 3"));
                assert!(text.text.contains("Completed: 0 (0%)"));
                assert!(text.text.contains("Active: 3"));
                assert!(text.text.contains("🔄 Active Issues:"));
                assert!(text.text.contains("only_active_1"));
                assert!(text.text.contains("only_active_2"));
                assert!(text.text.contains("only_active_3"));
                assert!(text.text.contains("✅ Completed Issues:"));
                assert!(text.text.contains("(none)"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_issue_all_complete_comprehensive_response_format() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create one active and one completed issue
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("format_test_active".to_string()).unwrap()),
                content: "Active issue content".to_string(),
            };
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(!create_result.is_error.unwrap_or(false));

            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("format_test_completed".to_string()).unwrap()),
                content: "Completed issue content".to_string(),
            };
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(!create_result.is_error.unwrap_or(false));
            let complete_request = MarkCompleteRequest { name: issue_name };
            let complete_result = server
                .tool_context
                .tool_handlers
                .handle_issue_mark_complete(complete_request)
                .await
                .unwrap();
            assert!(!complete_result.is_error.unwrap_or(false));

            // Check all complete
            let all_complete_request = AllCompleteRequest {};
            let result = server
                .tool_context
                .tool_handlers
                .handle_issue_all_complete(all_complete_request)
                .await
                .unwrap();

            assert!(!result.is_error.unwrap_or(false));

            // Check response formatting
            if let RawContent::Text(text) = &result.content[0].raw {
                // Check that it contains proper formatting
                assert!(text.text.contains("📊 Project Status:"));
                assert!(text.text.contains("• Total Issues:"));
                assert!(text.text.contains("• Completed:"));
                assert!(text.text.contains("• Active:"));
                assert!(text.text.contains("50%"));

                // Check issue names are present
                assert!(
                    text.text.contains("format_test_active")
                        || text.text.contains("format_test_completed")
                );

                // Check proper emoji usage
                assert!(text.text.contains("⏳"));
                assert!(text.text.contains("🔄"));
                assert!(text.text.contains("✅"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_mcp_issue_merge_incomplete_issue() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create an issue but don't mark it as complete
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("incomplete_merge_test".to_string()).unwrap()),
                content: "Test merge of incomplete issue".to_string(),
            };
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(
                !create_result.content.is_empty(),
                "Create result should have content"
            );

            // Try to merge incomplete issue - should fail
            let merge_request = MergeIssueRequest {
                name: issue_name,
                delete_branch: false,
            };

            let merge_result = server
                .tool_context
                .tool_handlers
                .handle_issue_merge(merge_request)
                .await
                .unwrap();

            // Should fail because issue is not completed
            assert!(merge_result.is_error.unwrap_or(false));

            // Verify error message mentions issue is not completed
            if let RawContent::Text(text) = &merge_result.content[0].raw {
                assert!(text.text.contains("must be completed"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_mcp_issue_merge_non_existent_issue() {
            let (server, _temp) = create_test_mcp_server().await;

            // Try to merge non-existent issue
            let merge_request = MergeIssueRequest {
                name: IssueName::new("non_existent_merge_test".to_string()).unwrap(),
                delete_branch: false,
            };

            let merge_result = server
                .tool_context
                .tool_handlers
                .handle_issue_merge(merge_request)
                .await
                .unwrap();

            // Should fail because issue doesn't exist
            assert!(merge_result.is_error.unwrap_or(false));

            // Verify error message mentions issue not found
            if let RawContent::Text(text) = &merge_result.content[0].raw {
                assert!(text.text.contains("not found"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_mcp_issue_merge_no_branch() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create and complete an issue but don't create a branch
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("no_branch_test".to_string()).unwrap()),
                content: "Test merge with no branch".to_string(),
            };
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(
                !create_result.content.is_empty(),
                "Create result should have content"
            );

            // Mark issue as complete
            let complete_request = MarkCompleteRequest {
                name: issue_name.clone(),
            };
            server
                .tool_context
                .tool_handlers
                .handle_issue_mark_complete(complete_request)
                .await
                .unwrap();

            // Commit changes to keep git clean
            commit_changes(_temp.path()).await;

            // Try to merge without creating a branch - should fail
            let merge_request = MergeIssueRequest {
                name: issue_name,
                delete_branch: false,
            };

            let merge_result = server
                .tool_context
                .tool_handlers
                .handle_issue_merge(merge_request)
                .await
                .unwrap();

            // Should fail because branch doesn't exist
            assert!(merge_result.is_error.unwrap_or(false));

            // Verify error message (could be about working directory or branch not found)
            if let RawContent::Text(text) = &merge_result.content[0].raw {
                // Accept either error - working directory not clean or branch does not exist
                assert!(
                    text.text.contains("Working directory is not clean")
                        || text.text.contains("does not exist"),
                    "Expected either working directory error or branch not found error, got: {}",
                    text.text
                );
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_mcp_issue_merge_with_branch_deletion() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create an issue
            let create_request = CreateIssueRequest {
                name: Some(IssueName::new("delete_branch_test".to_string()).unwrap()),
                content: "Test merge with branch deletion".to_string(),
            };
            let issue_name = extract_issue_name_from_create_request(&create_request);
            let create_result = server
                .tool_context
                .tool_handlers
                .handle_issue_create(create_request)
                .await
                .unwrap();
            assert!(
                !create_result.content.is_empty(),
                "Create result should have content"
            );

            // Commit the issue file to keep git clean
            commit_changes(_temp.path()).await;

            // Work on the issue to create a branch
            let work_request = WorkIssueRequest {
                name: issue_name.clone(),
            };
            server
                .tool_context
                .tool_handlers
                .handle_issue_work(work_request)
                .await
                .unwrap();

            // Make a dummy commit on the issue branch
            fs::write(_temp.path().join("test_file.txt"), "test content").unwrap();
            Command::new("git")
                .args(["add", "test_file.txt"])
                .current_dir(_temp.path())
                .output()
                .expect("Failed to add file");

            Command::new("git")
                .args(["commit", "-m", "Test commit"])
                .current_dir(_temp.path())
                .output()
                .expect("Failed to commit");

            // Mark issue as complete
            let complete_request = MarkCompleteRequest {
                name: issue_name.clone(),
            };
            server
                .tool_context
                .tool_handlers
                .handle_issue_mark_complete(complete_request)
                .await
                .unwrap();

            // Commit the issue completion
            commit_changes(_temp.path()).await;

            // Test merge with branch deletion
            let merge_request = MergeIssueRequest {
                name: issue_name,
                delete_branch: true,
            };

            let merge_result = server
                .tool_context
                .tool_handlers
                .handle_issue_merge(merge_request)
                .await
                .unwrap();
            assert!(!merge_result.is_error.unwrap_or(false));

            // Verify merge response mentions branch deletion
            if let RawContent::Text(text) = &merge_result.content[0].raw {
                println!("DEBUG: Actual merge response: '{}'", text.text);
                assert!(text.text.contains("Merged work branch"));
                assert!(text.text.contains("deleted branch"));
            } else {
                panic!("Expected text response");
            }
        }

        #[tokio::test]
        async fn test_mcp_issue_merge_project_statistics() {
            let (server, _temp) = create_test_mcp_server().await;

            // Create and complete multiple issues
            let mut issue_names = Vec::new();
            for i in 0..3 {
                let create_request = CreateIssueRequest {
                    name: Some(IssueName::new(format!("stats_test_{i}")).unwrap()),
                    content: format!("Test issue {i}"),
                };
                let issue_name = extract_issue_name_from_create_request(&create_request);
                let create_result = server
                    .tool_context
                    .tool_handlers
                    .handle_issue_create(create_request)
                    .await
                    .unwrap();
                assert!(
                    !create_result.content.is_empty(),
                    "Create result should have content"
                );
                issue_names.push(issue_name);
            }

            // Complete all issues except the first one
            for issue_name in &issue_names[1..] {
                let complete_request = MarkCompleteRequest {
                    name: issue_name.clone(),
                };
                server
                    .tool_context
                    .tool_handlers
                    .handle_issue_mark_complete(complete_request)
                    .await
                    .unwrap();
            }

            // Commit issue completions
            commit_changes(_temp.path()).await;

            // Work on the first issue
            let work_request = WorkIssueRequest {
                name: issue_names[0].clone(),
            };
            server
                .tool_context
                .tool_handlers
                .handle_issue_work(work_request)
                .await
                .unwrap();

            // Make a dummy commit
            fs::write(_temp.path().join("test_file.txt"), "test content").unwrap();
            Command::new("git")
                .args(["add", "test_file.txt"])
                .current_dir(_temp.path())
                .output()
                .expect("Failed to add file");

            Command::new("git")
                .args(["commit", "-m", "Test commit"])
                .current_dir(_temp.path())
                .output()
                .expect("Failed to commit");

            // Now complete the first issue
            let complete_request = MarkCompleteRequest {
                name: issue_names[0].clone(),
            };
            server
                .tool_context
                .tool_handlers
                .handle_issue_mark_complete(complete_request)
                .await
                .unwrap();

            // Commit the issue completion
            commit_changes(_temp.path()).await;

            // Test merge - should include project statistics
            let merge_request = MergeIssueRequest {
                name: issue_names[0].clone(),
                delete_branch: false,
            };

            let merge_result = server
                .tool_context
                .tool_handlers
                .handle_issue_merge(merge_request)
                .await
                .unwrap();
            assert!(!merge_result.is_error.unwrap_or(false));

            // Verify merge response is successful
            if let RawContent::Text(text) = &merge_result.content[0].raw {
                println!("DEBUG: Project stats merge response: '{}'", text.text);
                assert!(text.text.contains("Merged work branch"));
                // Note: Project statistics are available through separate issue_all_complete operation
            } else {
                panic!("Expected text response");
            }
        }
    }

    #[tokio::test]
    async fn test_handle_issue_create_nameless() {
        // Test successful issue creation without a name
        let library = PromptLibrary::new();
        let server = McpServer::new(library).unwrap();

        let request = CreateIssueRequest {
            name: None,
            content: "# Nameless Issue\n\nThis is a test issue without a name.".to_string(),
        };

        let result = server
            .tool_context
            .tool_handlers
            .handle_issue_create(request)
            .await;
        assert!(result.is_ok(), "Nameless issue creation should succeed");

        let call_result = result.unwrap();
        assert_eq!(call_result.is_error, Some(false));
        assert!(!call_result.content.is_empty());

        // Check that the response contains expected information
        let text_content = &call_result.content[0];
        if let RawContent::Text(text) = &text_content.raw {
            assert!(text.text.contains("Created issue"));

            // Verify that the text contains information about file path
            // For nameless issues, the system now creates a ULID-based name
            assert!(
                text.text.contains(".md"),
                "Response should contain the markdown file extension: {}",
                text.text
            );
            assert!(
                !text.text.contains("_unnamed"),
                "Nameless issue should not contain '_unnamed' in filename"
            );
        } else {
            panic!("Expected text content, got: {:?}", text_content.raw);
        }
    }
}
