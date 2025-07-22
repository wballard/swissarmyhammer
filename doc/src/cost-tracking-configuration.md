# Cost Tracking Configuration Reference

This document provides comprehensive reference for all cost tracking configuration options in SwissArmyHammer.

## Configuration Overview

Cost tracking configuration is specified in the `cost_tracking` section of your `swissarmyhammer.yaml` file:

```yaml
cost_tracking:
  enabled: boolean
  pricing_model: "paid" | "max"
  rates: CostRates
  session_management: SessionConfig
  aggregation: AggregationConfig
  reporting: ReportingConfig
  database: DatabaseConfig
```

## Complete Schema Reference

### Root Configuration

```yaml
cost_tracking:
  # Core Settings
  enabled: true                           # Enable/disable cost tracking [default: false]
  pricing_model: "paid"                   # "paid" or "max" [required if enabled]
  
  # Pricing Configuration (required for "paid" model)
  rates:
    input_token_cost: 0.000015            # Cost per input token in USD [required for "paid"]
    output_token_cost: 0.000075           # Cost per output token in USD [required for "paid"]
  
  # Session Management
  session_management:
    max_concurrent_sessions: 100          # Maximum parallel sessions [default: 100]
    session_timeout_hours: 24             # Session auto-cleanup timeout [default: 24]
    cleanup_interval_hours: 6             # Cleanup frequency [default: 6]
    max_api_calls_per_session: 500        # API call limit per session [default: 500]
  
  # Aggregation Settings
  aggregation:
    enabled: true                         # Enable cross-issue analytics [default: false]
    retention_days: 90                    # How long to keep aggregation data [default: 90]
    max_stored_sessions: 10000            # Maximum sessions to store [default: 10000]
  
  # Reporting Configuration
  reporting:
    include_in_issues: true               # Add cost reports to issue markdown [default: true]
    detailed_breakdown: true              # Include per-call details [default: true]
    cost_precision_decimals: 4            # Number of decimal places [default: 4]
    currency_locale: "en-US"              # Formatting locale [default: "en-US"]
    include_performance_metrics: true     # Include timing and efficiency data [default: true]
    table_max_endpoint_length: 30         # Max endpoint name length in tables [default: 30]
  
  # Optional Database Storage
  database:
    enabled: false                        # Enable SQLite backend [default: false]
    file_path: "./costs.db"               # Database file path [default: "./costs.db"]
    connection_timeout_seconds: 30        # Connection timeout [default: 30]
    max_connections: 10                   # Connection pool size [default: 10]
    retention_days: 365                   # Database retention period [default: 365]
```

## Configuration Sections

### Core Settings

#### `enabled` (boolean, default: `false`)

Controls whether cost tracking is active. When disabled, no cost data is collected or stored.

```yaml
cost_tracking:
  enabled: true  # Turn on cost tracking
```

#### `pricing_model` (string, required if enabled)

Specifies the pricing model to use:

- **`"paid"`**: Use actual token costs for billing calculation
- **`"max"`**: Unlimited plan - estimate costs for planning purposes

```yaml
cost_tracking:
  pricing_model: "paid"  # For users on paid Claude plans
```

```yaml  
cost_tracking:
  pricing_model: "max"   # For unlimited plan users
```

### Pricing Configuration (`rates`)

Required when `pricing_model` is `"paid"`. Specifies per-token costs in USD.

#### Token Cost Rates

```yaml
cost_tracking:
  rates:
    input_token_cost: 0.000015    # $0.000015 per input token
    output_token_cost: 0.000075   # $0.000075 per output token
```

**Validation Rules:**
- Both rates must be positive numbers
- Maximum value: 1.0 (prevents accidental misconfiguration)
- Precision: Up to 10 decimal places
- Units: US Dollars per token

**Current Claude Pricing Reference:**

