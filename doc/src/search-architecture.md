# Search Architecture

SwissArmyHammer implements a multi-tiered search architecture that combines traditional text search with modern semantic search capabilities. This document provides a deep dive into the search system's architecture, indexing strategies, and performance characteristics.

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     Search Frontend                        │
├─────────────────────────────────────────────────────────────┤
│  CLI Interface    │  MCP Server    │  Library API          │
└─────────────────────┬───────────────┬─────────────────────────┘
                      │               │
┌─────────────────────┴───────────────┴─────────────────────────┐
│                   Search Engine                             │
├─────────────────────────────────────────────────────────────┤
│  Hybrid Search Controller                                   │
│  ├─ Query Routing Logic                                     │
│  ├─ Result Aggregation                                      │
│  └─ Score Normalization                                     │
└─────────────────────┬───────────────┬─────────────────────────┘
                      │               │
┌─────────────────────┴───────────────┴─────────────────────────┐
│               Search Backends                               │
├─────────────────────────────────────────────────────────────┤
│  Fuzzy Search     │  Full-Text      │  Semantic Search       │
│  (SkimMatcher)    │  (Tantivy)      │  (Embeddings+DuckDB)   │
└─────────────────────┬───────────────┬─────────────────────────┘
                      │               │
┌─────────────────────┴───────────────┴─────────────────────────┐
│                 Storage Layer                               │
├─────────────────────────────────────────────────────────────┤
│  In-Memory        │  File System    │  Vector Database       │
│  (RAM Index)      │  (Persistent)   │  (DuckDB + VSS)        │
└─────────────────────────────────────────────────────────────┘
```

## Search Engines

### 1. Fuzzy Search Engine

**Implementation**: `search.rs::SearchEngine::fuzzy_search`

**Technology Stack**:
- **Matcher**: `skim_matcher_v2` (fuzzy string matching)
- **Storage**: In-memory prompt collection
- **Indexing**: None (searches on-demand)

**Architecture**:
```rust
pub struct SearchEngine {
    fuzzy_matcher: SkimMatcherV2,
    // Other fields...
}
```

**Performance Characteristics**:
- **Time Complexity**: O(n*m) where n=prompts, m=avg prompt length
- **Space Complexity**: O(1) additional memory
- **Latency**: 1-5ms for typical collections
- **Strengths**: Fast, handles typos, no indexing overhead
- **Weaknesses**: No semantic understanding, limited scalability

### 2. Full-Text Search Engine

**Implementation**: `search.rs::SearchEngine::search`

**Technology Stack**:
- **Search Engine**: Apache Tantivy
- **Storage**: RAM index with optional persistence
- **Query Language**: Lucene-compatible syntax

**Architecture**:
```rust
pub struct SearchEngine {
    index: Index,
    writer: IndexWriter,
    name_field: Field,
    description_field: Field,
    category_field: Field,
    tags_field: Field,
    template_field: Field,
}
```

**Index Schema**:
- `name`: Prompt name (TEXT | STORED)
- `description`: Prompt description (TEXT | STORED)  
- `category`: Prompt category (TEXT | STORED)
- `tags`: Space-separated tags (TEXT | STORED)
- `template`: Prompt content (TEXT only)

**Performance Characteristics**:
- **Indexing**: O(n log n) build time, O(log n) updates
- **Query**: O(log n) typical case
- **Memory**: ~50MB writer buffer + index size
- **Strengths**: Boolean queries, exact matching, fast retrieval
- **Weaknesses**: No semantic understanding, requires exact terms

### 3. Semantic Search Engine

**Implementation**: `semantic/` modules

**Technology Stack**:
- **Embeddings**: ONNX Runtime with transformer models
- **Vector Storage**: DuckDB with vector similarity search extension
- **Code Parsing**: TreeSitter multi-language parser
- **File Processing**: Async I/O with tokio

**Architecture**:
```rust
pub struct SemanticSearcher {
    storage: VectorStorage,
    embedding_engine: EmbeddingEngine,
    config: SemanticConfig,
}

pub struct VectorStorage {
    db: Connection,
    embeddings_table: String,
    chunks_table: String,
}

