use anyhow::Result;
use rmcp::{
    model::{
        GetPromptRequestParam, GetPromptResult, Implementation, ListPromptsResult,
        PaginatedRequestParam, Prompt, PromptArgument, PromptMessage, PromptMessageContent,
        PromptMessageRole, ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, Error, RoleServer, ServerHandler, ServiceExt,
};
use serde_json::{json, Value};
use tokio::sync::oneshot;
use tracing::info;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use swissarmyhammer::{PromptLibrary, Prompt as SwissPrompt, PromptLoader};

#[derive(Clone)]
pub struct MCPServer {
    name: String,
    version: String,
    library: Arc<Mutex<PromptLibrary>>,
}

impl MCPServer {
    pub fn new() -> Self {
        Self {
            name: "swissarmyhammer".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            library: Arc::new(Mutex::new(PromptLibrary::new())),
        }
    }
    
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing prompt storage...");
        
        let mut library = self.library.lock().unwrap();
        
        // Load builtin prompts
        let builtin_dir = dirs::data_dir()
            .map(|d| d.join("swissarmyhammer").join("prompts"))
            .filter(|p| p.exists());
        
        if let Some(dir) = builtin_dir {
            let count = library.add_directory(&dir)?;
            info!("Loaded {} builtin prompts from {:?}", count, dir);
        }
        
        // Load user prompts
        let user_dir = dirs::home_dir()
            .map(|d| d.join(".prompts"))
            .filter(|p| p.exists());
        
        if let Some(dir) = user_dir {
            let count = library.add_directory(&dir)?;
            info!("Loaded {} user prompts from {:?}", count, dir);
        }
        
        // Load local prompts
        let local_dir = std::path::Path::new("prompts");
        if local_dir.exists() {
            let count = library.add_directory(local_dir)?;
            info!("Loaded {} local prompts from {:?}", count, local_dir);
        }
        
        let total = library.list()?.len();
        info!("Loaded {} prompts total", total);
        
        Ok(())
    }

    /// Convert internal prompts to MCP-compatible format
    pub fn convert_prompts_to_mcp_format(&self) -> Value {
        let library = self.library.lock().unwrap();
        let prompts = library.list().unwrap_or_default();
        
        let mcp_prompts: Vec<Value> = prompts
            .iter()
            .map(|prompt| {
                let mcp_arguments: Vec<Value> = prompt.arguments
                    .iter()
                    .map(|arg| json!({
                        "name": arg.name,
                        "description": arg.description,
                        "required": arg.required
                    }))
                    .collect();

                json!({
                    "name": prompt.name,
                    "description": prompt.description.as_deref().unwrap_or(""),
                    "arguments": mcp_arguments
                })
            })
            .collect();

        json!({
            "prompts": mcp_prompts
        })
    }

    /// Get a specific prompt by name
    pub fn get_prompt_by_name(&self, name: &str, arguments: Option<&Value>) -> Result<Value> {
        let library = self.library.lock().unwrap();
        let prompt = library.get(name)?;
        
        // Convert JSON arguments to HashMap<String, String> for the template engine
        let args_map = if let Some(args) = arguments {
            if let Some(obj) = args.as_object() {
                obj.iter()
                    .map(|(k, v)| {
                        let value_str = match v {
                            Value::String(s) => s.clone(),
                            v => v.to_string(),
                        };
                        (k.clone(), value_str)
                    })
                    .collect()
            } else {
                HashMap::new()
            }
        } else {
            HashMap::new()
        };
        
        // Use the template engine to process the prompt
        let processed_content = prompt.render(&args_map)?;

        Ok(json!({
            "description": prompt.description.as_deref().unwrap_or(""),
            "messages": [{
                "role": "user",
                "content": {
                    "type": "text",
                    "text": processed_content
                }
            }]
        }))
    }
}

impl Default for MCPServer {
    fn default() -> Self {
        Self::new()
    }
}

impl MCPServer {
    pub async fn run(self, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        info!("Starting MCP server via stdio");

        // Initialize prompts
        self.initialize().await?;

        // Create server instance
        let role_server = ServerHandler::<Self>::stdio();

        // Run the server
        role_server.with_cancel(shutdown_rx).run().await;

        info!("MCP server stopped");
        Ok(())
    }
}

#[tool]
impl ServerHandler for MCPServer {
    type Services = (MCPServer,);

    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            prompts: Some(Default::default()),
            ..Default::default()
        }
    }

    fn info(&self) -> Implementation {
        Implementation {
            name: self.name.clone(),
            version: self.version.clone(),
        }
    }

    fn services(&self) -> Self::Services {
        (self.clone(),)
    }
}

impl MCPServer {
    /// Handler for listing prompts
    #[tool]
    pub async fn list_prompts(
        &self,
        _ctx: RequestContext,
        _req: PaginatedRequestParam,
    ) -> Result<ListPromptsResult, Error> {
        let library = self.library.lock().unwrap();
        let prompts = library.list()
            .map_err(|e| Error::server_error(e.to_string()))?;
        
        let mcp_prompts: Vec<Prompt> = prompts
            .into_iter()
            .map(|p| {
                let arguments = if p.arguments.is_empty() {
                    None
                } else {
                    Some(p.arguments.into_iter().map(|arg| PromptArgument {
                        name: arg.name,
                        description: arg.description,
                        required: Some(arg.required),
                    }).collect())
                };
                
                Prompt {
                    name: p.name,
                    description: p.description,
                    arguments,
                }
            })
            .collect();

        Ok(ListPromptsResult {
            prompts: mcp_prompts,
            next_cursor: None,
        })
    }

    /// Handler for getting a prompt by name
    #[tool]
    pub async fn get_prompt(
        &self,
        _ctx: RequestContext,
        req: GetPromptRequestParam,
    ) -> Result<GetPromptResult, Error> {
        let library = self.library.lock().unwrap();
        let prompt = library.get(&req.name)
            .map_err(|e| Error::server_error(e.to_string()))?;
        
        // Convert arguments
        let arguments = if prompt.arguments.is_empty() {
            None
        } else {
            Some(prompt.arguments.iter().map(|arg| PromptArgument {
                name: arg.name.clone(),
                description: arg.description.clone(),
                required: Some(arg.required),
            }).collect())
        };
        
        // Render template if arguments provided
        let content = if let Some(args) = req.arguments {
            // Convert JSON to string map
            let mut string_args = HashMap::new();
            if let Some(obj) = args.as_object() {
                for (k, v) in obj {
                    let value_str = match v {
                        Value::String(s) => s.clone(),
                        v => v.to_string(),
                    };
                    string_args.insert(k.clone(), value_str);
                }
            }
            
            prompt.render(&string_args)
                .map_err(|e| Error::server_error(e.to_string()))?
        } else {
            prompt.template.clone()
        };
        
        let message = PromptMessage {
            role: PromptMessageRole::User,
            content: PromptMessageContent::Text { text: content },
        };

        Ok(GetPromptResult {
            prompt: Prompt {
                name: prompt.name,
                description: prompt.description,
                arguments,
            },
            messages: vec![message],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_creation() {
        let server = MCPServer::new();
        assert_eq!(server.name, "swissarmyhammer");
    }

    #[tokio::test]
    async fn test_server_info() {
        let server = MCPServer::new();
        let info = server.info();
        assert_eq!(info.name, "swissarmyhammer");
    }

    #[tokio::test]
    async fn test_server_capabilities_include_prompts() {
        let server = MCPServer::new();
        let capabilities = server.capabilities();
        assert!(capabilities.prompts.is_some());
    }
}