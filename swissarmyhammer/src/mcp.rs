//! Model Context Protocol (MCP) server support

use crate::{Prompt, PromptLibrary, Result, SwissArmyHammerError};
use std::sync::Arc;
use tokio::sync::RwLock;

/// MCP server for serving prompts
pub struct McpServer {
    library: Arc<RwLock<PromptLibrary>>,
}

impl McpServer {
    /// Create a new MCP server
    pub fn new(library: PromptLibrary) -> Self {
        Self {
            library: Arc::new(RwLock::new(library)),
        }
    }
    
    /// Get the underlying library
    pub fn library(&self) -> &Arc<RwLock<PromptLibrary>> {
        &self.library
    }
    
    /// Get server info
    pub fn info(&self) -> ServerInfo {
        ServerInfo {
            name: "SwissArmyHammer".to_string(),
            version: crate::VERSION.to_string(),
        }
    }
}

/// Server information
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Server version
    pub version: String,
}

// Note: The actual MCP implementation would require the rmcp crate
// to be updated to expose the necessary types and traits.
// For now, this is a placeholder implementation.

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_mcp_server_creation() {
        let library = PromptLibrary::new();
        let server = McpServer::new(library);
        
        let info = server.info();
        assert_eq!(info.name, "SwissArmyHammer");
    }
}