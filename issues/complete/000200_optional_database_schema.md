# Optional Database Schema for Advanced Cost Analytics

## Summary

Implement an optional SQLite database schema for advanced cost analytics and reporting. This provides structured storage for cost data while maintaining the primary markdown-based storage approach.

## Context

While cost data is primarily stored in issue markdown files and in-memory metrics, advanced analytics and historical reporting benefit from structured database storage. This step implements an optional SQLite backend for enhanced cost analysis capabilities.

## Requirements

### Database Schema Design

Implement the schema specified in the PRD:

```sql
CREATE TABLE cost_sessions (
    id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    workflow_run_id TEXT NOT NULL,
    started_at DATETIME NOT NULL,
    completed_at DATETIME,
    total_cost DECIMAL(10,4),
    total_calls INTEGER,
    total_input_tokens INTEGER,
    total_output_tokens INTEGER,
    pricing_model TEXT NOT NULL,
    session_duration_ms INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE api_calls (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    session_id TEXT NOT NULL,
    timestamp DATETIME NOT NULL,
    endpoint TEXT NOT NULL,
    input_tokens INTEGER NOT NULL,
    output_tokens INTEGER NOT NULL,
    duration_ms INTEGER,
    cost DECIMAL(8,4),
    status TEXT NOT NULL,
    FOREIGN KEY (session_id) REFERENCES cost_sessions(id)
);

CREATE INDEX idx_cost_sessions_issue_id ON cost_sessions(issue_id);
CREATE INDEX idx_cost_sessions_started_at ON cost_sessions(started_at);
CREATE INDEX idx_api_calls_session_id ON api_calls(session_id);
CREATE INDEX idx_api_calls_timestamp ON api_calls(timestamp);
```

### Database Features

1. **Optional Storage Backend**
   - Configurable database storage (enabled/disabled)
   - SQLite for simplicity and portability
   - Schema migration support
   - Graceful fallback when database unavailable

2. **Advanced Analytics Support**
   - Complex cost queries and aggregations
   - Historical trend analysis
   - Cross-issue cost comparisons
   - Performance analytics

3. **Data Synchronization**
   - Sync with markdown-based cost sections
   - Maintain data consistency
   - Handle concurrent access
   - Support data export/import

4. **Query Interface**
   - Cost aggregation queries
   - Trend analysis functions
   - Reporting utilities
   - Data export capabilities

### Implementation Strategy

1. **Database Layer**
   - Create cost database module
   - Implement schema migrations
   - Add connection management
   - Support connection pooling

2. **Storage Integration**
   - Optional database writes during cost tracking
   - Sync with existing cost storage
   - Maintain primary markdown storage
   - Handle database failures gracefully

3. **Query API**
   - Cost analysis query functions
   - Aggregation and reporting utilities
   - Export functionality
   - Performance optimizations

## Implementation Details

### File Structure
- Create: `swissarmyhammer/src/cost/database/`
- Add: `mod.rs`, `schema.rs`, `queries.rs`, `migrations.rs`

### Core Components

```rust
pub struct CostDatabase {
    pool: Option<SqlitePool>,
    config: DatabaseConfig,
}

pub struct DatabaseConfig {
    pub enabled: bool,
    pub file_path: PathBuf,
    pub connection_timeout: Duration,
    pub max_connections: u32,
}

impl CostDatabase {
    pub async fn store_session(&self, session: &CostSession) -> Result<()> {
        // Store cost session and associated API calls
    }
    
    pub async fn query_cost_trends(&self, params: TrendQuery) -> Result<Vec<CostTrend>> {
        // Advanced cost trend analysis
    }
}
```

### Migration System
- Schema version tracking
- Automatic migration on startup
- Rollback support for failed migrations
- Schema validation and integrity checks

### Configuration Integration
```yaml
cost_tracking:
  database:
    enabled: false  # Optional by default
    file_path: "./costs.db"
    connection_timeout_seconds: 30
    max_connections: 10
    retention_days: 365
```

## Testing Requirements

### Database Testing
- Schema creation and migration tests
- CRUD operations validation
- Query accuracy verification
- Concurrent access testing

### Integration Testing
- Database sync with markdown storage
- Configuration integration testing
- Fallback behavior when database disabled
- Data consistency validation

