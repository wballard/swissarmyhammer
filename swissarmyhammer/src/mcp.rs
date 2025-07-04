//! Model Context Protocol (MCP) server support

use crate::PromptLibrary;
use rmcp::model::*;
use rmcp::{ServerHandler, RoleServer, Error as McpError};
use rmcp::service::{RequestContext, Peer};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

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

/// MCP server for serving prompts
#[derive(Clone)]
pub struct McpServer {
    library: Arc<RwLock<PromptLibrary>>,
    peer: Arc<Mutex<Option<Peer<RoleServer>>>>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(library: PromptLibrary) -> Self {
        Self {
            library: Arc::new(RwLock::new(library)),
            peer: Arc::new(Mutex::new(None)),
        }
    }

    /// Get the underlying library
    pub fn library(&self) -> &Arc<RwLock<PromptLibrary>> {
        &self.library
    }

    /// Initialize the server with prompt directories
    pub async fn initialize(&self) -> anyhow::Result<()> {
        let mut library = self.library.write().await;
        
        // Load builtin prompts
        let builtin_dir = dirs::data_dir()
            .map(|d| d.join("swissarmyhammer").join("prompts"))
            .filter(|p| p.exists());
        
        if let Some(dir) = builtin_dir {
            let count = library.add_directory(&dir)?;
            tracing::info!("Loaded {} builtin prompts from {:?}", count, dir);
        }
        
        // Load user prompts
        let user_dir = dirs::home_dir()
            .map(|d| d.join(".prompts"))
            .filter(|p| p.exists());
        
        if let Some(dir) = user_dir {
            let count = library.add_directory(&dir)?;
            tracing::info!("Loaded {} user prompts from {:?}", count, dir);
        }
        
        // Load local prompts
        let local_dir = std::path::Path::new("prompts");
        if local_dir.exists() {
            let count = library.add_directory(local_dir)?;
            tracing::info!("Loaded {} local prompts from {:?}", count, local_dir);
        }
        
        let total = library.list()?.len();
        tracing::info!("Loaded {} prompts total", total);
        
        Ok(())
    }

    /// List all available prompts  
    pub async fn list_prompts(&self) -> anyhow::Result<Vec<String>> {
        let library = self.library.read().await;
        let prompts = library.list()?;
        Ok(prompts.iter().map(|p| p.name.clone()).collect())
    }

    /// Get a specific prompt by name
    pub async fn get_prompt(&self, name: &str, arguments: Option<&HashMap<String, String>>) -> anyhow::Result<String> {
        let library = self.library.read().await;
        let prompt = library.get(name)?;
        
        // Handle arguments if provided
        let content = if let Some(args) = arguments {
            prompt.render(args)?
        } else {
            prompt.template.clone()
        };

        Ok(content)
    }
}

impl ServerHandler for McpServer {
    async fn initialize(
        &self,
        _request: InitializeRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        Ok(InitializeResult {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                prompts: Some(PromptsCapability {
                    list_changed: Some(true),
                }),
                tools: None,
                resources: None,
                logging: None,
                experimental: None,
            },
            instructions: Some("A flexible prompt management server for AI assistants. Use list_prompts to see available prompts and get_prompt to retrieve and render them.".into()),
            server_info: Implementation {
                name: "SwissArmyHammer".into(),
                version: crate::VERSION.into(),
            },
        })
    }

    async fn list_prompts(
        &self,
        _request: PaginatedRequestParam,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        let library = self.library.read().await;
        match library.list() {
            Ok(prompts) => {
                let prompt_list: Vec<Prompt> = prompts.iter().map(|p| {
                    let arguments = if p.arguments.is_empty() {
                        None
                    } else {
                        Some(p.arguments.iter().map(|arg| {
                            PromptArgument {
                                name: arg.name.clone(),
                                description: arg.description.clone(),
                                required: Some(arg.required),
                            }
                        }).collect())
                    };

                    Prompt {
                        name: p.name.clone(),
                        description: p.description.clone(),
                        arguments,
                    }
                }).collect();

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
                // Handle arguments if provided
                let content = if let Some(args) = &request.arguments {
                    // Convert serde_json::Map to HashMap<String, String>
                    let mut template_args = HashMap::new();
                    for (key, value) in args {
                        let value_str = match value {
                            Value::String(s) => s.clone(),
                            v => v.to_string()
                        };
                        template_args.insert(key.clone(), value_str);
                    }
                    
                    match prompt.render(&template_args) {
                        Ok(rendered) => rendered,
                        Err(e) => return Err(McpError::internal_error(format!("Template rendering error: {}", e), None)),
                    }
                } else {
                    prompt.template.clone()
                };

                Ok(GetPromptResult {
                    description: prompt.description,
                    messages: vec![PromptMessage {
                        role: PromptMessageRole::User,
                        content: PromptMessageContent::Text {
                            text: content,
                        },
                    }],
                })
            }
            Err(e) => Err(McpError::internal_error(format!("Prompt not found: {}", e), None)),
        }
    }

    fn get_peer(&self) -> Option<Peer<RoleServer>> {
        self.peer.lock().unwrap().clone()
    }

    fn set_peer(&mut self, peer: Peer<RoleServer>) {
        *self.peer.lock().unwrap() = Some(peer);
    }

    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::default(),
            capabilities: ServerCapabilities {
                prompts: Some(PromptsCapability {
                    list_changed: Some(true),
                }),
                tools: None,
                resources: None,
                logging: None,
                experimental: None,
            },
            server_info: Implementation {
                name: "SwissArmyHammer".into(),
                version: crate::VERSION.into(),
            },
            instructions: Some("A flexible prompt management server for AI assistants. Use list_prompts to see available prompts and get_prompt to retrieve and render them.".into()),
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
        let server = McpServer::new(library);

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

        let server = McpServer::new(library);
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

        let server = McpServer::new(library);
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
        let server = McpServer::new(library);

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
        assert!(info.instructions.unwrap().contains("prompt management"));
    }
}
