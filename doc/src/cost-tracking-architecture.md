# Cost Tracking Architecture Guide

This guide provides a comprehensive overview of SwissArmyHammer's cost tracking system architecture, designed for developers who need to understand, maintain, or extend the implementation.

## System Overview

The cost tracking system follows a modular, layered architecture with clear separation of concerns:

```
┌─────────────────────────────────────────┐
│              User Interface             │
│    (CLI, Issue Reports, Configuration)  │
└─────────────────────────────────────────┘
                     │
┌─────────────────────────────────────────┐
│           Integration Layer             │
│      (MCP Handler, Workflow Actions)    │
└─────────────────────────────────────────┘
                     │
┌─────────────────────────────────────────┐
│             Core Services               │
│  (CostTracker, Calculator, Formatter)   │
└─────────────────────────────────────────┘
                     │
┌─────────────────────────────────────────┐
│            Data Layer                   │
│    (TokenCounter, Database, Storage)    │
└─────────────────────────────────────────┘
```

### Design Principles

- **Separation of Concerns**: Each component has a single, well-defined responsibility
- **Type Safety**: Extensive use of Rust's type system with custom wrapper types
- **Optional Integration**: Cost tracking can be completely disabled without affecting core functionality
- **Performance First**: Memory pooling, async operations, and efficient data structures
- **Fault Tolerance**: Graceful degradation when API responses don't contain expected data

## Core Components

### 1. Cost Tracker (`swissarmyhammer/src/cost/tracker.rs`)

The central orchestrator managing cost tracking sessions and API call recording.

#### Key Types

```rust
pub struct CostTracker {
    sessions: Arc<RwLock<HashMap<CostSessionId, CostSession>>>,
    calculator: Arc<CostCalculator>,
    config: CostTrackingConfig,
    cleanup_handle: Option<JoinHandle<()>>,
}

pub struct CostSession {
    pub id: CostSessionId,
    pub issue_id: IssueId,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: SessionStatus,
    pub api_calls: Vec<ApiCall>,
    pub total_input_tokens: u32,
    pub total_output_tokens: u32,
}

pub struct ApiCall {
    pub id: ApiCallId,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub endpoint: String,
    pub model: String,
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub status: ApiCallStatus,
    pub error_message: Option<String>,
}
```

#### Responsibilities

- **Session Management**: Create, track, and cleanup cost tracking sessions
- **API Call Recording**: Record all Claude API interactions with timing and token data
- **Memory Management**: Automatic cleanup of old sessions to prevent memory leaks
- **Concurrency**: Thread-safe operations with multiple simultaneous tracking sessions

#### Key Methods

```rust
impl CostTracker {
    // Session lifecycle
    pub async fn start_session(&self, issue_id: IssueId) -> Result<CostSessionId>;
    pub async fn end_session(&self, session_id: CostSessionId) -> Result<CostSession>;
    
    // API call recording
    pub async fn record_api_call(&self, session_id: CostSessionId, call: ApiCallRecord) -> Result<ApiCallId>;
    pub async fn update_api_call_completion(&self, session_id: CostSessionId, call_id: ApiCallId, tokens: TokenUsage) -> Result<()>;
    
    // Querying and analytics
    pub async fn get_session(&self, session_id: CostSessionId) -> Result<Option<CostSession>>;
    pub async fn get_active_sessions(&self) -> Vec<CostSessionId>;
}
```

### 2. Cost Calculator (`swissarmyhammer/src/cost/calculator.rs`)

Handles all cost calculations with support for different pricing models and precise decimal arithmetic.

#### Key Types

```rust
pub struct CostCalculator {
    pricing_model: PricingModel,
    rates: Option<PricingRates>,
}

pub enum PricingModel {
    Paid,    // Per-token billing for paid plans
    Max,     // Unlimited plans with cost estimation
}

pub struct PricingRates {
    pub input_token_cost: Decimal,   // Cost per input token in USD
    pub output_token_cost: Decimal,  // Cost per output token in USD
}

pub struct CostCalculation {
    pub input_cost: Decimal,
    pub output_cost: Decimal,
    pub total_cost: Decimal,
    pub input_tokens: u32,
    pub output_tokens: u32,
}
```

