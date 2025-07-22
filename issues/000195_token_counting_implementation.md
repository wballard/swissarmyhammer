# Token Counting Implementation and Validation

## Summary

Implement accurate token counting for Claude Code API interactions, including validation against API-reported usage and fallback estimation when exact counts are unavailable.

## Context

Accurate token counting is crucial for cost tracking. While Claude API responses include usage statistics, we need robust token counting for validation, estimation when API data is missing, and understanding of token usage patterns.

## Requirements

### Token Counting Features

1. **Primary Token Counting**
   - Extract exact token counts from Claude API responses
   - Parse usage metadata from different API endpoints
   - Handle both input and output token reporting

2. **Validation and Fallback**
   - Validate API-reported counts against estimated counts
   - Implement fallback estimation when API data unavailable
   - Detect and handle token count discrepancies

3. **Estimation Methods**
   - Text-based token estimation using tokenization
   - Support for different Claude model tokenizers
   - Reasonable approximation algorithms

4. **Token Analysis**
   - Track token efficiency (output/input ratios)
   - Identify high-cost operations
   - Provide token usage insights

### Implementation Strategy

1. **API Response Parsing**
   - Parse Claude API `usage` fields accurately
   - Handle different response formats (streaming vs batch)
   - Support various API endpoint response structures

2. **Estimation Engine**
   - Implement text-based token estimation
   - Use Claude-compatible tokenization approach
   - Provide confidence levels for estimates

3. **Validation System**
   - Compare API counts vs estimated counts
   - Flag significant discrepancies
   - Log validation results for debugging

## Implementation Details

### File Location
- Create: `swissarmyhammer/src/cost/token_counter.rs`
- Utilities: `swissarmyhammer/src/cost/token_estimation.rs`

### Core Components

```rust
pub struct TokenCounter {
    /// Primary method: extract from API responses
    api_extractor: ApiTokenExtractor,
    /// Fallback method: estimate from text
    estimator: TokenEstimator,
    /// Validation and logging
    validator: TokenValidator,
}

pub struct TokenUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub source: TokenSource,
    pub confidence: ConfidenceLevel,
}

pub enum TokenSource {
    ApiResponse,      // From Claude API usage field
    Estimated,        // Calculated from text
    Mixed,           // Combination of both
}
```

### API Response Parsing
- Parse JSON response `usage` fields
- Handle different API versions and formats
- Support streaming response accumulation
- Extract both prompt and completion tokens

### Token Estimation
- Implement approximate tokenization
- Use reasonable heuristics (e.g., ~4 characters per token for English)
- Support different languages and content types
- Provide estimation confidence levels

### Validation Logic
- Compare API vs estimated counts
- Flag discrepancies > 10% difference
- Log validation results for analysis
- Maintain accuracy statistics

## Testing Requirements

### Accuracy Testing
- Compare against known token counts
- Test with various text types and lengths
- Validate API response parsing accuracy
- Test estimation algorithm precision

### Edge Case Testing
- Empty input/output scenarios
- Very large texts
- Special characters and formatting
- Different languages and encodings

### Performance Testing
- Token counting speed benchmarks
- Memory usage for large texts
- Concurrent token counting validation

## Integration

This step integrates with:
- Step 000194: Uses API call data from MCP integration
- Step 000191: Provides token counts for cost calculation
- Step 000190: Records token data in `ApiCall` structures

## Dependencies

Consider adding (if not present):
- `tiktoken` or similar tokenization library
- Text processing utilities
- Unicode handling libraries

## Success Criteria

- [ ] Accurate token extraction from Claude API responses
- [ ] Reliable token estimation for fallback scenarios
- [ ] Validation system comparing API vs estimated counts
- [ ] Comprehensive test coverage for all token counting scenarios
- [ ] Performance benchmarks meeting overhead requirements
- [ ] Support for different Claude model token counting
- [ ] Error handling for malformed API responses

## Notes

- Research Claude's actual tokenization approach for accuracy
- Consider caching token counts for repeated text
- Handle different API response formats gracefully
- Provide detailed logging for debugging token discrepancies
- Test with real-world Claude Code interaction patterns
- Consider future Claude model changes in token counting