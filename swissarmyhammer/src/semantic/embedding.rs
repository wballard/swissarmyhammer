//! Embedding generation using ONNX Runtime with nomic-embed-code model

use crate::semantic::{CodeChunk, Embedding, Result, SemanticError};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, info};

/// Configuration for the embedding engine
#[derive(Debug, Clone)]
pub struct EmbeddingConfig {
    /// Model identifier for the embedding model
    pub model_id: String,
    /// Device to run the model on (cpu, cuda, auto)
    pub device: String,
    /// Number of texts to process in a single batch
    pub batch_size: usize,
    /// Maximum text length in characters before truncation
    pub max_text_length: usize,
    /// Delay in milliseconds between batches to avoid rate limiting
    pub batch_delay_ms: u64,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            model_id: "nomic-embed-text-v1.5".to_string(),
            device: "api".to_string(), // Using API instead of local device
            batch_size: 10,
            max_text_length: 8000,
            batch_delay_ms: 100,
        }
    }
}

/// Information about the embedding model
#[derive(Debug, Clone)]
pub struct EmbeddingModelInfo {
    /// Model identifier
    pub model_id: String,
    /// Number of dimensions in the embedding vectors
    pub dimensions: usize,
    /// Maximum sequence length the model can handle
    pub max_sequence_length: usize,
    /// Type of quantization used (e.g., FP8, FP16)
    pub quantization: String,
}

/// Request structure for Nomic Atlas API
#[derive(Debug, Serialize)]
struct EmbeddingRequest {
    model: String,
    texts: Vec<String>,
    task_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensionality: Option<usize>,
}

/// Response structure from Nomic Atlas API
#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    embeddings: Vec<Vec<f32>>,
}

/// Embedding engine using Nomic Atlas API
pub struct EmbeddingEngine {
    model_id: String,
    config: EmbeddingConfig,
    client: Arc<Client>,
    api_key: String,
}

impl EmbeddingEngine {
    /// Create new embedding engine with nomic-embed-text-v1.5 model
    pub async fn new() -> Result<Self> {
        let config = EmbeddingConfig::default();
        Self::with_config(config).await
    }

    /// Create embedding engine with custom model
    pub async fn with_model_id(model_id: String) -> Result<Self> {
        let config = EmbeddingConfig {
            model_id,
            ..Default::default()
        };
        Self::with_config(config).await
    }

    /// Create engine with custom configuration
    pub async fn with_config(config: EmbeddingConfig) -> Result<Self> {
        if config.model_id.is_empty() {
            return Err(SemanticError::Config(
                "Model ID cannot be empty".to_string(),
            ));
        }

        // Get API key from environment variable
        let api_key = std::env::var("NOMIC_API_KEY")
            .map_err(|_| SemanticError::Config(
                "NOMIC_API_KEY environment variable is required".to_string(),
            ))?;

        info!("Initializing embedding engine with model: {}", config.model_id);

        // Create HTTP client
        let client = Client::new();

        info!("Successfully initialized embedding engine");

        Ok(Self {
            model_id: config.model_id.clone(),
            config,
            client: Arc::new(client),
            api_key,
        })
    }



    /// Generate embedding for a single code chunk
    pub async fn embed_chunk(&self, chunk: &CodeChunk) -> Result<Embedding> {
        let text = self.prepare_chunk_text(chunk);
        let vector = self.generate_embedding(&text).await?;

        Ok(Embedding {
            chunk_id: chunk.id.clone(),
            vector,
        })
    }