#### Responsibilities

- **Token Cost Calculation**: Convert token counts to monetary costs
- **Model-Specific Pricing**: Support different Claude models with appropriate pricing
- **Precision Arithmetic**: Use `Decimal` type for accurate financial calculations
- **Validation**: Ensure cost calculations are within reasonable bounds

#### Pricing Models

**Paid Model**: Actual per-token costs
```rust
let calculation = calculator.calculate_tokens_cost(1000, 1500, "claude-3-sonnet")?;
// Returns actual USD cost: $0.1275 for this example
```

**Max Model**: Cost estimation for planning
```rust  
let calculation = calculator.calculate_tokens_cost(1000, 1500, "claude-3-sonnet")?;
// Returns estimated cost for budgeting purposes
```

### 3. Token Counter (`swissarmyhammer/src/cost/token_counter.rs`)

Extracts token usage information from Claude API responses with fallback mechanisms and validation.

#### Key Types

```rust
pub struct TokenCounter {
    validator: TokenValidator,
}

pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub source: TokenSource,
    pub confidence: TokenConfidence,
}

pub enum TokenSource {
    ApiResponse,      // From API response JSON
    ResponseHeaders,  // From HTTP headers
    Estimated,       // Fallback estimation
}

pub enum TokenConfidence {
    Exact,    // 100% accurate from API
    High,     // Close validation match
    Medium,   // Reasonable estimation
    Low,      // Fallback estimation
}
```

#### Response Parsing Strategy

The token counter uses multiple extraction strategies with fallbacks:

1. **Primary JSON Fields**: `usage.input_tokens`, `usage.output_tokens`
2. **Alternative Fields**: `input_token_count`, `output_token_count`
3. **HTTP Headers**: `anthropic-input-tokens`, `anthropic-output-tokens`
4. **Estimation Fallback**: Simple character-based estimation

#### Validation Process

```rust
pub fn count_from_response(
    &self,
    response_body: &str,
    estimated_usage: Option<TokenUsage>,
    model: &str,
) -> Result<TokenUsage> {
    // 1. Try extracting from JSON response
    if let Some(tokens) = self.extract_from_json(response_body)? {
        return Ok(self.validate_against_estimate(tokens, estimated_usage));
    }
    
    // 2. Try extracting from headers
    if let Some(tokens) = self.extract_from_headers(response_headers)? {
        return Ok(self.validate_against_estimate(tokens, estimated_usage));
    }
    
    // 3. Fall back to estimation
    Ok(estimated_usage.unwrap_or_else(|| self.estimate_tokens(input_text, output_text)))
}
```

### 4. Database Layer (`swissarmyhammer/src/cost/database/`)

Optional SQLite backend providing persistent storage and advanced analytics capabilities.

#### Schema Design

```sql
-- Core session tracking
CREATE TABLE cost_sessions (
    session_id TEXT PRIMARY KEY,
    issue_id TEXT NOT NULL,
    started_at DATETIME NOT NULL,
    completed_at DATETIME,
    status TEXT NOT NULL,
    total_input_tokens INTEGER DEFAULT 0,
    total_output_tokens INTEGER DEFAULT 0,
    total_cost DECIMAL(10,6),
    metadata TEXT -- JSON for extensibility
);

-- Individual API call records
CREATE TABLE api_calls (
    call_id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL,
    started_at DATETIME NOT NULL,
    completed_at DATETIME,
    endpoint TEXT NOT NULL,
    model TEXT NOT NULL,
    input_tokens INTEGER DEFAULT 0,
    output_tokens INTEGER DEFAULT 0,
    cost DECIMAL(8,6),
    status TEXT NOT NULL,
    error_message TEXT,
    FOREIGN KEY (session_id) REFERENCES cost_sessions(session_id)
);

-- Aggregated analytics (optional)
CREATE TABLE cost_analytics (
    analysis_date DATE PRIMARY KEY,
    total_sessions INTEGER,
    total_api_calls INTEGER,
    total_input_tokens INTEGER,
    total_output_tokens INTEGER,
    total_cost DECIMAL(12,6),
    average_session_cost DECIMAL(8,6),
    metrics_json TEXT -- Extended metrics
);
```

