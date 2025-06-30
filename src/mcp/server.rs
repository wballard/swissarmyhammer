use anyhow::Result;
use rmcp::{
    model::{Implementation, ServerCapabilities, ServerInfo},
    tool, ServerHandler, ServiceExt,
};
use tokio::sync::oneshot;
use tracing::info;
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
            capabilities: ServerCapabilities::builder().enable_tools().build(),
        }
    }
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
}
