# Cost Tracking Examples and Tutorials

This guide provides complete, working examples for different cost tracking scenarios in SwissArmyHammer.

## Table of Contents

- [Basic Setup Examples](#basic-setup-examples)
- [Advanced Configuration Examples](#advanced-configuration-examples)  
- [Custom Reporting Examples](#custom-reporting-examples)
- [Integration Examples](#integration-examples)
- [Production Configuration Examples](#production-configuration-examples)
- [Environment-Specific Examples](#environment-specific-examples)

## Basic Setup Examples

### Example 1: Individual Developer Setup

**Scenario**: Solo developer wanting to track costs for personal Claude usage.

**Configuration** (`swissarmyhammer.yaml`):
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015   # Claude Sonnet rates
    output_token_cost: 0.000075
  reporting:
    include_in_issues: true
    detailed_breakdown: true
    cost_precision_decimals: 4
```

**Usage**:
```bash
# Work on an issue - cost tracking happens automatically
swissarmyhammer issue work feature-implementation.md

# Check cost tracking status
swissarmyhammer doctor
```

**Expected Output** (in completed issue):
```markdown
## Cost Analysis

**Total Cost**: $0.12
**Total API Calls**: 2
**Total Input Tokens**: 950
**Total Output Tokens**: 1,240  
**Session Duration**: 1m 45s

### API Call Breakdown
| Timestamp | Endpoint | Input | Output | Cost | Status |
|-----------|----------|-------|--------|------|--------|
| 10:30:20 | /v1/messages | 450 | 580 | $0.05 | ✓ |
| 10:30:45 | /v1/messages | 500 | 660 | $0.07 | ✓ |
```

### Example 2: Unlimited Plan with Cost Estimation

**Scenario**: Developer with unlimited Claude access who wants to understand usage patterns.

**Configuration**:
```yaml
cost_tracking:
  enabled: true
  pricing_model: "max"  # No actual costs, just estimation
  reporting:
    include_in_issues: true
    detailed_breakdown: false  # Simpler reports
    include_performance_metrics: true
```

**Expected Output**:
```markdown
## Cost Analysis (Estimated)

**Estimated Cost**: $0.12 (if on paid plan)
**Total API Calls**: 2
**Total Tokens**: 2,190 (950 input, 1,240 output)
**Token Efficiency**: 1.31 (output/input ratio)
**Session Duration**: 1m 45s
```

### Example 3: Minimal Cost Tracking

**Scenario**: Basic cost tracking with minimal configuration and reporting.

**Configuration**:
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  reporting:
    include_in_issues: true
    detailed_breakdown: false
    cost_precision_decimals: 2
```

**Expected Output**:
```markdown  
## Cost Analysis

**Total Cost**: $0.12
**API Calls**: 2
**Duration**: 1m 45s
```

## Advanced Configuration Examples

### Example 4: Team Environment with Database

**Scenario**: Development team sharing cost data and needing historical analytics.

**Configuration**:
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # Enhanced session management for team use
  session_management:
    max_concurrent_sessions: 200     # Higher limit for team
    session_timeout_hours: 48        # Longer timeout
    cleanup_interval_hours: 4        # More frequent cleanup
  
  # Enable persistent storage
  database:
    enabled: true
    file_path: "/shared/swissarmyhammer-costs.db"
    retention_days: 365              # Full year of data
    connection_timeout_seconds: 60   # Higher timeout
    max_connections: 20              # More connections
  
  # Enable cross-issue analytics
  aggregation:
    enabled: true
    retention_days: 180
    max_stored_sessions: 50000       # Higher limits
  
  # Comprehensive reporting
  reporting:
    include_in_issues: true
    detailed_breakdown: true
    currency_locale: "en-US"
    include_performance_metrics: true
    cost_precision_decimals: 6
```

**Setup Commands**:
```bash
# Create shared directory
mkdir -p /shared
chmod 755 /shared

# Initialize database
swissarmyhammer doctor

# Verify team setup
swissarmyhammer doctor --verbose
```

### Example 5: High-Volume Production Environment

**Scenario**: Production system processing many issues with optimized performance settings.

**Configuration**:
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # Optimized for high throughput
  session_management:
    max_concurrent_sessions: 500
    session_timeout_hours: 12        # Shorter timeout
    cleanup_interval_hours: 2        # More aggressive cleanup
    max_api_calls_per_session: 1000  # Higher call limits
  
  # High-performance database settings
  database:
    enabled: true
    file_path: "/data/costs.db"
    connection_timeout_seconds: 30
    max_connections: 50              # Large connection pool
    retention_days: 90               # Shorter retention for performance
  
  # Lightweight reporting
  reporting:
    include_in_issues: true
    detailed_breakdown: false        # Reduced overhead
    include_performance_metrics: false
    cost_precision_decimals: 4
  
  # Disabled aggregation for performance
  aggregation:
    enabled: false
```

**Monitoring Script** (`monitor-costs.sh`):
```bash
#!/bin/bash

# Monitor cost tracking performance
echo "=== Cost Tracking Status ==="
swissarmyhammer doctor | grep -A 10 "Cost tracking"

echo "=== Database Size ==="
ls -lh /data/costs.db

echo "=== Memory Usage ==="
ps aux | grep swissarmyhammer | awk '{print $4, $6}'

echo "=== Active Sessions ==="
sqlite3 /data/costs.db "SELECT COUNT(*) FROM cost_sessions WHERE completed_at IS NULL;"
```

## Custom Reporting Examples

### Example 6: CSV Export Configuration

**Scenario**: Export cost data to CSV for external analysis.

**Configuration**:
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  database:
    enabled: true
    file_path: "./costs.db"
  reporting:
    include_in_issues: false  # Disable markdown reports
    export_csv: true          # Custom extension point
    csv_file_path: "./cost-exports/"
```

**Export Script** (`export-costs.py`):
```python
#!/usr/bin/env python3
import sqlite3
import csv
import sys
from datetime import datetime

def export_costs_csv(db_path, output_dir):
    """Export cost data to CSV files"""
    conn = sqlite3.connect(db_path)
    conn.row_factory = sqlite3.Row
    
    # Export sessions
    cursor = conn.execute("""
        SELECT session_id, issue_id, started_at, completed_at,
               total_input_tokens, total_output_tokens, total_cost
        FROM cost_sessions 
        WHERE completed_at IS NOT NULL
        ORDER BY started_at DESC
    """)
    
    with open(f"{output_dir}/sessions.csv", 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['Session ID', 'Issue ID', 'Started', 'Completed', 
                        'Input Tokens', 'Output Tokens', 'Total Cost'])
        
        for row in cursor:
            writer.writerow(row)
    
    # Export API calls
    cursor = conn.execute("""
        SELECT call_id, session_id, started_at, endpoint, model,
               input_tokens, output_tokens, status
        FROM api_calls
        ORDER BY started_at DESC
    """)
    
    with open(f"{output_dir}/api_calls.csv", 'w', newline='') as f:
        writer = csv.writer(f)
        writer.writerow(['Call ID', 'Session ID', 'Started', 'Endpoint', 
                        'Model', 'Input Tokens', 'Output Tokens', 'Status'])
        
        for row in cursor:
            writer.writerow(row)
    
    conn.close()
    print(f"Cost data exported to {output_dir}/")

if __name__ == "__main__":
    export_costs_csv("./costs.db", "./cost-exports")
```

### Example 7: Custom Markdown Template

**Scenario**: Customize the cost report format in issue markdown.

**Custom Template** (`cost-report-template.md.liquid`):
```liquid
## 💰 Cost Analysis

{% if session.pricing_model == "paid" -%}
**💵 Total Cost**: ${{ session.total_cost | money_format }}
{% else -%}
**📊 Estimated Cost**: ${{ session.total_cost | money_format }} (hypothetical)
{% endif -%}

**📞 API Calls**: {{ session.api_calls | size }}
**📥 Input Tokens**: {{ session.total_input_tokens | number_format }}
**📤 Output Tokens**: {{ session.total_output_tokens | number_format }}
**⏱️ Duration**: {{ session.duration | duration_format }}

{% if session.api_calls | size > 0 -%}
### 📋 Performance Summary

- **Average Cost per Call**: ${{ session.average_cost_per_call | money_format }}
- **Token Efficiency**: {{ session.token_efficiency | round: 2 }}x (output/input ratio)
- **Success Rate**: {{ session.success_rate | percentage }}
- **Average Response Time**: {{ session.average_response_time | duration_format }}

{% if detailed_breakdown -%}
### 🔍 Detailed Breakdown

| Time | Endpoint | Model | Input | Output | Duration | Cost | Status |
|------|----------|-------|-------|--------|----------|------|--------|
{% for call in session.api_calls -%}
| {{ call.started_at | time_format }} | {{ call.endpoint | truncate: 20 }} | {{ call.model }} | {{ call.input_tokens }} | {{ call.output_tokens }} | {{ call.duration | duration_format }} | ${{ call.cost | money_format }} | {{ call.status | status_icon }} |
{% endfor -%}
{% endif -%}

### 💡 Cost Optimization Tips

{% if session.token_efficiency < 1.2 -%}
- Consider optimizing prompts to generate more focused responses
{% endif -%}
{% if session.average_response_time > 5 -%}
- API calls are taking longer than expected - check prompt complexity
{% endif -%}
{% if session.api_calls | size > 10 -%}
- High number of API calls - consider batching operations
{% endif -%}

{% endif -%}
```

### Example 8: Slack Integration

**Scenario**: Send cost alerts to Slack when thresholds are exceeded.

**Configuration**:
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # Custom webhook configuration (extension point)
  notifications:
    slack_webhook_url: "https://hooks.slack.com/services/YOUR/WEBHOOK/URL"
    cost_threshold: 1.00  # Alert when session exceeds $1.00
    daily_digest: true    # Send daily cost summary
```

**Slack Notification Script** (`slack-notify.py`):
```python
#!/usr/bin/env python3
import json
import requests
from datetime import datetime

def send_cost_alert(webhook_url, session_data):
    """Send cost alert to Slack"""
    
    color = "danger" if session_data['total_cost'] > 1.00 else "good"
    
    payload = {
        "attachments": [{
            "color": color,
            "title": f"Cost Alert: Issue {session_data['issue_id']}",
            "fields": [
                {
                    "title": "Total Cost",
                    "value": f"${session_data['total_cost']:.4f}",
                    "short": True
                },
                {
                    "title": "API Calls", 
                    "value": str(session_data['api_calls']),
                    "short": True
                },
                {
                    "title": "Duration",
                    "value": session_data['duration'],
                    "short": True
                },
                {
                    "title": "Tokens Used",
                    "value": f"{session_data['total_tokens']:,}",
                    "short": True
                }
            ],
            "footer": "SwissArmyHammer Cost Tracking",
            "ts": int(datetime.now().timestamp())
        }]
    }
    
    response = requests.post(webhook_url, json=payload)
    response.raise_for_status()
    
    print(f"Slack notification sent for {session_data['issue_id']}")
```

## Integration Examples

### Example 9: CI/CD Pipeline Integration

**Scenario**: Integrate cost tracking into GitHub Actions workflow.

**GitHub Actions Workflow** (`.github/workflows/issue-processing.yml`):
```yaml
name: Process Issues with Cost Tracking

on:
  workflow_dispatch:
    inputs:
      issue_file:
        description: 'Issue file to process'
        required: true
        type: string

jobs:
  process-issue:
    runs-on: ubuntu-latest
    
    steps:
    - name: Checkout repository
      uses: actions/checkout@v3
      
    - name: Install SwissArmyHammer
      run: |
        curl -sSL https://raw.githubusercontent.com/wballard/swissarmyhammer/main/install.sh | sh
        echo "$HOME/.local/bin" >> $GITHUB_PATH
    
    - name: Configure Cost Tracking
      env:
        CLAUDE_API_KEY: ${{ secrets.CLAUDE_API_KEY }}
        SAH_COST_TRACKING_ENABLED: true
        SAH_COST_PRICING_MODEL: paid
        SAH_COST_INPUT_TOKEN_COST: 0.000015
        SAH_COST_OUTPUT_TOKEN_COST: 0.000075
        SAH_COST_DATABASE_ENABLED: true
        SAH_COST_DATABASE_FILE_PATH: ./github-actions-costs.db
      run: |
        swissarmyhammer doctor
    
    - name: Process Issue
      env:
        CLAUDE_API_KEY: ${{ secrets.CLAUDE_API_KEY }}
      run: |
        swissarmyhammer issue work "${{ github.event.inputs.issue_file }}"
    
    - name: Extract Cost Data
      run: |
        python3 scripts/extract-costs.py ./github-actions-costs.db >> $GITHUB_STEP_SUMMARY
    
    - name: Upload Cost Database
      uses: actions/upload-artifact@v3
      with:
        name: cost-tracking-data
        path: github-actions-costs.db
        
    - name: Cost Summary Comment
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v6
      with:
        script: |
          const fs = require('fs');
          const costData = fs.readFileSync('cost-summary.md', 'utf8');
          
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: costData
          });
```

### Example 10: Docker Environment

**Scenario**: Run SwissArmyHammer with cost tracking in Docker container.

**Dockerfile**:
```dockerfile
FROM rust:1.75-slim as builder

# Install SwissArmyHammer
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    sqlite3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/swissarmyhammer /usr/local/bin/

# Create data directory
RUN mkdir -p /data && chmod 755 /data

# Copy default configuration
COPY docker/swissarmyhammer.yaml /etc/swissarmyhammer/

WORKDIR /workspace
VOLUME ["/workspace", "/data"]

ENTRYPOINT ["swissarmyhammer"]
```

**Docker Compose** (`docker-compose.yml`):
```yaml
version: '3.8'

services:
  swissarmyhammer:
    build: .
    environment:
      - CLAUDE_API_KEY=${CLAUDE_API_KEY}
      - SAH_COST_TRACKING_ENABLED=true
      - SAH_COST_PRICING_MODEL=paid  
      - SAH_COST_INPUT_TOKEN_COST=0.000015
      - SAH_COST_OUTPUT_TOKEN_COST=0.000075
      - SAH_COST_DATABASE_ENABLED=true
      - SAH_COST_DATABASE_FILE_PATH=/data/costs.db
    volumes:
      - ./workspace:/workspace
      - cost_data:/data
    command: ["serve", "--host", "0.0.0.0", "--port", "8080"]
    ports:
      - "8080:8080"

volumes:
  cost_data:
    driver: local
```

**Usage**:
```bash
# Start container with cost tracking
docker-compose up -d

# Process issue via container
docker-compose exec swissarmyhammer issue work issue-file.md

# View cost database
docker-compose exec swissarmyhammer sqlite3 /data/costs.db "SELECT * FROM cost_sessions;"
```

## Production Configuration Examples

### Example 11: Multi-Environment Setup

**Scenario**: Different configurations for development, staging, and production environments.

**Development** (`config/development.yaml`):
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
    cost_precision_decimals: 6  # High precision for development
  aggregation:
    enabled: true  # Full analytics in development
```

**Staging** (`config/staging.yaml`):
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  database:
    enabled: true
    file_path: "/data/staging-costs.db"
    retention_days: 30  # Shorter retention
  reporting:
    include_in_issues: true
    detailed_breakdown: false  # Lighter reports
    cost_precision_decimals: 4
  session_management:
    cleanup_interval_hours: 2  # More aggressive cleanup
```

**Production** (`config/production.yaml`):
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # High-performance settings
  session_management:
    max_concurrent_sessions: 1000
    session_timeout_hours: 6
    cleanup_interval_hours: 1
    max_api_calls_per_session: 2000
  
  # Production database
  database:
    enabled: true
    file_path: "/data/production-costs.db" 
    connection_timeout_seconds: 120
    max_connections: 100
    retention_days: 365
  
  # Minimal reporting for performance
  reporting:
    include_in_issues: false  # Disable for performance
    detailed_breakdown: false
  
  # Disabled aggregation
  aggregation:
    enabled: false
```

**Environment Script** (`scripts/set-environment.sh`):
```bash
#!/bin/bash

ENV=${1:-development}

case $ENV in
  development)
    export SAH_CONFIG_FILE="config/development.yaml"
    export SAH_COST_DATABASE_FILE_PATH="./dev-costs.db"
    ;;
  staging) 
    export SAH_CONFIG_FILE="config/staging.yaml"
    export SAH_COST_DATABASE_FILE_PATH="/data/staging-costs.db"
    ;;
  production)
    export SAH_CONFIG_FILE="config/production.yaml"
    export SAH_COST_DATABASE_FILE_PATH="/data/production-costs.db"
    ;;
  *)
    echo "Invalid environment: $ENV"
    exit 1
    ;;
esac

echo "Environment set to: $ENV"
echo "Config file: $SAH_CONFIG_FILE"
swissarmyhammer doctor
```

### Example 12: Load Balancer Configuration

**Scenario**: Multiple SwissArmyHammer instances sharing cost tracking data.

**Shared Configuration** (`config/shared.yaml`):
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # Shared database backend  
  database:
    enabled: true
    file_path: "/shared/nfs/costs.db"  # Network shared storage
    connection_timeout_seconds: 180   # Higher timeout for network
    max_connections: 200              # Support multiple instances
    retention_days: 365
  
  # Instance-specific session management
  session_management:
    max_concurrent_sessions: 200      # Per instance
    session_timeout_hours: 24
    cleanup_interval_hours: 6
  
  # Lightweight reporting
  reporting:
    include_in_issues: true
    detailed_breakdown: false
    cost_precision_decimals: 4
    
  # Shared aggregation
  aggregation:
    enabled: true
    retention_days: 90
    max_stored_sessions: 100000       # Higher limit for shared data
```

**Load Balancer Setup** (`nginx.conf`):
```nginx
upstream swissarmyhammer {
    server 10.0.1.10:8080 weight=1;
    server 10.0.1.11:8080 weight=1;
    server 10.0.1.12:8080 weight=1;
}

server {
    listen 80;
    server_name swissarmyhammer.internal;
    
    location / {
        proxy_pass http://swissarmyhammer;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        
        # Ensure session affinity for cost tracking
        sticky_cookie_path /;
    }
}
```

## Environment-Specific Examples

### Example 13: Local Development with Hot Reload

**Configuration** (`swissarmyhammer-dev.yaml`):
```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # Development-friendly settings
  session_management:
    max_concurrent_sessions: 10       # Lower for local development
    session_timeout_hours: 1          # Quick cleanup
    cleanup_interval_hours: 0.25      # Clean up every 15 minutes
  
  # Local database
  database:
    enabled: true
    file_path: "./dev-costs.db"
    retention_days: 7                 # Short retention for development
  
  # Detailed reporting for debugging
  reporting:
    include_in_issues: true
    detailed_breakdown: true
    include_performance_metrics: true
    cost_precision_decimals: 8        # Maximum precision for analysis
  
  # Full aggregation for testing
  aggregation:
    enabled: true
    retention_days: 7
    max_stored_sessions: 1000
```

**Development Script** (`dev.sh`):
```bash
#!/bin/bash

# Development environment setup
export RUST_LOG=debug
export SAH_CONFIG_FILE="swissarmyhammer-dev.yaml"

# Clean up previous session
rm -f dev-costs.db

echo "Starting SwissArmyHammer in development mode..."
echo "Cost tracking enabled with detailed logging"

# Watch for configuration changes
fswatch -o swissarmyhammer-dev.yaml | while read f; do
  echo "Configuration changed, restarting..."
  pkill swissarmyhammer
  swissarmyhammer serve --reload &
done &

# Start with hot reload
swissarmyhammer serve --reload --config swissarmyhammer-dev.yaml
```

This comprehensive examples guide provides practical, tested configurations for every major use case of SwissArmyHammer's cost tracking system.