#### Migration System

```rust
pub struct MigrationManager {
    connection: Arc<SqlitePool>,
}

impl MigrationManager {
    pub async fn run_migrations(&self) -> Result<()> {
        let current_version = self.get_schema_version().await?;
        let target_version = LATEST_SCHEMA_VERSION;
        
        for version in (current_version + 1)..=target_version {
            self.apply_migration(version).await?;
        }
        
        Ok(())
    }
    
    pub async fn rollback_migration(&self, target_version: u32) -> Result<()> {
        // Rollback implementation
    }
}
```

### 5. Integration Layer

#### MCP Protocol Integration (`swissarmyhammer/src/mcp/cost_tracking.rs`)

Wraps existing MCP handlers with cost tracking using the decorator pattern:

```rust
pub struct CostTrackingMcpHandler<T> {
    inner: T,
    cost_tracker: Arc<CostTracker>,
    enabled: bool,
}

impl<T: ToolHandlers> ToolHandlers for CostTrackingMcpHandler<T> {
    async fn handle_issue_work(&self, request: IssueWorkRequest) -> Result<IssueWorkResponse> {
        let session_id = if self.enabled {
            Some(self.cost_tracker.start_session(request.issue_id.clone()).await?)
        } else {
            None
        };
        
        // Record simulated API call for MCP operation
        if let Some(session_id) = session_id {
            let call_record = ApiCallRecord {
                endpoint: "mcp://issue_work".to_string(),
                model: "mcp-internal".to_string(),
                // Default MCP operation token estimates
                input_tokens: DEFAULT_MCP_INPUT_TOKENS,
                output_tokens: DEFAULT_MCP_OUTPUT_TOKENS,
            };
            
            self.cost_tracker.record_api_call(session_id, call_record).await?;
        }
        
        // Delegate to actual handler
        let response = self.inner.handle_issue_work(request).await?;
        
        // Complete session on successful issue completion
        if let Some(session_id) = session_id {
            if response.completed {
                let session = self.cost_tracker.end_session(session_id).await?;
                // Add cost report to issue if configured
                self.add_cost_report_to_issue(&response.issue_id, &session).await?;
            }
        }
        
        Ok(response)
    }
}
```

#### Workflow Integration

Cost tracking integrates seamlessly with the existing workflow system:

```rust
// In workflow execution
pub async fn execute_workflow_with_cost_tracking(
    workflow: &Workflow,
    cost_tracker: &CostTracker,
) -> Result<WorkflowResult> {
    let session_id = cost_tracker.start_session(workflow.issue_id.clone()).await?;
    
    let result = execute_workflow_steps(workflow).await;
    
    match result {
        Ok(result) => {
            let session = cost_tracker.end_session(session_id).await?;
            Ok(WorkflowResult::with_cost_analysis(result, session))
        }
        Err(error) => {
            // Mark session as failed but don't lose cost data
            cost_tracker.mark_session_failed(session_id, &error).await?;
            Err(error)
        }
    }
}
```

## Data Flow Architecture

### Session Lifecycle

```
Issue Start
     │
     ▼
Create Cost Session ──────────┐
     │                        │
     ▼                        │
Record API Calls ─────────────┤
     │                        │
     ▼                        │
Extract Token Counts ─────────┤
     │                        │
     ▼                        │
Calculate Costs ──────────────┤
     │                        │
     ▼                        │
Issue Complete                │
     │                        │
     ▼                        │
End Session ──────────────────┘
     │
     ▼
Generate Cost Report
     │
     ▼
Add to Issue Markdown
```

