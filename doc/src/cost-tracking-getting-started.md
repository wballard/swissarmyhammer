# Getting Started with Cost Tracking

This guide will help you set up cost tracking for your SwissArmyHammer installation in just a few minutes.

## Prerequisites

Before you begin, ensure you have:
- SwissArmyHammer installed and working with issues
- A `swissarmyhammer.yaml` configuration file (or create one)
- Basic familiarity with YAML configuration

## Quick Setup (5 minutes)

### Step 1: Enable Cost Tracking

Add the following to your `swissarmyhammer.yaml` file:

```yaml
# Basic cost tracking configuration
cost_tracking:
  enabled: true
  pricing_model: "paid"  # or "max" for unlimited plans
  rates:
    input_token_cost: 0.000015   # Claude Sonnet rates as example
    output_token_cost: 0.000075
  reporting:
    include_in_issues: true      # Add cost reports to completed issues
```

### Step 2: Verify Configuration

Test your configuration:

```bash
swissarmyhammer doctor
```

You should see confirmation that cost tracking is enabled and properly configured.

### Step 3: Work on an Issue  

Start working on any issue:

```bash
swissarmyhammer issue work your-issue.md
```

Cost tracking will automatically begin when you start working and complete when the issue finishes.

### Step 4: View Cost Report

When your issue completes, check the bottom of the completed issue file for a cost analysis section like this:

```markdown
## Cost Analysis

**Total Cost**: $0.12
**Total API Calls**: 2
**Total Input Tokens**: 950
**Total Output Tokens**: 1,240
**Session Duration**: 1m 30s

### API Call Breakdown
| Timestamp | Endpoint | Input | Output | Cost | Status |
|-----------|----------|-------|--------|------|--------|
| 14:30:20 | /v1/messages | 450 | 680 | $0.06 | ✓ |
| 14:30:35 | /v1/messages | 500 | 560 | $0.06 | ✓ |
```

That's it! You now have cost tracking enabled.

## Pricing Model Configuration

### For Paid Claude Plans

If you're on a paid Claude plan, use the actual token costs:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    # Claude Sonnet (example rates)
    input_token_cost: 0.000015    # $15 per million input tokens
    output_token_cost: 0.000075   # $75 per million output tokens
```

**Current Claude Pricing (as of January 2024):**
- **Claude Sonnet**: $15/$75 per million tokens (input/output)
- **Claude Opus**: $75/$375 per million tokens (input/output) 
- **Claude Haiku**: $2.50/$12.50 per million tokens (input/output)

### For Unlimited Plans

If you have unlimited API access but want cost insights for planning:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "max"  # Estimates costs without actual billing
```

This will show you what your usage *would* cost on a paid plan, helping you understand usage patterns.

## Configuration Options

### Basic Options

```yaml
cost_tracking:
  enabled: true                    # Enable/disable cost tracking
  pricing_model: "paid"           # "paid" or "max" 
  
  # Token costs (only for "paid" model)
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # Reporting settings
  reporting:
    include_in_issues: true        # Add reports to completed issues
    detailed_breakdown: true       # Include per-call breakdown
    cost_precision_decimals: 4     # Cost display precision
```

### Advanced Options

For production environments, you might want additional configuration:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  
  # Session management
  session_management:
    max_concurrent_sessions: 100   # Maximum parallel tracking sessions
    session_timeout_hours: 24      # Auto-cleanup timeout
    cleanup_interval_hours: 6      # How often to run cleanup
  
  # Optional database storage for analytics
  database:
    enabled: true                  # Enable SQLite storage
    file_path: "./costs.db"        # Database file location
    retention_days: 90             # How long to keep data
```

## Environment Variables

You can override any configuration using environment variables:

```bash
export SAH_COST_TRACKING_ENABLED=true
export SAH_COST_PRICING_MODEL=paid
export SAH_COST_INPUT_TOKEN_COST=0.000015
export SAH_COST_OUTPUT_TOKEN_COST=0.000075
```

This is useful for different environments (development, staging, production).

## Working Examples

### Example 1: Development Setup

Perfect for individual developers who want to track costs:

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

### Example 2: Team Environment

For teams that need shared cost visibility:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  database:
    enabled: true
    file_path: "./shared-costs.db"
    retention_days: 365
  aggregation:
    enabled: true
    retention_days: 90
```

### Example 3: Unlimited Plan Estimation

For users with unlimited API access who want usage insights:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "max"
  reporting:
    include_in_issues: true
    detailed_breakdown: false
```

## Verification and Testing

### Check Configuration

Verify your setup works correctly:

```bash
# Check configuration is valid
swissarmyhammer doctor

# Look for cost tracking status in the output
# Should show: "✓ Cost tracking: enabled"
```

### Test with Simple Issue

Create a test issue to verify cost tracking:

```bash
echo "# Test Issue

Fix a simple bug or implement a small feature." > test-cost-tracking.md

swissarmyhammer issue work test-cost-tracking.md
```

After completion, check `test-cost-tracking.md` for a cost analysis section.

## Common Issues and Quick Fixes

### Issue: Cost tracking not appearing in issues

**Solution**: Check your configuration:
```yaml
cost_tracking:
  reporting:
    include_in_issues: true  # Must be true
```

### Issue: "Invalid pricing model" error

**Solution**: Ensure you're using a valid pricing model:
```yaml
cost_tracking:
  pricing_model: "paid"  # Must be "paid" or "max"
```

### Issue: Token costs seem wrong

**Solution**: Verify you're using the correct rates for your Claude plan. Check Claude's current pricing documentation.

## Next Steps

Now that you have basic cost tracking working:

1. **[Configuration Reference](./cost-tracking-configuration.md)** - Learn about all available options
2. **[Advanced Examples](./cost-tracking-examples.md)** - Production configurations and custom reporting
3. **[Troubleshooting Guide](./cost-tracking-troubleshooting.md)** - Solutions to common problems
4. **[Architecture Guide](./cost-tracking-architecture.md)** - Understand how cost tracking works internally

## Need Help?

- Check the [troubleshooting guide](./cost-tracking-troubleshooting.md) for common issues
- Review [configuration reference](./cost-tracking-configuration.md) for all options
- Examine [working examples](./cost-tracking-examples.md) for different scenarios