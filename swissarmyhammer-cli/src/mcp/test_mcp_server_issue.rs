#[cfg(test)]
mod tests {
    use super::super::MCPServer;

    #[tokio::test]
    async fn test_mcp_server_real_implementation() {
        // Test that we can create and initialize the real MCP server (not stub)
        let server = MCPServer::new();
        
        // Test that the server initializes without error
        let init_result = server.initialize().await;
        assert!(init_result.is_ok(), "Real MCP server should initialize successfully");
        
        // Test that it's the real implementation by checking it has proper methods
        assert_eq!(server.name(), "swissarmyhammer");
        assert!(!server.version().is_empty());
        
        // Test that we can handle requests (this proves it's not a stub)
        let test_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize",
            "id": 1,
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"prompts": {}},
                "clientInfo": {"name": "test", "version": "1.0"}
            }
        });
        
        let response = server.handle_request(test_request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert!(response["result"].is_object());
        
        // Test passes - we have a real implementation, not a stub
    }

    #[tokio::test]
    async fn test_mcp_server_basic_functionality() -> Result<(), Box<dyn std::error::Error>> {
        // Test that we can create an MCP server and it initializes properly
        let server = MCPServer::new();
        
        // Test initialization
        let init_result = server.initialize().await;
        assert!(init_result.is_ok(), "Server initialization should succeed");
        
        // Test that server has the expected name and version
        assert_eq!(server.name(), "swissarmyhammer");
        assert!(!server.version().is_empty());
        
        println!("✓ MCP server basic functionality works");
        Ok(())
    }

    #[tokio::test] 
    async fn test_mcp_server_request_handling() -> Result<(), Box<dyn std::error::Error>> {
        let server = MCPServer::new();
        server.initialize().await?;

        // Test initialize request
        let init_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "initialize", 
            "id": 1,
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {"prompts": {}},
                "clientInfo": {"name": "test-client", "version": "1.0.0"}
            }
        });

        let response = server.handle_request(init_request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 1);
        assert!(response["result"].is_object());
        println!("✓ Initialize request handling works");

        // Test prompts/list request
        let list_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "prompts/list",
            "id": 2,
            "params": {}
        });

        let response = server.handle_request(list_request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 2);
        assert!(response["result"]["prompts"].is_array());
        println!("✓ Prompts list request handling works");

        // Test unknown method
        let unknown_request = serde_json::json!({
            "jsonrpc": "2.0", 
            "method": "unknown/method",
            "id": 3,
            "params": {}
        });

        let response = server.handle_request(unknown_request).await;
        assert_eq!(response["jsonrpc"], "2.0");
        assert_eq!(response["id"], 3);
        assert!(response["error"].is_object());
        assert_eq!(response["error"]["code"], -32601);
        println!("✓ Unknown method error handling works");

        println!("✅ All MCP server request handling tests passed!");
        Ok(())
    }
}