| Model | Input Cost | Output Cost | Configuration |
|-------|------------|-------------|---------------|
| Claude Sonnet | $15/million | $75/million | `0.000015` / `0.000075` |
| Claude Opus | $75/million | $375/million | `0.000075` / `0.000375` |
| Claude Haiku | $2.50/million | $12.50/million | `0.0000025` / `0.0000125` |

### Session Management (`session_management`)

Controls how cost tracking sessions are managed in memory.

```yaml
cost_tracking:
  session_management:
    max_concurrent_sessions: 100      # Maximum simultaneous tracking sessions
    session_timeout_hours: 24         # Auto-cleanup inactive sessions after 24 hours
    cleanup_interval_hours: 6         # Run cleanup every 6 hours
    max_api_calls_per_session: 500    # Limit API calls per session (prevents memory issues)
```

**Validation:**
- `max_concurrent_sessions`: 1-10,000
- `session_timeout_hours`: 1-168 (1 week max)
- `cleanup_interval_hours`: 1-72 (3 days max)
- `max_api_calls_per_session`: 1-10,000

### Aggregation Settings (`aggregation`)

Controls cross-issue cost analysis and trend tracking.

```yaml
cost_tracking:
  aggregation:
    enabled: true                     # Enable aggregation analytics
    retention_days: 90                # Keep aggregated data for 90 days
    max_stored_sessions: 10000        # Maximum sessions in aggregation database
```

**Features when enabled:**
- Cross-issue cost summaries
- Trend analysis and pattern detection
- Project-wide usage statistics  
- Cost optimization recommendations

**Storage Impact:**
- In-memory: ~1MB per 1000 sessions
- Database: ~100KB per 1000 sessions (if database enabled)

### Reporting Configuration (`reporting`)

Controls how cost information is presented in completed issues.

```yaml
cost_tracking:
  reporting:
    include_in_issues: true               # Add cost sections to completed issues
    detailed_breakdown: true              # Show individual API calls
    cost_precision_decimals: 4            # Display $0.0000 format
    currency_locale: "en-US"              # US formatting ($1,234.56)
    include_performance_metrics: true     # Show timing and efficiency data
    table_max_endpoint_length: 30         # Truncate long endpoint names
```

#### Locale Options

Cost formatting supports multiple locales:

| Locale | Currency Format | Example |
|--------|----------------|---------|
| `"en-US"` | $1,234.56 | `$0.1234` |
| `"en-GB"` | £1,234.56 | `£0.1234` |
| `"de-DE"` | 1.234,56 € | `0,1234 €` |
| `"fr-FR"` | 1 234,56 € | `0,1234 €` |
| `"ja-JP"` | ¥1,234 | `¥0` |
| `"zh-CN"` | ¥1,234.56 | `¥0.12` |

#### Sample Report Output

With `detailed_breakdown: true`:

```markdown
## Cost Analysis

**Total Cost**: $0.34
**Total API Calls**: 4
**Total Input Tokens**: 2,100  
**Total Output Tokens**: 3,250
**Session Duration**: 3m 45s
**Completed**: 2024-01-15 14:30:25 UTC

### API Call Breakdown

| Timestamp | Endpoint | Input | Output | Duration | Cost | Status |
|-----------|----------|-------|--------|----------|------|--------|
| 14:30:20 | /v1/messages | 500 | 750 | 45s | $0.08 | ✓ |
| 14:30:35 | /v1/messages | 800 | 1,200 | 1m 20s | $0.15 | ✓ |

### Performance Metrics
- **Average cost per call**: $0.085
- **Token efficiency**: 1.55 (output/input ratio)
- **Success rate**: 100% (4/4 successful)
```

With `detailed_breakdown: false`:

```markdown
## Cost Analysis

**Total Cost**: $0.34
**Total API Calls**: 4
**Session Duration**: 3m 45s
```

### Database Configuration (`database`)

Optional SQLite backend for persistent storage and advanced analytics.

