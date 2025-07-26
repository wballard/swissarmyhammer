# Index Management Guide

This guide covers the management, maintenance, and optimization of SwissArmyHammer's search indices. The system maintains multiple types of indices to support different search strategies, each requiring specific management approaches.

## Index Types Overview

SwissArmyHammer maintains three distinct index types:

1. **Text Index**: Tantivy-based full-text search index
2. **Vector Index**: DuckDB-based semantic embeddings storage
3. **In-Memory Cache**: Runtime fuzzy search acceleration

## Text Index Management

### Index Location and Structure

**Default Locations**:
```bash
# User data directory
~/.swissarmyhammer/index/          # Text indices
~/.swissarmyhammer/cache/          # Temporary cache files

# Project-specific indices
.swissarmyhammer/index/            # Local project indices
.swissarmyhammer/semantic.db       # Semantic database
```

**Index Structure**:
```
index/
├── meta.json                      # Index metadata
├── segments/                      # Tantivy segments
│   ├── segment_0/
│   ├── segment_1/
│   └── ...
└── .managed                       # Management marker
```

### Index Creation and Rebuilding

**Automatic Index Creation**:
```bash
# Index is created automatically on first search
swissarmyhammer search "query"

# Force index rebuild
swissarmyhammer search --rebuild-index
```

**Manual Index Management**:
```bash
# Check index status
swissarmyhammer doctor --check-indices

# Rebuild all indices
swissarmyhammer index rebuild --all

# Rebuild specific index type
swissarmyhammer index rebuild --text
swissarmyhammer index rebuild --semantic
```

**Programmatic Index Creation**:
```rust
use swissarmyhammer::search::SearchEngine;

// Create in-memory index
let mut engine = SearchEngine::new()?;

// Create persistent index
let mut engine = SearchEngine::with_directory("/path/to/index")?;

// Index prompts
engine.index_prompts(&prompts)?;
engine.commit()?;
```

### Index Updates and Maintenance

**Incremental Updates**:
```rust
// Add new prompt to index
engine.index_prompt(&new_prompt)?;
engine.commit()?;

// Batch updates for better performance
for prompt in new_prompts {
    engine.index_prompt(&prompt)?;
}
engine.commit()?; // Single commit at the end
```

**Index Optimization**:
```bash
# Optimize index segments
swissarmyhammer index optimize

# Force merge segments
swissarmyhammer index merge --force

# Compact index storage
swissarmyhammer index compact
```

**Configuration Options**:
```rust
// Tantivy writer configuration
let writer = index.writer_with_num_threads(2, 50_000_000)?;

// Custom merge policy
use tantivy::merge_policy::LogMergePolicy;
let merge_policy = LogMergePolicy::default()
    .set_min_merge_size(8)
    .set_min_layer_size(10_000);
```

### Text Index Monitoring

**Index Statistics**:
```bash
# Get index information
swissarmyhammer index stats

# Detailed segment information
swissarmyhammer index stats --detailed
```

**Performance Metrics**:
```rust
pub struct IndexStats {
    pub total_documents: usize,
    pub total_segments: usize,
    pub index_size_bytes: u64,
    pub last_commit_timestamp: DateTime<Utc>,
    pub avg_query_time_ms: f64,
}
```

## Vector Index Management

### Semantic Database Structure

**Database Schema**:
```sql
-- Code chunks metadata
CREATE TABLE chunks (
    id TEXT PRIMARY KEY,
    file_path TEXT NOT NULL,
    content TEXT NOT NULL,
    language TEXT,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    chunk_type TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT now(),
    updated_at TIMESTAMP DEFAULT now()
);

-- Vector embeddings
CREATE TABLE embeddings (
    chunk_id TEXT PRIMARY KEY,
    embedding FLOAT[] NOT NULL,
    embedding_model TEXT NOT NULL,
    model_version TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT now(),
    FOREIGN KEY (chunk_id) REFERENCES chunks(id) ON DELETE CASCADE
);

-- Index metadata
CREATE TABLE index_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP DEFAULT now()
);
```

### Semantic Index Operations

**Database Initialization**:
```bash
# Initialize semantic database
swissarmyhammer semantic init

# Check database schema
swissarmyhammer semantic schema --verify

# Migrate database schema
swissarmyhammer semantic migrate
```

**Embedding Generation**:
```bash
# Generate embeddings for codebase
swissarmyhammer semantic index /path/to/code

# Incremental embedding updates
swissarmyhammer semantic update --since last-commit

# Regenerate embeddings for specific files
swissarmyhammer semantic index --files "*.rs,*.py"
```

