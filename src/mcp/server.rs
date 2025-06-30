use anyhow::Result;
use rmcp::{
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, ServerHandler, ServiceExt,
};
use serde_json::{json, Value};
use tokio::sync::oneshot;
use tracing::info;
use std::collections::HashMap;
use crate::prompts::{PromptLoader, PromptWatcher, PromptStorage};

#[derive(Clone)]
pub struct MCPServer {
    name: String,
    version: String,
    storage: PromptStorage,
}

impl MCPServer {
    pub fn new() -> Self {
        Self {
            name: "swissarmyhammer".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            storage: PromptStorage::new(),
        }
    }
    
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing prompt storage...");
        let mut loader = PromptLoader::new();
        loader.storage = self.storage.clone();
        
        // Load all prompts at startup
        loader.load_all()?;
        info!("Loaded {} prompts", self.storage.len());
        
        Ok(())
    }

    /// Convert internal prompts to MCP-compatible format
    pub fn convert_prompts_to_mcp_format(&self) -> Value {
        let mut mcp_prompts = Vec::new();

        for (name, prompt) in self.storage.iter() {
            let mcp_arguments: Vec<Value> = prompt.arguments
                .iter()
                .map(|arg| json!({
                    "name": arg.name,
                    "description": arg.description,
                    "required": arg.required
                }))
                .collect();

            let mcp_prompt = json!({
                "name": name,
                "description": prompt.description.as_deref().unwrap_or(""),
                "arguments": mcp_arguments
            });

            mcp_prompts.push(mcp_prompt);
        }

        json!({
            "prompts": mcp_prompts
        })
    }

    /// Get a specific prompt by name
    pub fn get_prompt_by_name(&self, name: &str, arguments: Option<&Value>) -> Result<Value> {
        match self.storage.get(name) {
            Some(prompt) => {
                // Convert JSON arguments to HashMap for the template engine
                let args_map = if let Some(args) = arguments {
                    if let Some(obj) = args.as_object() {
                        obj.iter()
                            .map(|(k, v)| (k.clone(), v.clone()))
                            .collect()
                    } else {
                        HashMap::new()
                    }
                } else {
                    HashMap::new()
                };
                
                // Use the template engine to process the prompt
                let processed_content = prompt.process_template(&args_map)?;

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
            None => Err(anyhow::anyhow!("Prompt '{}' not found", name)),
        }
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

        // Set up file watcher
        let mut loader = PromptLoader::new();
        loader.storage = self.storage.clone();
        
        let watcher_result = PromptWatcher::new(self.storage.clone());
        let watcher_task = match watcher_result {
            Ok(watcher) => {
                info!("File watcher initialized successfully");
                Some(tokio::spawn(async move {
                    if let Err(e) = watcher.run(loader).await {
                        tracing::error!("File watcher error: {}", e);
                    }
                }))
            }
            Err(e) => {
                tracing::warn!("Failed to initialize file watcher (continuing without file watching): {}", e);
                None
            }
        };

        let transport = (tokio::io::stdin(), tokio::io::stdout());

        tokio::select! {
            result = self.serve(transport) => {
                match result {
                    Ok(server) => {
                        // Wait for the server to complete
                        let quit_reason = server.waiting().await?;
                        info!("MCP server shut down: {:?}", quit_reason);
                    }
                    Err(e) => return Err(e.into()),
                }
            }
            _ = shutdown_rx => {
                info!("MCP server shutting down due to signal");
            }
        }

        // Clean up the watcher task if it was created
        if let Some(task) = watcher_task {
            task.abort();
        }

        Ok(())
    }
}

// Create toolbox for storing tool definitions
#[tool(tool_box)]
impl MCPServer {
    // We'll add tools in future steps
}

impl ServerHandler for MCPServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            server_info: Implementation {
                name: self.name.clone(),
                version: self.version.clone(),
            },
            instructions: Some(
                "SwissArmyHammer MCP Server - Manage prompts as markdown files".into(),
            ),
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .build(),
        }
    }

    // Note: These prompt methods are not implemented yet
    // The rmcp framework will provide default implementations that return "not implemented" errors
    // We'll implement these properly once we understand the correct error type
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mcp_server_creation() {
        let server = MCPServer::new();
        assert_eq!(server.name, "swissarmyhammer");
        assert_eq!(server.version, env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn test_server_info() {
        let server = MCPServer::new();
        let info = server.get_info();

        assert_eq!(info.server_info.name, "swissarmyhammer");
        assert_eq!(info.server_info.version, env!("CARGO_PKG_VERSION"));
        assert!(info.instructions.is_some());
    }

    #[test]
    fn test_server_capabilities_include_prompts() {
        let server = MCPServer::new();
        let info = server.get_info();
        
        // Test that prompts are enabled in capabilities
        assert!(info.capabilities.prompts.is_some());
    }

    #[tokio::test]
    async fn test_prompt_storage_after_initialization() {
        let server = MCPServer::new();
        server.initialize().await.unwrap();
        
        // Test that prompts are loaded into storage
        assert!(!server.storage.is_empty()); // Should have at least builtin prompts
        assert!(server.storage.contains_key("example")); // Should have example prompt
    }

    #[tokio::test]
    async fn test_convert_prompts_to_mcp_format() {
        let server = MCPServer::new();
        server.initialize().await.unwrap();
        
        let mcp_format = server.convert_prompts_to_mcp_format();
        
        // Verify the MCP format structure
        assert!(mcp_format["prompts"].is_array());
        let prompts = mcp_format["prompts"].as_array().unwrap();
        assert!(!prompts.is_empty());
        
        // Check that example prompt is present and has correct structure
        let example_prompt = prompts.iter()
            .find(|p| p["name"] == "example")
            .expect("Example prompt should be present");
        
        assert!(example_prompt["description"].is_string());
        assert!(example_prompt["arguments"].is_array());
    }

    #[tokio::test]
    async fn test_get_prompt_by_name() {
        let server = MCPServer::new();
        server.initialize().await.unwrap();
        
        // Test getting an existing prompt
        let result = server.get_prompt_by_name("example", None);
        assert!(result.is_ok());
        
        let prompt_result = result.unwrap();
        assert!(prompt_result["description"].is_string());
        assert!(prompt_result["messages"].is_array());
        
        // Test getting a non-existent prompt
        let result = server.get_prompt_by_name("nonexistent", None);
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_prompt_template_substitution() {
        let server = MCPServer::new();
        server.initialize().await.unwrap();
        
        // Test with template arguments
        let args = serde_json::json!({
            "topic": "rust programming"
        });
        
        let result = server.get_prompt_by_name("help", Some(&args));
        assert!(result.is_ok());
        
        let prompt_result = result.unwrap();
        let message_text = prompt_result["messages"][0]["content"]["text"].as_str().unwrap();
        
        // The template should have substituted the topic
        // Note: This test assumes the help prompt has a {{topic}} placeholder
        if message_text.contains("{{topic}}") {
            // If the placeholder wasn't substituted, we need to check our template logic
            assert!(message_text.contains("rust programming"), 
                "Template substitution should replace {{topic}} with the provided value");
        }
    }
}
