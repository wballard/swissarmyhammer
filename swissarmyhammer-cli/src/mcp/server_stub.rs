use anyhow::Result;
use tokio::sync::oneshot;

#[derive(Clone)]
pub struct MCPServer;

impl MCPServer {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(self, shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        // Stub implementation - MCP server not yet implemented
        // This would require proper implementation with the rmcp crate
        tracing::warn!("MCP server is not yet implemented - using stub");

        // Wait for shutdown signal
        let _ = shutdown_rx.await;

        Ok(())
    }
}

impl Default for MCPServer {
    fn default() -> Self {
        Self::new()
    }
}