pub struct EmbeddingEngine {
    session: Session,
    tokenizer: Tokenizer,
    model_config: ModelConfig,
}
```

**Database Schema**:
```sql
-- Code chunks table
CREATE TABLE chunks (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    content TEXT NOT NULL,
    language TEXT,
    start_line INTEGER,
    end_line INTEGER,
    chunk_type TEXT,
    created_at TIMESTAMP DEFAULT now()
);

-- Vector embeddings table
CREATE TABLE embeddings (
    chunk_id TEXT PRIMARY KEY,
    embedding FLOAT[],
    embedding_model TEXT,
    created_at TIMESTAMP DEFAULT now(),
    FOREIGN KEY (chunk_id) REFERENCES chunks(id)
);
```

**Performance Characteristics**:
- **Indexing**: O(n*k) where k=embedding dimension (384-1536)
- **Query**: O(n) similarity calculation with HNSW optimization
- **Memory**: Model size (100MB-2GB) + embeddings cache
- **Strengths**: Semantic understanding, cross-language search
- **Weaknesses**: High memory usage, slower than text search

## Indexing Strategies

### Text Index Management

**Index Creation**:
```rust
// In-memory index (default)
let engine = SearchEngine::new()?;

// Persistent index
let engine = SearchEngine::with_directory("/path/to/index")?;
```

**Index Updates**:
- **Incremental**: Add/remove individual prompts
- **Batch**: Bulk updates with commit optimization
- **Rebuild**: Full index reconstruction

**Index Persistence**:
- Memory-mapped files for fast loading
- Atomic commits to prevent corruption
- Configurable buffer sizes for performance tuning

### Vector Index Management

**Embedding Generation**:
```rust
// Code chunk processing pipeline
CodeFile -> TreeSitter Parse -> Chunks -> Embeddings -> Storage
```

**Vector Storage**:
- **Chunking Strategy**: Function/class-level granularity
- **Embedding Models**: Configurable ONNX models
- **Similarity Metrics**: Cosine similarity (default)
- **Index Types**: HNSW for approximate nearest neighbor

**Update Strategies**:
- **Lazy Updates**: Generate embeddings on first search
- **Eager Updates**: Pre-compute all embeddings
- **Incremental**: Update only changed files

## Query Processing Pipeline

### 1. Query Analysis
```rust
enum QueryType {
    Simple(String),
    Regex(String),
    Boolean(BooleanQuery),
    Semantic(SemanticQuery),
}
```

### 2. Strategy Selection
```rust
impl SearchEngine {
    fn select_strategy(&self, query: &Query) -> SearchStrategy {
        match query {
            Query { regex: true, .. } => SearchStrategy::Regex,
            Query { semantic: true, .. } => SearchStrategy::Semantic,
            Query { fuzzy: true, .. } => SearchStrategy::Fuzzy,
            _ => SearchStrategy::Hybrid,
        }
    }
}
```

### 3. Result Aggregation
```rust
pub fn hybrid_search(&self, query: &str, prompts: &[Prompt]) -> Result<Vec<SearchResult>> {
    let mut results = HashMap::new();
    
    // Combine multiple search strategies
    let text_results = self.search(query, prompts)?;
    let fuzzy_results = self.fuzzy_search(query, prompts);
    
    // Merge and deduplicate results
    // Score normalization and ranking
    
    Ok(final_results)
}
```

## Performance Optimization

### Caching Strategies

**Query Result Caching**:
- LRU cache for frequent queries
- TTL-based invalidation
- Configurable cache size limits

**Index Caching**:
- Memory-mapped index files
- Lazy loading of index segments
- Background index warming

**Embedding Caching**:
- Persistent embedding storage
- Model result caching
- Batch processing optimization

### Memory Management

**Memory Usage Patterns**:
```
Component               | Memory Usage
------------------------|----------------------------------
Fuzzy Search           | O(1) - no additional memory
Full-Text Index        | O(index_size) ~ 10-20% of data
Semantic Embeddings    | O(chunks * dimensions) ~ 1-5GB
Model Loading          | 100MB - 2GB depending on model
Result Sets            | O(result_count * prompt_size)
```

**Optimization Strategies**:
- Stream processing for large result sets
- Configurable result limits
- Memory-mapped storage for large indices
- Model quantization for embedding models

### Performance Tuning

**Configuration Parameters**:
```rust
pub struct SearchConfig {
    // Full-text search
    pub tantivy_writer_buffer_size: usize,
    pub tantivy_merge_policy: MergePolicy,
    