    /// Generate embedding for raw text
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        self.generate_embedding(text).await
    }

    /// Generate embeddings for multiple text strings efficiently
    pub async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();

        // Process in batches to avoid overwhelming the model
        for text_batch in texts.chunks(self.config.batch_size) {
            let batch_results = self.process_text_batch(text_batch).await?;
            embeddings.extend(batch_results);

            // Add small delay between batches to avoid rate limiting
            if text_batch.len() == self.config.batch_size {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    self.config.batch_delay_ms,
                ))
                .await;
            }
        }

        Ok(embeddings)
    }

    /// Generate embeddings for multiple chunks efficiently
    pub async fn embed_chunks_batch(&self, chunks: &[CodeChunk]) -> Result<Vec<Embedding>> {
        let mut embeddings = Vec::new();

        // Process in batches to avoid overwhelming the model
        for chunk_batch in chunks.chunks(self.config.batch_size) {
            let batch_results = self.process_chunk_batch(chunk_batch).await?;
            embeddings.extend(batch_results);

            // Add small delay between batches to avoid rate limiting
            if chunk_batch.len() == self.config.batch_size {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    self.config.batch_delay_ms,
                ))
                .await;
            }
        }

        Ok(embeddings)
    }

    /// Get model information
    pub fn model_info(&self) -> EmbeddingModelInfo {
        EmbeddingModelInfo {
            model_id: self.model_id.clone(),
            dimensions: 384,           // nomic-embed-code dimensions
            max_sequence_length: 8192, // Typical for code embedding models
            quantization: "FP8".to_string(),
        }
    }

    // Private implementation methods

    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Validate input
        if text.is_empty() {
            return Err(SemanticError::Embedding("Empty text provided".to_string()));
        }

        // Clean and truncate text
        let cleaned_text = self.clean_text(text);

        // Prepare text with task prefix for code embeddings
        let prefixed_text = format!("search_document: {}", cleaned_text);

        // Create API request
        let request = EmbeddingRequest {
            model: self.model_id.clone(),
            texts: vec![prefixed_text],
            task_type: "search_document".to_string(),
            dimensionality: Some(384), // Use 384 dimensions to match expected output
        };

        // Make API call
        let response = self.client
            .post("https://api-atlas.nomic.ai/v1/embedding/text")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| SemanticError::Embedding(format!("API request failed: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SemanticError::Embedding(format!(
                "API returned error {}: {}",
                status, error_text
            )));
        }

        // Parse response
        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| SemanticError::Embedding(format!("Failed to parse API response: {}", e)))?;

        // Extract the embedding vector
        let embedding = embedding_response
            .embeddings
            .into_iter()
            .next()
            .ok_or_else(|| SemanticError::Embedding("No embeddings in API response".to_string()))?;

        if embedding.len() != 384 {
            return Err(SemanticError::Embedding(format!(
                "Expected 384-dimensional vector, got {}",
                embedding.len()
            )));
        }

        debug!("Generated embedding with {} dimensions", embedding.len());
        Ok(embedding)
    }


    async fn process_text_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut batch_embeddings = Vec::new();

        for text in texts {
            match self.generate_embedding(text).await {
                Ok(embedding) => {
                    batch_embeddings.push(embedding);
                    tracing::debug!("Generated embedding for text of length: {}", text.len());
                }
                Err(e) => {
                    tracing::error!("Failed to embed text: {}", e);
                    // Continue with other texts instead of failing entire batch
                }
            }
        }

        Ok(batch_embeddings)
    }

    async fn process_chunk_batch(&self, chunks: &[CodeChunk]) -> Result<Vec<Embedding>> {
        let mut batch_embeddings = Vec::new();

        for chunk in chunks {
            match self.embed_chunk(chunk).await {
                Ok(embedding) => {
                    batch_embeddings.push(embedding);
                    tracing::debug!("Generated embedding for chunk: {}", chunk.id);
                }
                Err(e) => {
                    tracing::error!("Failed to embed chunk {}: {}", chunk.id, e);
                    // Continue with other chunks instead of failing entire batch
                }
            }
        }

        Ok(batch_embeddings)
    }

    /// Prepare chunk text for embedding with nomic-embed-code format
    fn prepare_chunk_text(&self, chunk: &CodeChunk) -> String {
        let mut text = String::new();

        // Add language and type context (more concise format for code embedding)
        text.push_str(&format!("{:?} {:?}: ", chunk.language, chunk.chunk_type));
        
        // Add the actual code content directly
        text.push_str(&chunk.content);

        // Clean up the text for better embedding quality
        self.clean_text(&text)
    }

    fn clean_text(&self, text: &str) -> String {
        let mut result = text
            // Remove excessive whitespace from each line (both leading and trailing)
            .lines()
            .map(|line| line.trim())
            .collect::<Vec<_>>()
            .join("\n");
        
        // Remove excessive blank lines (3 or more consecutive newlines become 2)
        while result.contains("\n\n\n") {
            result = result.replace("\n\n\n", "\n\n");
        }
        
        // Truncate if too long (embedding models have token limits)
        result.chars().take(self.config.max_text_length).collect()
    }

    #[cfg(test)]
    /// Create embedding engine for testing without requiring API key
    pub async fn new_for_testing() -> Result<Self> {
        let config = EmbeddingConfig::default();
        Self::with_config_for_testing(config).await
    }

    #[cfg(test)]
    /// Create engine with custom configuration for testing without requiring API key
    pub async fn with_config_for_testing(config: EmbeddingConfig) -> Result<Self> {
        if config.model_id.is_empty() {
            return Err(SemanticError::Config(
                "Model ID cannot be empty".to_string(),
            ));
        }

        info!("Initializing test embedding engine with model: {}", config.model_id);

        // Create HTTP client (won't be used in tests that don't make API calls)
        let client = Client::new();

        info!("Successfully initialized test embedding engine");

        Ok(Self {
            model_id: config.model_id.clone(),
            config,
            client: Arc::new(client),
            api_key: "test-key".to_string(), // Dummy API key for testing
        })
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::semantic::{ChunkType, ContentHash, Language};
    use std::path::PathBuf;

    // Helper function to check if API key is available for integration tests
    fn api_key_available() -> bool {
        std::env::var("NOMIC_API_KEY").is_ok()
    }

    #[tokio::test]
    async fn test_embedding_engine_creation() {
        if !api_key_available() {
            println!("Skipping test_embedding_engine_creation: NOMIC_API_KEY not set");
            return;
        }
        let engine = EmbeddingEngine::new().await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_embedding_engine_with_model_id() {
        if !api_key_available() {
            println!("Skipping test_embedding_engine_with_model_id: NOMIC_API_KEY not set");
            return;
        }
        let engine = EmbeddingEngine::with_model_id("test-model".to_string()).await;
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        assert_eq!(engine.model_id, "test-model");
    }

    #[tokio::test]
    async fn test_embedding_engine_with_config() {
        if !api_key_available() {
            println!("Skipping test_embedding_engine_with_config: NOMIC_API_KEY not set");
            return;
        }
        let config = EmbeddingConfig {
            model_id: "custom-model".to_string(),
            device: "cpu".to_string(),
            batch_size: 5,
            max_text_length: 4000,
            batch_delay_ms: 50,
        };

        let engine = EmbeddingEngine::with_config(config.clone()).await;
        assert!(engine.is_ok());

        let engine = engine.unwrap();
        assert_eq!(engine.model_id, "custom-model");
        assert_eq!(engine.config.batch_size, 5);
    }

    #[tokio::test]
    async fn test_embedding_engine_invalid_config() {
        let config = EmbeddingConfig {
            model_id: "".to_string(),
            ..Default::default()
        };

        let engine = EmbeddingEngine::with_config(config).await;
        assert!(engine.is_err());
    }

    #[tokio::test]
    async fn test_embed_text() {
        if !api_key_available() {
            println!("Skipping test_embed_text: NOMIC_API_KEY not set");
            return;
        }
        let engine = EmbeddingEngine::new().await.unwrap();
        let embedding = engine.embed_text("fn main() {}").await;

        assert!(embedding.is_ok());
        let embedding = embedding.unwrap();
        assert_eq!(embedding.len(), 384);

        // Check that embedding values are normalized (typical for embedding models)
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001); // Should be approximately 1.0
    }

    #[tokio::test]
    async fn test_embed_text_empty() {
        if !api_key_available() {
            println!("Skipping test_embed_text_empty: NOMIC_API_KEY not set");
            return;
        }
        let engine = EmbeddingEngine::new().await.unwrap();
        let embedding = engine.embed_text("").await;

        assert!(embedding.is_err());
    }

    #[tokio::test]
    async fn test_embed_chunk() {
        if !api_key_available() {
            println!("Skipping test_embed_chunk: NOMIC_API_KEY not set");
            return;
        }
        let engine = EmbeddingEngine::new().await.unwrap();

        let chunk = CodeChunk {
            id: "test_chunk".to_string(),
            file_path: PathBuf::from("test.rs"),
            language: Language::Rust,
            content: "fn main() {\n    println!(\"Hello, world!\");\n}".to_string(),
            start_line: 1,
            end_line: 3,
            chunk_type: ChunkType::Function,
            content_hash: ContentHash("hash123".to_string()),
        };

        let embedding = engine.embed_chunk(&chunk).await;
        assert!(embedding.is_ok());

        let embedding = embedding.unwrap();
        assert_eq!(embedding.chunk_id, "test_chunk");
        assert_eq!(embedding.vector.len(), 384);
    }

    #[tokio::test]
    async fn test_embed_batch() {
        if !api_key_available() {
            println!("Skipping test_embed_batch: NOMIC_API_KEY not set");
            return;
        }
        let engine = EmbeddingEngine::new().await.unwrap();
        let texts = vec!["fn main() {}", "println!(\"hello\");"];
        let embeddings = engine.embed_batch(&texts).await;

        assert!(embeddings.is_ok());
        let embeddings = embeddings.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
        assert_eq!(embeddings[1].len(), 384);
    }

    #[tokio::test]
    async fn test_embed_chunks_batch() {
        if !api_key_available() {
            println!("Skipping test_embed_chunks_batch: NOMIC_API_KEY not set");
            return;
        }
        let engine = EmbeddingEngine::new().await.unwrap();

        let chunks = vec![
            CodeChunk {
                id: "chunk1".to_string(),
                file_path: PathBuf::from("test1.rs"),
                language: Language::Rust,
                content: "fn test1() {}".to_string(),
                start_line: 1,
                end_line: 1,
                chunk_type: ChunkType::Function,
                content_hash: ContentHash("hash1".to_string()),
            },
            CodeChunk {
                id: "chunk2".to_string(),
                file_path: PathBuf::from("test2.rs"),
                language: Language::Rust,
                content: "fn test2() {}".to_string(),
                start_line: 1,
                end_line: 1,
                chunk_type: ChunkType::Function,
                content_hash: ContentHash("hash2".to_string()),
            },
        ];

        let embeddings = engine.embed_chunks_batch(&chunks).await;
        assert!(embeddings.is_ok());

        let embeddings = embeddings.unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].chunk_id, "chunk1");
        assert_eq!(embeddings[1].chunk_id, "chunk2");
        assert_eq!(embeddings[0].vector.len(), 384);
        assert_eq!(embeddings[1].vector.len(), 384);
    }

    #[test]
    fn test_model_info() {
        let engine_result = futures::executor::block_on(EmbeddingEngine::new_for_testing());
        let engine = engine_result.unwrap();

        let info = engine.model_info();
        assert_eq!(info.model_id, "nomic-embed-text-v1.5");
        assert_eq!(info.dimensions, 384);
        assert_eq!(info.max_sequence_length, 8192);
        assert_eq!(info.quantization, "FP8");
    }

    #[test]
    fn test_prepare_chunk_text() {
        let engine = futures::executor::block_on(EmbeddingEngine::new_for_testing()).unwrap();

        let chunk = CodeChunk {
            id: "test_chunk".to_string(),
            file_path: PathBuf::from("test.rs"),
            language: Language::Rust,
            content: "fn main() {}".to_string(),
            start_line: 1,
            end_line: 1,
            chunk_type: ChunkType::Function,
            content_hash: ContentHash("hash123".to_string()),
        };

        let prepared_text = engine.prepare_chunk_text(&chunk);
        assert!(prepared_text.contains("Rust"));
        assert!(prepared_text.contains("Function"));
        assert!(prepared_text.contains("fn main() {}"));
    }

    #[test]
    fn test_clean_text() {
        let engine = futures::executor::block_on(EmbeddingEngine::new_for_testing()).unwrap();

        let text = "line1  \n\n\n\nline2\n   line3   \n\n\n\nline4";
        let cleaned = engine.clean_text(text);

        // Should remove excessive whitespace and blank lines
        assert!(!cleaned.contains("   "));
        assert!(!cleaned.contains("\n\n\n"));
        assert!(cleaned.contains("line1"));
        assert!(cleaned.contains("line2"));
        assert!(cleaned.contains("line3"));
        assert!(cleaned.contains("line4"));
    }

    #[test]
    fn test_clean_text_truncation() {
        let mut config = EmbeddingConfig::default();
        config.max_text_length = 10;

        let engine = futures::executor::block_on(EmbeddingEngine::with_config_for_testing(config)).unwrap();

        let long_text = "This is a very long text that should be truncated";
        let cleaned = engine.clean_text(long_text);

        assert_eq!(cleaned.len(), 10);
        assert_eq!(cleaned, "This is a ");
    }

    #[test]
    fn test_embedding_config_default() {
        let config = EmbeddingConfig::default();
        assert_eq!(config.model_id, "nomic-embed-text-v1.5");
        assert_eq!(config.device, "api");
        assert_eq!(config.batch_size, 10);
        assert_eq!(config.max_text_length, 8000);
        assert_eq!(config.batch_delay_ms, 100);
    }

}