### Performance Testing
- Query performance benchmarks
- Large dataset handling
- Connection pool efficiency
- Memory usage validation

## Integration

This step integrates with:
- Step 000192: Configuration system for database settings
- Step 000196: Workflow integration for data storage
- Step 000199: Metrics system for analytics queries

Optional enhancement for:
- Advanced cost reporting
- Historical analysis
- Business intelligence integration

## Dependencies

Add to `Cargo.toml`:
- `sqlx` with SQLite feature for database operations
- `sqlite` for embedded database support

## Success Criteria

- [ ] Complete SQLite schema implementation
- [ ] Optional database storage configuration
- [ ] Schema migration system
- [ ] Advanced analytics query functions
- [ ] Integration with existing cost tracking
- [ ] Comprehensive test coverage
- [ ] Performance validation for database operations

## Notes

- Database storage is completely optional and configurable
- Primary storage remains markdown-based for simplicity
- Database provides enhanced analytics only
- Handle database failures gracefully without affecting core functionality
- Consider future database backends (PostgreSQL, etc.)
- Implement proper connection management and pooling
- Support data export for external analytics tools
- Consider database backup and recovery procedures

## Proposed Solution

After examining the existing cost tracking system, I propose implementing the optional database schema as follows:

### 1. Database Implementation Strategy

**Phase 1: Foundation**
- Add `sqlx` with SQLite feature to `Cargo.toml` workspace dependencies
- Create `swissarmyhammer/src/cost/database/` module with:
  - `mod.rs` - Public interface and main database wrapper
  - `schema.rs` - Database schema definitions and table creation
  - `migrations.rs` - Migration system for schema versioning
  - `queries.rs` - Query functions for analytics and reporting

**Phase 2: Integration**  
- Extend existing `CostTracker` to optionally write to database
- Add database configuration to config system
- Implement sync between markdown storage and database
- Add graceful fallback when database is unavailable

**Phase 3: Analytics**
- Implement advanced query functions for cost analysis
- Add cross-issue aggregation capabilities
- Create export functions for external analytics

### 2. Technical Design

**Database Schema Mapping:**
- `cost_sessions` table maps to existing `CostSession` struct
- `api_calls` table maps to existing `ApiCall` struct  
- Use existing ULID-based identifiers for primary keys
- Leverage existing chrono DateTime types for timestamps

**Configuration Extension:**
```yaml
cost_tracking:
  database:
    enabled: false          # Optional by default
    file_path: "./costs.db" 
    connection_timeout_seconds: 30
    max_connections: 10
    retention_days: 365
```

**Core Database Wrapper:**
```rust
pub struct CostDatabase {
    pool: Option<SqlitePool>,
    config: DatabaseConfig,
}

impl CostDatabase {
    pub async fn new(config: DatabaseConfig) -> Result<Self>;
    pub async fn store_session(&self, session: &CostSession) -> Result<()>;
    pub async fn store_api_call(&self, session_id: &CostSessionId, call: &ApiCall) -> Result<()>;
    pub async fn query_cost_trends(&self, params: TrendQuery) -> Result<Vec<CostTrend>>;
}
```

**Integration with Existing CostTracker:**
- Add optional database field to `CostTracker` 
- Modify session/call completion methods to write to database
- Maintain backward compatibility with existing in-memory tracking
- Handle database failures gracefully without affecting core functionality

### 3. Testing Strategy

**Database Tests:**
- Schema creation and migration tests
- CRUD operations validation  
- Connection pooling and timeout handling
- Concurrent access testing

**Integration Tests:**
- Test database sync with existing markdown storage
- Verify graceful fallback when database disabled/unavailable
- Validate data consistency between storage methods

**Performance Tests:**
- Query performance with large datasets
- Memory usage validation
- Connection pool efficiency

### 4. Implementation Plan

1. **Add Dependencies**: Add `sqlx` to workspace dependencies
2. **Create Database Module**: Implement core database functionality
3. **Add Configuration**: Extend config system for database settings  
4. **Integrate with CostTracker**: Add optional database writes
5. **Implement Analytics**: Add query functions for advanced analysis
6. **Write Tests**: Comprehensive test coverage
7. **Add Documentation**: Usage examples and configuration guide

This approach leverages the existing robust cost tracking foundation while adding optional database capabilities for enhanced analytics, maintaining the primary markdown-based storage for simplicity.