//! Example showing async usage and MCP server integration

#[cfg(feature = "mcp")]
use swissarmyhammer::{mcp::McpServer, Prompt, PromptLibrary};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "mcp")]
    {
        // Create and populate a prompt library
        let mut library = PromptLibrary::new();

        // Add some example prompts
        library.add(
            Prompt::new("explain-code", "Explain the following code:\n\n{{ code }}")
                .with_description("Get a detailed explanation of code")
                .with_category("education"),
        )?;

        library.add(
            Prompt::new("translate", "Translate the following text from {{ source_lang }} to {{ target_lang }}:\n\n{{ text }}")
                .with_description("Translate text between languages")
                .with_category("translation")
        )?;

        library.add(
            Prompt::new(
                "summarize",
                "Summarize the following text in {{ style }} style:\n\n{{ text }}",
            )
            .with_description("Create summaries in different styles")
            .with_category("writing"),
        )?;

        // Create an MCP server
        let server = McpServer::new(library);

        println!("MCP Server Information:");
        let info = server.info();
        println!("  Name: {}", info.name);
        println!("  Version: {}", info.version);

        // In a real application, you would run the server like this:
        // server.run().await?;

        // For this example, we'll just demonstrate the capabilities
        println!("\nServer capabilities include prompt support");

        // The server would handle MCP protocol requests for:
        // - Listing available prompts
        // - Getting prompt details
        // - Rendering prompts with arguments
    }

    #[cfg(not(feature = "mcp"))]
    {
        println!("This example requires the 'mcp' feature to be enabled.");
        println!("Run with: cargo run --example async_and_mcp --features mcp");
    }

    Ok(())
}

// Example of a custom storage backend (async)
#[cfg(feature = "async")]
mod custom_storage {
    use std::collections::HashMap;
    use swissarmyhammer::{Prompt, Result, SwissArmyHammerError};
    use tokio::sync::RwLock;

    /// Example async storage backend using tokio RwLock
    #[allow(dead_code)]
    pub struct AsyncStorage {
        prompts: RwLock<HashMap<String, Prompt>>,
    }

    #[allow(dead_code)]
    impl AsyncStorage {
        pub fn new() -> Self {
            Self {
                prompts: RwLock::new(HashMap::new()),
            }
        }
    }

    // Note: This would require an async trait in the actual implementation
    // For now, this is just an example of how it could work
    #[allow(dead_code)]
    impl AsyncStorage {
        pub async fn async_store(&self, prompt: Prompt) -> Result<()> {
            let mut prompts = self.prompts.write().await;
            prompts.insert(prompt.name.clone(), prompt);
            Ok(())
        }

        pub async fn async_get(&self, name: &str) -> Result<Prompt> {
            let prompts = self.prompts.read().await;
            prompts
                .get(name)
                .cloned()
                .ok_or_else(|| SwissArmyHammerError::PromptNotFound(name.to_string()))
        }

        pub async fn async_list(&self) -> Result<Vec<Prompt>> {
            let prompts = self.prompts.read().await;
            Ok(prompts.values().cloned().collect())
        }
    }
}
