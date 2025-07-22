# Cost Tracking Troubleshooting Guide

This guide helps you diagnose and resolve common issues with SwissArmyHammer's cost tracking system.

## Quick Diagnostics

### Check Cost Tracking Status

Run the diagnostic command to see current status:

```bash
swissarmyhammer doctor
```

Look for the cost tracking section in the output:

```
✓ Cost tracking: enabled
  - Pricing model: paid
  - Current sessions: 2 active
  - Database: disabled
  - Last cleanup: 2024-01-15 10:30:00 UTC
```

## Common Issues and Solutions

### Issue: Cost Tracking Not Working

**Symptoms:**
- No cost analysis appears in completed issues
- `swissarmyhammer doctor` shows "Cost tracking: disabled"

**Solution 1: Enable Cost Tracking**

Check your `swissarmyhammer.yaml` file:

```yaml
cost_tracking:
  enabled: true  # Must be explicitly set to true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
```

**Solution 2: Verify Configuration File Location**

Ensure SwissArmyHammer is reading your config file:

```bash
# Check current config
swissarmyhammer doctor --verbose

# Specify config file explicitly  
swissarmyhammer --config ./swissarmyhammer.yaml doctor
```

**Solution 3: Check Environment Variables**

Environment variables override config file settings:

```bash
# Check for conflicting environment variables
env | grep SAH_COST

# If you see SAH_COST_TRACKING_ENABLED=false, either:
unset SAH_COST_TRACKING_ENABLED
# Or set it properly:
export SAH_COST_TRACKING_ENABLED=true
```

### Issue: "Invalid pricing model" Error

**Symptoms:**
```
Error: Invalid pricing model 'payed'. Must be 'paid' or 'max'
```

**Solution:**

Fix the spelling in your configuration:

```yaml
cost_tracking:
  pricing_model: "paid"  # Not "payed", "Pay", or "PAID"
```

Valid options are exactly:
- `"paid"` - for users on paid Claude plans  
- `"max"` - for unlimited plan users

### Issue: Missing Token Costs

**Symptoms:**
```
Error: Pricing rates required when pricing_model is 'paid'
```

**Solution:**

Add the required `rates` section:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:  # This section is required for "paid" model
    input_token_cost: 0.000015
    output_token_cost: 0.000075
```

For unlimited plans, use:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "max"  # No rates section needed
```

### Issue: Cost Reports Not Appearing in Issues

**Symptoms:**
- Cost tracking is enabled and working
- Issues complete successfully but no cost section appears

**Solution 1: Check Reporting Settings**

Ensure reporting is enabled:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
  reporting:
    include_in_issues: true  # Must be true
```

**Solution 2: Verify Issue Completion**

Cost reports are only added to *completed* issues. Check that:
1. Your issue workflow completed successfully
2. The issue file exists and is writable
3. No errors occurred during issue processing

**Solution 3: Check File Permissions**

Ensure SwissArmyHammer can write to the issue file:

```bash
# Check file permissions
ls -la your-issue.md

# Fix permissions if needed
chmod 644 your-issue.md
```

### Issue: Incorrect Cost Calculations

**Symptoms:**
- Costs seem too high or too low
- Token counts don't match expectations

**Solution 1: Verify Token Rates**

Check you're using the correct rates for your Claude plan:

| Model | Input (per million) | Output (per million) | Config Values |
|-------|-------|-------|-------|
| Sonnet | $15 | $75 | `0.000015` / `0.000075` |
| Opus | $75 | $375 | `0.000075` / `0.000375` |
| Haiku | $2.50 | $12.50 | `0.0000025` / `0.0000125` |

**Solution 2: Enable Detailed Breakdown**

Get more visibility into calculations:

```yaml
cost_tracking:
  reporting:
    detailed_breakdown: true  # Shows per-API-call costs
    include_performance_metrics: true  # Shows token efficiency
```

**Solution 3: Check Token Counting**

Enable debug logging to see token extraction:

```bash
RUST_LOG=debug swissarmyhammer issue work your-issue.md
```

Look for log entries about token counting and API response parsing.

### Issue: Database Errors

**Symptoms:**
- "Database connection failed" errors
- "Unable to create cost database" messages

**Solution 1: Check Database Configuration**

Verify database settings:

```yaml
cost_tracking:
  database:
    enabled: true
    file_path: "./costs.db"  # Ensure directory exists and is writable
    connection_timeout_seconds: 30
```

**Solution 2: Create Database Directory**

Ensure the database directory exists:

```bash
# If file_path is "./data/costs.db"
mkdir -p ./data

# Check permissions
ls -ld ./data
```

**Solution 3: Disable Database Temporarily**

If database issues persist, disable it temporarily:

```yaml
cost_tracking:
  database:
    enabled: false  # Cost tracking will work without database
```

**Solution 4: Reset Database**

If database is corrupted:

```bash
# Backup first (if needed)
cp costs.db costs.db.backup

# Remove corrupted database
rm costs.db

# SwissArmyHammer will recreate it on next run
```

### Issue: High Memory Usage

**Symptoms:**
- SwissArmyHammer using excessive memory
- System becoming slow during issue processing

**Solution 1: Reduce Session Limits**

Lower memory usage by reducing session retention:

```yaml
cost_tracking:
  session_management:
    max_concurrent_sessions: 50     # Reduce from default 100
    max_api_calls_per_session: 200  # Reduce from default 500
    session_timeout_hours: 12       # Reduce from default 24