    // Semantic search
    pub embedding_batch_size: usize,
    pub similarity_threshold: f32,
    pub max_results_per_query: usize,
    
    // Hybrid search
    pub score_combination_strategy: ScoreStrategy,
    pub result_deduplication: bool,
}
```

**Benchmarking Results**:
```
Search Type     | Latency (p50) | Latency (p99) | Throughput
----------------|---------------|---------------|------------
Fuzzy Search    | 2ms          | 8ms           | 500 qps
Full-Text       | 5ms          | 15ms          | 200 qps
Semantic        | 50ms         | 150ms         | 20 qps
Hybrid          | 25ms         | 80ms          | 40 qps
```

## Scalability Considerations

### Data Size Limits

**Recommended Limits**:
- **Prompts**: Up to 100,000 prompts
- **Code Files**: Up to 1 million files
- **Index Size**: Up to 10GB total
- **Concurrent Users**: Up to 100 simultaneous searches

### Scaling Strategies

**Horizontal Scaling**:
- Distributed search with result merging
- Sharded indices by category/source
- Load balancing across search nodes

**Vertical Scaling**:
- Memory optimization for large datasets
- SSD storage for persistent indices
- GPU acceleration for semantic search

## Integration Points

### MCP Server Integration
```rust
pub struct McpSearchHandler {
    search_engine: Arc<SearchEngine>,
    semantic_searcher: Arc<SemanticSearcher>,
}
```

### CLI Integration
```rust
pub async fn handle_search_command(
    args: SearchArgs,
    prompt_library: &PromptLibrary,
) -> Result<()> {
    // Route to appropriate search engine
    // Format results for terminal display
}
```

### Library API
```rust
pub trait SearchProvider {
    fn search(&self, query: &SearchQuery) -> Result<Vec<SearchResult>>;
    fn suggest(&self, partial: &str) -> Result<Vec<String>>;
    fn similar(&self, item_id: &str) -> Result<Vec<SearchResult>>;
}
```

## Future Enhancements

### Planned Features
- **Vector Database**: Migration to specialized vector DB (Qdrant/Weaviate)
- **Hybrid Retrieval**: BM25 + vector search combination
- **Query Expansion**: Automatic query term expansion
- **Personalization**: User-specific result ranking
- **Real-time Updates**: Streaming index updates

### Research Directions
- **Multi-modal Embeddings**: Code + documentation + comments
- **Graph-based Search**: Code dependency graph traversal
- **Federated Search**: Cross-repository search capabilities
- **Explainable Rankings**: Search result explanations

## Troubleshooting

### Common Issues

**Index Corruption**:
```bash
# Rebuild text index
rm -rf ~/.swissarmyhammer/index
swissarmyhammer search --rebuild-index

# Rebuild semantic index
swissarmyhammer search --rebuild-embeddings
```

**Memory Issues**:
```bash
# Reduce memory usage
export SWISSARMYHAMMER_MAX_RESULTS=100
export SWISSARMYHAMMER_EMBEDDING_BATCH_SIZE=10
```

**Performance Problems**:
```bash
# Enable search timing
swissarmyhammer search --timing "query"

# Profile search performance
swissarmyhammer search --profile "detailed query"
```

### Monitoring

**Metrics to Track**:
- Query latency percentiles
- Index size growth
- Memory usage patterns
- Cache hit rates
- Error rates by search type

**Logging Configuration**:
```rust
// Enable debug logging for search
RUST_LOG=swissarmyhammer::search=debug cargo run
```

## See Also

- [Search Guide](./search-guide.md) - User guide for search features
- [CLI Search Reference](./cli-search.md) - Command-line interface
- [Performance Tuning](./performance.md) - Optimization guidelines
- [Index Management](./index-management.md) - Index maintenance guide