**Vector Index Configuration**:
```rust
pub struct SemanticConfig {
    pub database_path: PathBuf,
    pub embedding_model: String,
    pub batch_size: usize,
    pub max_chunk_size: usize,
    pub similarity_threshold: f32,
    pub index_update_strategy: UpdateStrategy,
}
```

### Embedding Model Management

**Model Configuration**:
```bash
# List available embedding models
swissarmyhammer semantic models list

# Download and cache model
swissarmyhammer semantic models download sentence-transformers/all-MiniLM-L6-v2

# Set default model
swissarmyhammer semantic models set-default all-MiniLM-L6-v2

# Model information
swissarmyhammer semantic models info all-MiniLM-L6-v2
```

**Model Switching**:
```bash
# Switch to different model (requires re-indexing)
swissarmyhammer semantic models switch all-mpnet-base-v2 --reindex

# Compare models performance
swissarmyhammer semantic benchmark --models all-MiniLM-L6-v2,all-mpnet-base-v2
```

### Vector Index Maintenance

**Database Maintenance**:
```bash
# Vacuum database
swissarmyhammer semantic vacuum

# Analyze query performance
swissarmyhammer semantic analyze

# Repair corrupted database
swissarmyhammer semantic repair --backup
```

**Embedding Validation**:
```bash
# Validate embedding integrity
swissarmyhammer semantic validate

# Check for orphaned embeddings
swissarmyhammer semantic cleanup --dry-run
swissarmyhammer semantic cleanup --execute

# Verify embedding models consistency
swissarmyhammer semantic verify-models
```

## Performance Optimization

### Index Performance Tuning

**Text Index Optimization**:
```rust
// Writer configuration for performance
pub struct IndexConfig {
    pub writer_buffer_size: usize,      // Default: 50MB
    pub merge_policy: MergePolicy,      // LogMergePolicy recommended
    pub num_threads: usize,             // CPU cores
    pub commit_interval: Duration,      // Auto-commit frequency
}

// Performance settings
let config = IndexConfig {
    writer_buffer_size: 100_000_000,   // 100MB for large datasets
    num_threads: num_cpus::get(),      // Use all CPU cores
    commit_interval: Duration::from_secs(30), // Commit every 30s
    ..Default::default()
};
```

**Vector Index Optimization**:
```rust
pub struct VectorIndexConfig {
    pub batch_size: usize,              // Embedding batch size
    pub connection_pool_size: usize,    // DB connection pool
    pub cache_size: usize,              // Result cache size
    pub similarity_cache_ttl: Duration, // Cache expiration
}
```

### Memory Management

**Memory Usage Patterns**:
```bash
# Monitor memory usage
swissarmyhammer index stats --memory

# Set memory limits
export SWISSARMYHAMMER_MAX_MEMORY=4GB
export SWISSARMYHAMMER_INDEX_CACHE_SIZE=1GB
```

**Memory Optimization Strategies**:
```rust
// Streaming index updates for large datasets
pub fn stream_index_updates<I>(
    engine: &mut SearchEngine,
    prompts: I,
    batch_size: usize,
) -> Result<()>
where
    I: Iterator<Item = Prompt>,
{
    for batch in prompts.chunks(batch_size) {
        for prompt in batch {
            engine.index_prompt(&prompt)?;
        }
        engine.commit()?; // Commit each batch
    }
    Ok(())
}
```

### Storage Optimization

**Disk Usage Management**:
```bash
# Check storage usage
swissarmyhammer index disk-usage

# Clean temporary files
swissarmyhammer index clean --temp

# Archive old indices
swissarmyhammer index archive --older-than 30d
```

**Compression Settings**:
```rust
// Enable index compression
let index = Index::builder()
    .compression(Compression::Lz4)
    .block_size(16384)
    .build()?;
```

## Index Backup and Recovery

### Backup Strategies

**Automated Backups**:
```bash
# Create full backup
swissarmyhammer backup create --type full --output backup.tar.gz

# Create incremental backup
swissarmyhammer backup create --type incremental --since last-backup

# Schedule regular backups
swissarmyhammer backup schedule --daily --retention 7d
```

**Manual Backup**:
```bash
# Backup text indices
tar -czf text-index-backup.tar.gz ~/.swissarmyhammer/index/

# Backup semantic database
cp ~/.swissarmyhammer/semantic.db semantic-backup.db

# Backup configuration
cp -r ~/.swissarmyhammer/config/ config-backup/
```

### Recovery Procedures

**Index Recovery**:
```bash
# Restore from backup
swissarmyhammer backup restore backup.tar.gz

# Rebuild from source
swissarmyhammer index rebuild --from-source

# Partial recovery
swissarmyhammer index recover --text-only
swissarmyhammer index recover --semantic-only
```