```

**Solution 2: Enable Database Storage**

Move cost data from memory to disk:

```yaml
cost_tracking:
  database:
    enabled: true  # Moves long-term storage to SQLite
```

**Solution 3: Increase Cleanup Frequency**

Clean up old data more often:

```yaml
cost_tracking:
  session_management:
    cleanup_interval_hours: 2  # Clean up every 2 hours instead of 6
```

**Solution 4: Disable Aggregation**

Disable memory-intensive features:

```yaml
cost_tracking:
  aggregation:
    enabled: false  # Disables cross-issue analytics
```

### Issue: Slow Performance

**Symptoms:**
- Issue processing slower than expected
- Long delays when starting/completing issues

**Solution 1: Optimize Reporting**

Reduce reporting overhead:

```yaml
cost_tracking:
  reporting:
    detailed_breakdown: false  # Skip detailed API call tables
    include_performance_metrics: false  # Skip performance calculations
```

**Solution 2: Increase Database Timeouts**

If using database, increase timeouts:

```yaml
cost_tracking:
  database:
    connection_timeout_seconds: 60  # Increase from default 30
    max_connections: 20             # Increase from default 10
```

**Solution 3: Disable Features Temporarily**

For debugging, disable cost tracking entirely:

```yaml
cost_tracking:
  enabled: false
```

Then re-enable with minimal configuration once performance is acceptable.

### Issue: Configuration Validation Errors

**Symptoms:**
- SwissArmyHammer won't start
- Configuration validation errors on startup

**Solution 1: Check YAML Syntax**

Validate your YAML file:

```bash
# Use a YAML validator
python -c "import yaml; yaml.safe_load(open('swissarmyhammer.yaml'))"

# Or use an online YAML validator
```

**Solution 2: Check Value Ranges**

Common validation failures:

```yaml
cost_tracking:
  rates:
    input_token_cost: 0.000015   # Must be > 0 and < 1.0
    output_token_cost: 0.000075  # Must be > 0 and < 1.0
  session_management:
    max_concurrent_sessions: 100 # Must be 1-10,000
    session_timeout_hours: 24    # Must be 1-168
```

**Solution 3: Use Default Values**

Start with minimal valid configuration:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015
    output_token_cost: 0.000075
```

Add additional options after confirming basic setup works.

## Advanced Troubleshooting

### Enable Debug Logging

For detailed troubleshooting information:

```bash
RUST_LOG=swissarmyhammer::cost=debug swissarmyhammer issue work your-issue.md
```

This shows detailed logs for:
- Cost session creation and management
- API call recording and token extraction
- Cost calculations and formatting
- Database operations (if enabled)

### Check System Resources

Monitor resource usage during cost tracking:

```bash
# Monitor memory usage
top -p $(pgrep swissarmyhammer)

# Check disk space (if using database)
df -h .

# Monitor file descriptors  
lsof -p $(pgrep swissarmyhammer) | wc -l
```

### Validate Token Extraction

Test token counting with a simple API call:

```bash
# Enable token counting debug logs
RUST_LOG=swissarmyhammer::cost::token_counter=trace swissarmyhammer issue work test-issue.md
```

Look for log entries showing:
- Raw API response parsing
- Token count extraction attempts
- Validation against estimated counts

### Database Inspection

If using SQLite database, inspect the data:

```bash
# Open database
sqlite3 costs.db

# Check tables
.tables

# View recent sessions
SELECT * FROM cost_sessions ORDER BY started_at DESC LIMIT 10;

# View API calls
SELECT * FROM api_calls ORDER BY started_at DESC LIMIT 10;

# Exit
.quit
```

## Getting Help

### Before Asking for Help

1. **Run `swissarmyhammer doctor`** and include the output
2. **Check recent logs** with debug logging enabled
3. **Test with minimal configuration** to isolate the issue
4. **Verify your SwissArmyHammer version** is current

### Information to Provide

When reporting issues, include:

- **SwissArmyHammer version**: `swissarmyhammer --version`
- **Operating system and version**
- **Full configuration file** (redact sensitive values)
- **Complete error message** and stack trace if available
- **Debug logs** for the failing operation
- **Steps to reproduce** the issue

### Common Support Scenarios

**Performance Issues:**
- System specs (CPU, RAM, disk)
- Number of concurrent issues being processed
- Database file size (if using database)
- Sample issue complexity and API call counts

**Configuration Issues:**  
- Full `swissarmyhammer.yaml` content
- Environment variables (relevant SAH_COST_* variables)
- File permissions and ownership
- Directory structure where SwissArmyHammer is running

**Cost Calculation Issues:**
- Expected vs actual costs
- Claude plan type and pricing tier
- Sample API responses (if possible)
- Token counting debug logs

### Emergency Troubleshooting

If cost tracking is completely broken and blocking your work:

1. **Disable cost tracking** immediately:
   ```yaml
   cost_tracking:
     enabled: false
   ```

2. **Continue your work** without cost tracking

3. **Troubleshoot separately** using test issues

4. **Re-enable gradually** once issues are resolved

This ensures cost tracking problems don't interfere with your primary SwissArmyHammer usage.