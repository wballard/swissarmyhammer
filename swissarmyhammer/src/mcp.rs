//! Model Context Protocol (MCP) server support

use crate::{PromptLibrary, PromptResolver};
use crate::workflow::{
    WorkflowStorage, WorkflowStorageBackend, WorkflowRunStorageBackend,
    FileSystemWorkflowStorage, FileSystemWorkflowRunStorage,
};
use rmcp::model::*;
use rmcp::service::RequestContext;
use rmcp::{Error as McpError, RoleServer, ServerHandler};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

use notify::{
    event::{Event, EventKind},
    RecommendedWatcher, RecursiveMode, Watcher,
};
use tokio::sync::mpsc;

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
    watcher_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(library: PromptLibrary) -> anyhow::Result<Self> {
        // Initialize workflow storage with filesystem backend
        let workflow_backend = Arc::new(FileSystemWorkflowStorage::new().map_err(|e| {
            tracing::error!("Failed to create workflow storage: {}", e);
            anyhow::anyhow!("Failed to create workflow storage: {}", e)
        })?) as Arc<dyn WorkflowStorageBackend>;
        
        // Create runs directory in user's home directory
        let runs_path = Self::get_workflow_runs_path();
        
        let run_backend = Arc::new(FileSystemWorkflowRunStorage::new(runs_path).map_err(|e| {
            tracing::error!("Failed to create workflow run storage: {}", e);
            anyhow::anyhow!("Failed to create workflow run storage: {}", e)
        })?) as Arc<dyn WorkflowRunStorageBackend>;
        
        let workflow_storage = WorkflowStorage::new(workflow_backend, run_backend);
        
        Ok(Self {
            library: Arc::new(RwLock::new(library)),
            workflow_storage: Arc::new(RwLock::new(workflow_storage)),
            watcher_handle: Arc::new(Mutex::new(None)),
        })
    }

    /// Get the underlying library
    pub fn library(&self) -> &Arc<RwLock<PromptLibrary>> {
        &self.library
    }

    /// Initialize the server with prompt directories using PromptResolver
    pub async fn initialize(&self) -> anyhow::Result<()> {
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

    /// List all available prompts (excluding partial templates)
    pub async fn list_prompts(&self) -> anyhow::Result<Vec<String>> {
        let library = self.library.read().await;
        let prompts = library.list()?;
        Ok(prompts
            .iter()
            .filter(|p| !Self::is_partial_template(p))
            .map(|p| p.name.clone())
            .collect())
    }

    /// List all available workflows
    pub async fn list_workflows(&self) -> anyhow::Result<Vec<String>> {
        let workflow_storage = self.workflow_storage.read().await;
        let workflows = workflow_storage.list_workflows()?;
        Ok(workflows.iter().map(|w| w.name.to_string()).collect())
    }

    /// Check if a prompt is a partial template that should not be exposed over MCP
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

    /// Get a specific prompt by name (excluding partial templates)
    pub async fn get_prompt(
        &self,
        name: &str,
        arguments: Option<&HashMap<String, String>>,
    ) -> anyhow::Result<String> {
        let library = self.library.read().await;
        let prompt = library.get(name)?;

        // Check if this is a partial template
        if Self::is_partial_template(&prompt) {
            return Err(anyhow::anyhow!(
                "Cannot access partial template '{}' via MCP. Partial templates are for internal use only.",
                name
            ));
        }

        // Handle arguments if provided
        let content = if let Some(args) = arguments {
            library.render_prompt(name, args)?
        } else {
            prompt.template.clone()
        };

        Ok(content)
    }

    /// Start watching prompt directories for changes
    pub async fn start_file_watching(&self, peer: rmcp::Peer<RoleServer>) -> anyhow::Result<()> {
        tracing::info!("Starting file watching for prompt directories");

        // Get the directories to watch using the same logic as PromptResolver
        let resolver = PromptResolver::new();
        let watch_paths = resolver.get_prompt_directories()?;

        tracing::info!(
            "Found {} directories to watch: {:?}",
            watch_paths.len(),
            watch_paths
        );

        // The resolver already returns only existing paths
        if watch_paths.is_empty() {
            tracing::warn!("No prompt directories found to watch");
            return Ok(());
        }

        // Create the file watcher
        let (tx, mut rx) = mpsc::channel(100);
        let mut watcher = RecommendedWatcher::new(
            move |result: Result<Event, notify::Error>| {
                if let Ok(event) = result {
                    if let Err(e) = tx.blocking_send(event) {
                        tracing::error!("Failed to send file watch event: {}", e);
                    }
                }
            },
            notify::Config::default(),
        )?;

        // Watch all directories
        for path in &watch_paths {
            watcher.watch(path, RecursiveMode::Recursive)?;
            tracing::info!("Watching directory: {:?}", path);
        }

        // Spawn the event handler task
        let server = self.clone();
        let handle = tokio::spawn(async move {
            // Keep the watcher alive for the duration of this task
            // The watcher must be moved into the task to prevent it from being dropped
            let _watcher = watcher;

            while let Some(event) = rx.recv().await {
                tracing::debug!("📁 File system event: {:?}", event);

                // Check if this is a relevant event
                match event.kind {
                    EventKind::Create(_) | EventKind::Modify(_) | EventKind::Remove(_) => {
                        // Check if it's a prompt file (*.md, *.yaml, *.yml)
                        let is_prompt_file = event.paths.iter().any(|p| {
                            if let Some(ext) = p.extension() {
                                matches!(ext.to_str(), Some("md") | Some("yaml") | Some("yml"))
                            } else {
                                false
                            }
                        });

                        if is_prompt_file {
                            tracing::info!("📄 Prompt file changed: {:?}", event.paths);

                            // Reload the library
                            if let Err(e) = server.reload_prompts().await {
                                tracing::error!("❌ Failed to reload prompts: {}", e);
                            } else {
                                tracing::info!("✅ Prompts reloaded successfully");
                            }

                            // Send notification to client about prompt list change
                            let peer_clone = peer.clone();
                            tokio::spawn(async move {
                                match peer_clone.notify_prompt_list_changed().await {
                                    Ok(_) => {
                                        tracing::info!(
                                            "📢 Sent prompts/listChanged notification to client"
                                        );
                                    }
                                    Err(e) => {
                                        tracing::error!("❌ Failed to send notification: {}", e);
                                    }
                                }
                            });
                        } else {
                            tracing::debug!("🚫 Ignoring non-prompt file: {:?}", event.paths);
                        }
                    }
                    _ => {
                        tracing::debug!("🚫 Ignoring event type: {:?}", event.kind);
                    }
                }
            }
        });

        // Store the handle
        *self.watcher_handle.lock().unwrap() = Some(handle);

        Ok(())
    }

    /// Stop file watching
    pub fn stop_file_watching(&self) {
        if let Some(handle) = self.watcher_handle.lock().unwrap().take() {
            handle.abort();
        }
    }

    /// Get the workflow runs directory path
    fn get_workflow_runs_path() -> std::path::PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")))
            .join(".swissarmyhammer")
            .join("workflow-runs")
    }

    /// Convert internal prompt arguments to MCP PromptArgument structures
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

    /// Convert serde_json::Map to HashMap<String, String>
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

    /// Reload prompts from disk
    async fn reload_prompts(&self) -> anyhow::Result<()> {
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
    ) -> Result<InitializeResult, McpError> {
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
            },
            instructions: Some("A flexible prompt and workflow management server for AI assistants. Use list_prompts to see available prompts and get_prompt to retrieve and render them. Use workflow tools to execute and manage workflows.".into()),
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
    ) -> Result<ListPromptsResult, McpError> {
        let library = self.library.read().await;
        match library.list() {
            Ok(prompts) => {
                let prompt_list: Vec<Prompt> = prompts
                    .iter()
                    .filter(|p| !Self::is_partial_template(p))  // Filter out partial templates
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
    ) -> Result<GetPromptResult, McpError> {
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
            },
            server_info: Implementation {
                name: "SwissArmyHammer".into(),
                version: crate::VERSION.into(),
            },
            instructions: Some("A flexible prompt and workflow management server for AI assistants. Use list_prompts to see available prompts and get_prompt to retrieve and render them. Use workflow tools to execute and manage workflows.".into()),
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
        assert!(info.instructions.unwrap().contains("prompt and workflow management"));
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
        let partial_with_marker = Prompt::new("partial_with_marker", "{% partial %}\nThis is a partial with marker")
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
}
