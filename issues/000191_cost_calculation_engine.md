# Cost Calculation Engine

## Summary

Implement the cost calculation engine that computes costs based on token usage for both Claude Code paid plans and Claude Code Max (unlimited) plans. This builds on the data structures from step 000190.

## Context

Different Claude Code plans have different pricing models:
- **Paid Plans**: Per-token pricing with different rates for input/output tokens
- **Max Plans**: Unlimited usage but still need token tracking for insights

The calculation engine must handle both models and provide accurate cost estimates based on current pricing.

## Requirements

### Core Components

1. **CostCalculator** - Main calculation engine
   - Pricing model detection and configuration
   - Token-to-cost conversion logic
   - Support for different pricing tiers

2. **PricingModel** - Enum for different plan types
   - `Paid(PaidPlanConfig)` - Per-token pricing
   - `Max(MaxPlanConfig)` - Unlimited with token tracking

3. **PricingRates** - Current pricing configuration
   - Input token cost per token
   - Output token cost per token
   - Configurable and updatable rates

### Pricing Integration

Based on research, implement current Claude pricing:
- Input tokens: ~$0.000015 per token (configurable)
- Output tokens: ~$0.000075 per token (configurable)
- Different rates for different model tiers

## Implementation Details

### File Location
- Extend: `swissarmyhammer/src/cost/` module
- New file: `swissarmyhammer/src/cost/calculator.rs`

### Key Features
- **Accurate Calculations**: Precise decimal arithmetic for costs
- **Model Support**: Handle Claude Sonnet, Opus, Haiku pricing differences
- **Rate Updates**: Support for pricing changes without code updates
- **Currency Handling**: USD-based calculations with proper precision

### Cost Calculation Logic
```rust
// Example calculation approach
fn calculate_call_cost(
    input_tokens: u32,
    output_tokens: u32,
    pricing_rates: &PricingRates,
) -> Decimal {
    let input_cost = Decimal::from(input_tokens) * pricing_rates.input_token_cost;
    let output_cost = Decimal::from(output_tokens) * pricing_rates.output_token_cost;
    input_cost + output_cost
}
```

### Configuration Integration
- Read pricing from configuration system
- Support environment variable overrides
- Default to current Claude Code pricing
- Validate pricing configuration on startup

## Testing Requirements

- Unit tests for all calculation scenarios
- Edge case testing (zero tokens, large numbers)
- Precision testing for decimal arithmetic
- Configuration validation tests
- Both paid and max plan testing

## Integration

This step builds on:
- Step 000190: Uses `CostSession` and `ApiCall` structures
- Integrates with: Step 000192 (configuration system)
- Prepares for: Step 000194 (MCP protocol integration)

## Dependencies

Add if not already present:
- `rust_decimal` for precise cost calculations
- Use existing `serde` for configuration serialization

## Success Criteria

- [ ] `CostCalculator` with accurate pricing calculations implemented
- [ ] Support for both paid and max plan models
- [ ] Configurable pricing rates system
- [ ] Precise decimal arithmetic for cost calculations
- [ ] Comprehensive test coverage for all scenarios
- [ ] Integration with existing configuration patterns
- [ ] Validation of pricing configuration integrity

## Notes

- Use `rust_decimal` for financial calculations to avoid floating point errors
- Support for pricing model changes without requiring code updates
- Consider future pricing tiers and model variations
- Ensure calculations match Anthropic's actual billing
- Test with real-world token counts and costs