**Corruption Recovery**:
```bash
# Detect corruption
swissarmyhammer index verify

# Repair text index
swissarmyhammer index repair --text

# Repair semantic database
swissarmyhammer semantic repair --auto-fix

# Recovery from corruption
swissarmyhammer index recover --rebuild-corrupted
```

## Troubleshooting

### Common Issues

**Index Corruption**:
```bash
# Symptoms
# - Search results incomplete
# - Index verification failures
# - Crash during search operations

# Diagnosis
swissarmyhammer index verify --verbose
swissarmyhammer doctor --check-indices

# Resolution
swissarmyhammer index rebuild --force
```

**Performance Issues**:
```bash
# Symptoms
# - Slow search responses
# - High memory usage
# - Long indexing times

# Diagnosis
swissarmyhammer index stats --performance
swissarmyhammer index analyze --slow-queries

# Resolution
swissarmyhammer index optimize --aggressive
swissarmyhammer index config --tune-performance
```

**Storage Issues**:
```bash
# Symptoms
# - Disk space errors
# - Cannot create index
# - Write permission errors

# Diagnosis
swissarmyhammer index disk-usage --detailed
swissarmyhammer doctor --check-permissions

# Resolution
swissarmyhammer index clean --aggressive
sudo chown -R $USER ~/.swissarmyhammer/
```

### Diagnostic Commands

**Index Health Check**:
```bash
# Comprehensive health check
swissarmyhammer doctor --indices

# Specific checks
swissarmyhammer index health --text
swissarmyhammer index health --semantic
swissarmyhammer index health --all
```

**Performance Analysis**:
```bash
# Query performance profiling
swissarmyhammer search --profile "test query"

# Index performance metrics
swissarmyhammer index benchmark

# Memory usage analysis
swissarmyhammer index memory-profile
```

**Debug Information**:
```bash
# Enable debug logging
RUST_LOG=swissarmyhammer::search=debug swissarmyhammer search query

# Export debug information
swissarmyhammer debug export --indices

# Generate diagnostic report
swissarmyhammer doctor --report diagnostic-report.txt
```

## Best Practices

### Index Management Best Practices

1. **Regular Maintenance**:
   - Schedule weekly index optimization
   - Monitor index size growth
   - Clean temporary files regularly

2. **Performance Monitoring**:
   - Track query response times
   - Monitor memory usage patterns
   - Set up alerts for index corruption

3. **Backup Strategy**:
   - Daily incremental backups
   - Weekly full backups
   - Test recovery procedures regularly

4. **Configuration Management**:
   - Version control index configuration
   - Document performance tuning changes
   - Test configuration changes in development

### Development Workflow Integration

**CI/CD Integration**:
```yaml
# Example GitHub Actions workflow
- name: Update Search Index
  run: |
    swissarmyhammer index update --check-changes
    swissarmyhammer index optimize
    swissarmyhammer index verify
```

**Pre-commit Hooks**:
```bash
#!/bin/bash
# .git/hooks/pre-commit
swissarmyhammer index update --incremental
swissarmyhammer index verify --quick
```

## Configuration Reference

### Environment Variables

```bash
# Index locations
export SWISSARMYHAMMER_INDEX_DIR="/custom/index/path"
export SWISSARMYHAMMER_SEMANTIC_DB="/custom/semantic.db"

# Performance tuning
export SWISSARMYHAMMER_INDEX_BUFFER_SIZE="100MB"
export SWISSARMYHAMMER_MAX_THREADS="8"
export SWISSARMYHAMMER_CACHE_SIZE="1GB"

# Maintenance settings
export SWISSARMYHAMMER_AUTO_OPTIMIZE="true"
export SWISSARMYHAMMER_BACKUP_RETENTION="30d"
```

### Configuration File

```toml
# ~/.swissarmyhammer/config.toml
[index]
buffer_size = "50MB"
auto_optimize = true
backup_enabled = true

[index.text]
merge_policy = "log"
commit_interval = "30s"

[index.semantic]
model = "all-MiniLM-L6-v2"
batch_size = 32
similarity_threshold = 0.7

[performance]
max_results = 1000
cache_ttl = "1h"
memory_limit = "4GB"
```

## See Also

- [Search Architecture](./search-architecture.md) - System architecture overview
- [Search Guide](./search-guide.md) - User guide for search features
- [Performance Tuning](./performance.md) - Advanced optimization
- [Troubleshooting](./troubleshooting.md) - General troubleshooting guide