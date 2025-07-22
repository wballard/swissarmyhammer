# Configuration System Extension for Cost Tracking

## Summary

Extend the existing YAML configuration system to support cost tracking settings. This includes pricing configuration, tracking preferences, and integration with the existing config infrastructure.

## Context

The SwissArmyHammer configuration system (`config.rs`) already supports YAML configuration files and environment variables. This step extends it to include cost tracking settings while maintaining backward compatibility.

## Requirements

### Configuration Structure

Add cost tracking section to YAML configuration:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"  # "paid" or "max"
  rates:
    input_token_cost: 0.000015   # per token in USD
    output_token_cost: 0.000075  # per token in USD
  session_management:
    max_concurrent_sessions: 100
    session_timeout_hours: 24
    cleanup_interval_hours: 6
  aggregation:
    enabled: true
    retention_days: 90
    max_stored_sessions: 10000
  reporting:
    include_in_issues: true
    detailed_breakdown: true
    cost_precision_decimals: 4
```

### Configuration Fields

1. **Basic Settings**
   - `enabled`: Master toggle for cost tracking
   - `pricing_model`: "paid" or "max" plan type

2. **Pricing Configuration**
   - `input_token_cost`: Cost per input token (Decimal)
   - `output_token_cost`: Cost per output token (Decimal)
   - Support for model-specific pricing overrides

3. **Session Management**
   - Maximum concurrent sessions
   - Session timeout settings
   - Cleanup intervals

4. **Aggregation Settings**
   - Data retention policies
   - Storage limits
   - Performance tuning options

5. **Reporting Options**
   - Issue integration preferences
   - Detail levels
   - Format options

## Implementation Details

### File Modifications
- Extend: `swissarmyhammer/src/config.rs`
- Add new struct: `CostTrackingConfig`
- Update: `Config` struct to include cost tracking

### Integration Points
- Use existing `serde` patterns for YAML parsing
- Follow existing environment variable naming (`SAH_COST_*`)
- Maintain existing validation patterns
- Use `EnvLoader` for environment overrides

### Environment Variables
```
SAH_COST_TRACKING_ENABLED=true
SAH_COST_PRICING_MODEL=paid
SAH_COST_INPUT_TOKEN_COST=0.000015
SAH_COST_OUTPUT_TOKEN_COST=0.000075
```

### Default Values
- Cost tracking disabled by default
- Pricing model defaults to "paid"
- Current Claude Code pricing as defaults
- Reasonable limits for session management

### Validation
- Validate pricing model values ("paid" or "max")
- Ensure positive pricing values
- Check reasonable limits for session counts
- Validate retention settings

## Testing Requirements

- Configuration parsing tests (YAML and environment)
- Default value verification
- Environment variable override testing
- Configuration validation testing
- Backward compatibility tests (existing configs still work)

## Integration

This step integrates with:
- Step 000190: Provides configuration for `CostTracker`
- Step 000191: Configures pricing for `CostCalculator`
- Existing: YAML configuration system and environment loading

## Success Criteria

- [ ] `CostTrackingConfig` struct with all required fields
- [ ] YAML configuration parsing with serde integration
- [ ] Environment variable support following existing patterns
- [ ] Configuration validation with appropriate error messages
- [ ] Default values matching specification requirements
- [ ] Backward compatibility maintained
- [ ] Comprehensive test coverage for all configuration scenarios

## Notes

- Follow existing patterns in `config.rs` for consistency
- Use `rust_decimal` for precise cost configuration values
- Ensure configuration changes don't require application restart where possible
- Support both development and production configuration patterns
- Consider future configuration needs (model-specific pricing, etc.)