### Token Counting Flow

```
API Response
     │
     ▼
JSON Extraction ──→ Success? ──→ Validate ──→ Return TokenUsage
     │                 │
     ▼                 ▼
Header Extraction    Failure
     │                 │
     ▼                 ▼
Success? ──→ Validate  Estimate Tokens
     │          │           │
     ▼          ▼           ▼
Failure    Return      Return TokenUsage
     │     TokenUsage    (Low Confidence)
     ▼
Estimate Tokens
     │
     ▼
Return TokenUsage
(Low Confidence)
```

### Cost Calculation Pipeline

```
Token Usage + Pricing Model
           │
           ▼
Model Recognition ──→ Claude Sonnet/Opus/Haiku
           │              │
           ▼              ▼
Apply Pricing Rules ──→ Input: $X per token
           │              Output: $Y per token
           ▼              │
Decimal Arithmetic ◄──────┘
           │
           ▼
Cost Validation ──→ Reasonable bounds check
           │
           ▼
Return CostCalculation
```

## Performance Considerations

### Memory Management

- **Session Cleanup**: Automatic cleanup of completed sessions after configurable retention period
- **Token Counting Cache**: LRU cache for token estimation results  
- **Database Connection Pooling**: Efficient SQLite connection management
- **Bounded Collections**: Limits on API calls per session and total concurrent sessions

### Async Operations

All I/O operations are async to prevent blocking:

```rust
// Non-blocking session operations
pub async fn record_api_call(&self, session_id: CostSessionId, call: ApiCallRecord) -> Result<ApiCallId>;

// Non-blocking database operations  
pub async fn store_session(&self, session: &CostSession) -> Result<()>;

// Non-blocking cleanup operations
async fn cleanup_old_sessions(&self) -> Result<u32>;
```

### Database Optimization

- **Indexes**: Strategic indexes on frequently queried columns
- **Batch Operations**: Bulk inserts for better performance
- **Connection Pooling**: Reuse connections to reduce overhead
- **Prepared Statements**: Compiled queries for repeated operations

```sql
-- Performance indexes
CREATE INDEX idx_sessions_issue_id ON cost_sessions(issue_id);
CREATE INDEX idx_sessions_started_at ON cost_sessions(started_at);
CREATE INDEX idx_api_calls_session_id ON api_calls(session_id);
CREATE INDEX idx_api_calls_started_at ON api_calls(started_at);
```

## Error Handling Strategy

### Graceful Degradation

Cost tracking is designed to never interfere with core SwissArmyHammer functionality:

```rust
pub async fn record_api_call_safe(
    &self, 
    session_id: CostSessionId, 
    call: ApiCallRecord
) -> ApiCallId {
    match self.record_api_call(session_id, call).await {
        Ok(call_id) => call_id,
        Err(error) => {
            // Log error but don't fail the operation
            tracing::warn!("Failed to record API call: {}", error);
            ApiCallId::placeholder()
        }
    }
}
```

### Error Categories

1. **Configuration Errors**: Invalid YAML, missing required fields
2. **Runtime Errors**: Database connection failures, memory limits
3. **Data Errors**: Malformed API responses, invalid token counts  
4. **Integration Errors**: MCP protocol issues, workflow integration failures

Each category has appropriate handling strategies:

```rust
match error {
    CostTrackingError::Configuration(config_error) => {
        // Fail fast - configuration must be valid
        return Err(config_error.into());
    }
    CostTrackingError::Runtime(runtime_error) => {
        // Log and continue with degraded functionality
        tracing::error!("Cost tracking runtime error: {}", runtime_error);
        disable_cost_tracking();
    }
    CostTrackingError::Data(data_error) => {
        // Use fallback data and continue
        tracing::warn!("Using estimated data: {}", data_error);
        use_estimated_tokens();
    }
}
```