```yaml
cost_tracking:
  database:
    enabled: true                         # Enable SQLite storage
    file_path: "./costs.db"               # Database file location
    connection_timeout_seconds: 30        # Connection timeout
    max_connections: 10                   # Connection pool size
    retention_days: 365                   # How long to keep data
```

**Benefits when enabled:**
- Persistent cost data across restarts
- Advanced analytics and querying
- Historical trend analysis
- Data export capabilities
- Cross-session aggregation

**Storage Requirements:**
- ~100KB per 1000 API calls
- Automatic database cleanup based on `retention_days`
- SQLite file grows incrementally

## Environment Variable Overrides

Any configuration option can be overridden using environment variables with the `SAH_COST_` prefix:

```bash
# Core settings
export SAH_COST_TRACKING_ENABLED=true
export SAH_COST_PRICING_MODEL=paid

# Pricing
export SAH_COST_INPUT_TOKEN_COST=0.000015
export SAH_COST_OUTPUT_TOKEN_COST=0.000075

# Session management
export SAH_COST_MAX_CONCURRENT_SESSIONS=50
export SAH_COST_SESSION_TIMEOUT_HOURS=12
export SAH_COST_CLEANUP_INTERVAL_HOURS=3

# Reporting
export SAH_COST_INCLUDE_IN_ISSUES=true
export SAH_COST_DETAILED_BREAKDOWN=false
export SAH_COST_CURRENCY_LOCALE=en-GB

# Database
export SAH_COST_DATABASE_ENABLED=true
export SAH_COST_DATABASE_FILE_PATH=/data/costs.db
export SAH_COST_DATABASE_RETENTION_DAYS=180
```

**Environment Variable Priority:**
1. Environment variables (highest)
2. YAML configuration file
3. Built-in defaults (lowest)

## Configuration Examples

### Minimal Configuration

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
```

### Development Environment

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  reporting:
    include_in_issues: true
    detailed_breakdown: true
```

### Team/Production Environment

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  session_management:
    max_concurrent_sessions: 200
    session_timeout_hours: 48
  aggregation:
    enabled: true
    retention_days: 180
  database:
    enabled: true
    file_path: "/data/swissarmyhammer-costs.db"
    retention_days: 365
  reporting:
    currency_locale: "en-US"
    cost_precision_decimals: 6
```

### Unlimited Plan Setup

```yaml
cost_tracking:
  enabled: true
  pricing_model: "max"
  reporting:
    include_in_issues: true
    detailed_breakdown: false
    include_performance_metrics: true
```

## Configuration Validation

SwissArmyHammer validates all configuration at startup:

### Common Validation Errors

**Invalid pricing model:**
```
Error: Invalid pricing model 'payed'. Must be 'paid' or 'max'
```

**Missing rates for paid model:**
```
Error: Pricing rates required when pricing_model is 'paid'
```

**Invalid cost values:**
```
Error: input_token_cost must be positive and less than 1.0
```

**Invalid session limits:**
```
Error: max_concurrent_sessions must be between 1 and 10000
```

### Validation Command

Check your configuration:

```bash
swissarmyhammer doctor
```

This will validate your cost tracking configuration and report any issues.

## Configuration Tips

### Performance Optimization

- **Reduce `detailed_breakdown`** for high-volume environments
- **Increase `cleanup_interval_hours`** to reduce cleanup overhead  
- **Enable database** for better performance with aggregation
- **Limit `max_api_calls_per_session`** to prevent memory growth

### Security Considerations

- **Protect database files** with appropriate file permissions
- **Use environment variables** for sensitive configuration in production
- **Regular backups** of cost database if business-critical

### Monitoring and Maintenance

- **Monitor disk usage** if database is enabled
- **Review retention settings** periodically
- **Check cleanup effectiveness** with `swissarmyhammer doctor`

## Schema Migration

When upgrading SwissArmyHammer, configuration schemas may change. The system will automatically:

1. **Validate** new configuration format
2. **Migrate** compatible settings
3. **Warn** about deprecated options  
4. **Error** on breaking changes

Always review release notes for configuration changes.