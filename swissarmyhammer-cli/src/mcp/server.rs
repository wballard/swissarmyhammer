use anyhow::Result;
use tokio::sync::oneshot;
use tracing::{info, error, debug};
use std::sync::{Arc, Mutex};
use swissarmyhammer::PromptLibrary;
use serde_json::{json, Value};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use std::collections::HashMap;

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

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> &str {
        &self.version
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

    pub async fn run(self, mut shutdown_rx: oneshot::Receiver<()>) -> Result<()> {
        info!("Starting MCP server - real implementation active");

        // Initialize prompts
        self.initialize().await?;

        info!("MCP server initialized with {} prompts", {
            let library = self.library.lock().unwrap();
            library.list().unwrap_or_default().len()
        });

        // Create stdin/stdout handles
        let stdin = tokio::io::stdin();
        let mut stdout = tokio::io::stdout();
        let mut reader = BufReader::new(stdin);
        let mut line = String::new();

        info!("MCP server listening on stdio");

        loop {
            tokio::select! {
                // Handle shutdown signal
                _ = &mut shutdown_rx => {
                    info!("Shutdown signal received");
                    break;
                }
                
                // Handle incoming requests
                result = reader.read_line(&mut line) => {
                    match result {
                        Ok(0) => {
                            // EOF reached
                            debug!("EOF reached on stdin");
                            break;
                        }
                        Ok(_) => {
                            // Process the request
                            let trimmed = line.trim();
                            if !trimmed.is_empty() {
                                debug!("Received request: {}", trimmed);
                                
                                if let Ok(request) = serde_json::from_str::<Value>(trimmed) {
                                    let response = self.handle_request(request).await;
                                    let response_json = serde_json::to_string(&response)?;
                                    stdout.write_all(response_json.as_bytes()).await?;
                                    stdout.write_all(b"\n").await?;
                                    stdout.flush().await?;
                                    debug!("Sent response: {}", response_json);
                                } else {
                                    error!("Failed to parse JSON request: {}", trimmed);
                                }
                            }
                            line.clear();
                        }
                        Err(e) => {
                            error!("Error reading from stdin: {}", e);
                            break;
                        }
                    }
                }
            }
        }
        
        info!("MCP server stopped");
        Ok(())
    }

    pub async fn handle_request(&self, request: Value) -> Value {
        let method = request.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = request.get("id");
        let params = request.get("params");

        debug!("Handling method: {}", method);

        match method {
            "initialize" => self.handle_initialize(id, params),
            "prompts/list" => self.handle_prompts_list(id, params),
            "prompts/get" => self.handle_prompts_get(id, params).await,
            _ => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32601,
                        "message": "Method not found"
                    }
                })
            }
        }
    }

    fn handle_initialize(&self, id: Option<&Value>, _params: Option<&Value>) -> Value {
        json!({
            "jsonrpc": "2.0",
            "id": id,
            "result": {
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "prompts": {
                        "listChanged": true
                    }
                },
                "serverInfo": {
                    "name": self.name,
                    "version": self.version
                }
            }
        })
    }

    fn handle_prompts_list(&self, id: Option<&Value>, _params: Option<&Value>) -> Value {
        let library = self.library.lock().unwrap();
        
        match library.list() {
            Ok(prompts) => {
                let prompt_list: Vec<Value> = prompts.iter().map(|p| {
                    let arguments = if p.arguments.is_empty() {
                        None
                    } else {
                        Some(p.arguments.iter().map(|arg| {
                            json!({
                                "name": arg.name,
                                "description": arg.description,
                                "required": arg.required
                            })
                        }).collect::<Vec<Value>>())
                    };

                    json!({
                        "name": p.name,
                        "description": p.description.as_deref().unwrap_or(""),
                        "arguments": arguments
                    })
                }).collect();

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "prompts": prompt_list
                    }
                })
            }
            Err(e) => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": format!("Internal error: {}", e)
                    }
                })
            }
        }
    }

    async fn handle_prompts_get(&self, id: Option<&Value>, params: Option<&Value>) -> Value {
        let params = match params {
            Some(p) => p,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Invalid params"
                    }
                });
            }
        };

        let name = match params.get("name").and_then(|n| n.as_str()) {
            Some(n) => n,
            None => {
                return json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32602,
                        "message": "Missing 'name' parameter"
                    }
                });
            }
        };

        let library = self.library.lock().unwrap();
        
        match library.get(name) {
            Ok(prompt) => {
                // Handle arguments if provided
                let mut content = prompt.template.clone();
                
                if let Some(arguments) = params.get("arguments") {
                    if let Some(args_obj) = arguments.as_object() {
                        let mut template_args = HashMap::new();
                        for (key, value) in args_obj {
                            let value_str = match value {
                                Value::String(s) => s.clone(),
                                v => v.to_string()
                            };
                            template_args.insert(key.clone(), value_str);
                        }
                        
                        // Render the template with arguments
                        match prompt.render(&template_args) {
                            Ok(rendered) => content = rendered,
                            Err(e) => {
                                return json!({
                                    "jsonrpc": "2.0",
                                    "id": id,
                                    "error": {
                                        "code": -32603,
                                        "message": format!("Template rendering error: {}", e)
                                    }
                                });
                            }
                        }
                    }
                }

                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "result": {
                        "description": prompt.description.as_deref().unwrap_or(""),
                        "messages": [{
                            "role": "user",
                            "content": {
                                "type": "text",
                                "text": content
                            }
                        }]
                    }
                })
            }
            Err(e) => {
                json!({
                    "jsonrpc": "2.0",
                    "id": id,
                    "error": {
                        "code": -32603,
                        "message": format!("Prompt not found: {}", e)
                    }
                })
            }
        }
    }
}

impl Default for MCPServer {
    fn default() -> Self {
        Self::new()
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
    async fn test_server_initialization() {
        let server = MCPServer::new();
        // Test that initialization doesn't fail
        let result = server.initialize().await;
        assert!(result.is_ok());
    }
}