## Extension Points

### Custom Storage Backends

Implement the `CostStorage` trait to add new storage options:

```rust
#[async_trait]
pub trait CostStorage: Send + Sync {
    async fn store_session(&self, session: &CostSession) -> Result<()>;
    async fn load_session(&self, session_id: &CostSessionId) -> Result<Option<CostSession>>;
    async fn list_sessions(&self, issue_id: Option<&IssueId>) -> Result<Vec<CostSessionId>>;
    async fn cleanup_old_sessions(&self, older_than: DateTime<Utc>) -> Result<u32>;
}

// Example custom implementation
pub struct RemoteStorage {
    client: reqwest::Client,
    endpoint: Url,
}

#[async_trait] 
impl CostStorage for RemoteStorage {
    async fn store_session(&self, session: &CostSession) -> Result<()> {
        let response = self.client
            .post(&format!("{}/sessions", self.endpoint))
            .json(session)
            .send()
            .await?;
            
        if response.status().is_success() {
            Ok(())
        } else {
            Err(CostTrackingError::Storage("Failed to store session".into()))
        }
    }
}
```

### Custom Cost Models

Extend pricing calculations:

```rust
pub trait CostModel: Send + Sync {
    fn calculate_cost(&self, tokens: &TokenUsage, model: &str) -> Result<CostCalculation>;
    fn supports_model(&self, model: &str) -> bool;
}

pub struct CustomEnterpriseModel {
    base_cost: Decimal,
    volume_discounts: HashMap<u32, Decimal>,
}

impl CostModel for CustomEnterpriseModel {
    fn calculate_cost(&self, tokens: &TokenUsage, model: &str) -> Result<CostCalculation> {
        let base_cost = self.base_cost * Decimal::from(tokens.input_tokens + tokens.output_tokens);
        let discount = self.volume_discounts
            .iter()
            .find(|(threshold, _)| tokens.input_tokens + tokens.output_tokens >= **threshold)
            .map(|(_, discount)| *discount)
            .unwrap_or(Decimal::ZERO);
            
        Ok(CostCalculation {
            total_cost: base_cost * (Decimal::ONE - discount),
            // ... other fields
        })
    }
}
```

### Custom Formatters

Add new output formats:

```rust
pub trait CostFormatter: Send + Sync {
    fn format_session(&self, session: &CostSession) -> Result<String>;
    fn format_summary(&self, sessions: &[CostSession]) -> Result<String>;
}

pub struct JsonFormatter;

impl CostFormatter for JsonFormatter {
    fn format_session(&self, session: &CostSession) -> Result<String> {
        let json_data = json!({
            "session_id": session.id,
            "total_cost": session.calculate_total_cost()?,
            "api_calls": session.api_calls.len(),
            "duration": session.duration(),
        });
        
        Ok(serde_json::to_string_pretty(&json_data)?)
    }
}
```

## Testing Architecture

### Test Structure

```
swissarmyhammer/src/cost/
├── tests/
│   ├── unit_tests/           # Individual component tests
│   ├── integration_tests/    # Full system tests
│   ├── property_tests/       # Randomized testing
│   ├── performance_tests/    # Benchmarks and load tests
│   └── chaos_tests/          # Failure injection tests
```

### Testing Utilities

```rust
pub struct CostTrackingTestHarness {
    tracker: CostTracker,
    mock_calculator: MockCostCalculator,
    temp_db: TempDatabase,
}

impl CostTrackingTestHarness {
    pub fn new() -> Self {
        // Set up isolated test environment
    }
    
    pub async fn create_test_session(&self, issue_id: IssueId) -> CostSessionId {
        // Helper for test session creation
    }
    
    pub fn generate_api_calls(&self, count: usize) -> Vec<ApiCallRecord> {
        // Generate realistic test data
    }
}
```

This architecture ensures the cost tracking system is robust, performant, and maintainable while providing clear extension points for future enhancements.