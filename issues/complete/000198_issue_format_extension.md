# Issue Markdown Format Extension for Cost Tracking

## Summary

Extend the issue markdown format to include comprehensive cost tracking information. This step implements the cost section specification and integrates with the existing issue storage system.

## Context

The SwissArmyHammer system stores issues as markdown files (see `src/issues/filesystem.rs`). This step extends the format to include detailed cost information at the bottom of completed issue files, as specified in the cost tracking PRD.

## Requirements

### Cost Section Format

Implement the cost section as specified in the PRD:

```markdown
## Cost Analysis

**Total Cost**: $2.34 (or "Unlimited Plan - 15,420 tokens used" for Max plan)
**Total API Calls**: 12
**Total Input Tokens**: 8,450
**Total Output Tokens**: 6,970
**Session Duration**: 2m 34s
**Completed**: 2024-01-15 14:32:17 UTC

### API Call Breakdown

| Timestamp | Endpoint | Input Tokens | Output Tokens | Duration | Cost |
|-----------|----------|--------------|---------------|----------|------|
| 14:30:15 | /v1/messages | 1,200 | 850 | 1.2s | $0.18 |
| 14:31:22 | /v1/messages | 2,100 | 1,400 | 2.1s | $0.31 |

### Cost Summary
- **Average cost per call**: $0.19
- **Most expensive call**: $0.45 (2,500 input + 1,800 output tokens)
- **Token efficiency**: 0.82 (output/input ratio)
```

### Implementation Features

1. **Cost Section Generation**
   - Format cost data into markdown sections
   - Support both paid and max plan formats
   - Handle missing or partial cost data gracefully
   - Generate human-readable summaries

2. **Integration with Issue Storage**
   - Extend `FileSystemIssueStorage` to append cost sections
   - Add cost data when marking issues complete
   - Preserve existing issue content and formatting
   - Support cost updates for re-processed issues

3. **Data Formatting**
   - Currency formatting with appropriate precision
   - Date/time formatting in UTC
   - Duration formatting (human-readable)
   - Token count formatting with thousands separators

### Technical Implementation

1. **Markdown Generation**
   - Create `CostSectionFormatter` for cost data rendering
   - Support different detail levels (summary vs full breakdown)
   - Handle locale-specific formatting preferences
   - Generate valid markdown with proper table formatting

2. **Storage Integration**
   - Extend issue completion workflow
   - Add cost data to issue metadata
   - Support cost section updates and regeneration
   - Maintain backward compatibility with existing issues

## Implementation Details

### File Modifications
- Extend: `swissarmyhammer/src/issues/filesystem.rs`
- Create: `swissarmyhammer/src/cost/formatting.rs`
- Update: Issue completion workflows

### Core Components

```rust
pub struct CostSectionFormatter {
    config: CostTrackingConfig,
    precision: usize,
    locale: String,
}

pub struct IssueCostData {
    pub session_data: CostSession,
    pub total_cost: Option<Decimal>,
    pub pricing_model: PricingModel,
    pub summary_stats: CostSummaryStats,
}

impl CostSectionFormatter {
    pub fn format_cost_section(&self, cost_data: &IssueCostData) -> String {
        // Generate complete markdown cost section
    }
}
```

### Integration Points
- Hook into `mark_complete` workflow in issue storage
- Extract cost data from completed workflow sessions
- Append formatted cost section to issue markdown
- Handle issues without cost data gracefully

### Formatting Features
- Currency formatting (USD with proper precision)
- Date/time formatting (ISO 8601 with timezone)
- Duration formatting (human-readable: "2m 34s")
- Large number formatting (thousands separators)
- Table formatting with proper alignment

### Configuration Options
- Enable/disable cost sections in issues
- Detail level configuration (summary vs full)
- Currency precision settings
- Date/time format preferences

## Testing Requirements

### Formatting Tests
- Cost section markdown generation accuracy
- Currency and number formatting validation
- Date/time formatting correctness
- Table formatting and alignment

### Integration Tests
- Issue completion with cost data
- Backward compatibility with existing issues
- Cost section updates and regeneration
- Error handling for missing cost data

### Content Validation Tests
- Markdown syntax validity
- Table structure correctness
- Cost calculation accuracy in output
- Summary statistics validation

## Integration

This step integrates with:
- Step 000190: Uses `CostSession` data
- Step 000196: Gets cost data from workflow completion
- Existing issue storage system

Prepares for:
- Complete cost tracking workflow
- User-facing cost reporting

## Proposed Solution

Based on analysis of the existing codebase, I will implement the cost tracking markdown format extension through the following approach:

### 1. Core Components Architecture

#### Cost Formatting Module (`src/cost/formatting.rs`)
```rust
pub struct CostSectionFormatter {
    config: CostFormattingConfig,
    precision: usize,
    locale: String,
}

pub struct IssueCostData {
    pub session_data: CostSession,
    pub total_cost: Option<Decimal>,  
    pub pricing_model: PricingModel,
    pub summary_stats: CostSummaryStats,
}

pub struct CostFormattingConfig {
    pub enabled: bool,
    pub detail_level: DetailLevel,
    pub currency_precision: usize,
    pub show_breakdown_table: bool,
    pub date_format: String,
}

pub enum DetailLevel {
    Summary,
    Full,
    Breakdown,
}
```

#### Integration Points
- **FileSystemIssueStorage::mark_complete()**: Extend to append cost sections before moving files
- **Cost extraction**: Hook into workflow completion to gather cost data from CostTracker
- **Backward compatibility**: Only append cost sections when cost data is available

### 2. Implementation Strategy

#### Phase 1: Cost Section Generation
1. Create `CostSectionFormatter::format_cost_section()` 
2. Implement currency formatting with locale support
3. Add date/time formatting with UTC handling
4. Build markdown table generation for API call breakdown

#### Phase 2: Storage Integration  
1. Modify `move_issue()` to accept optional cost data parameter
2. Append cost section to markdown content before file operations
3. Maintain atomicity of file operations
4. Preserve existing content and formatting

#### Phase 3: Data Collection
1. Create method to extract cost data from completed sessions  
2. Match issue numbers to cost session data
3. Calculate summary statistics and cost totals
4. Handle missing or partial cost data gracefully

### 3. Technical Implementation Details

#### File Structure Changes
- **Extend**: `swissarmyhammer/src/issues/filesystem.rs` - Add cost integration to mark_complete
- **Create**: `swissarmyhammer/src/cost/formatting.rs` - Cost section formatter
- **Update**: Cost tracker integration for issue completion

#### Markdown Generation Logic
```rust
impl CostSectionFormatter {
    pub fn format_cost_section(&self, cost_data: &IssueCostData) -> String {
        let mut sections = vec![];
        
        // Main cost analysis section
        sections.push(self.format_cost_summary(&cost_data));
        
        // API call breakdown table (if configured)
        if self.config.show_breakdown_table {
            sections.push(self.format_api_breakdown(&cost_data.session_data));
        }
        
        // Cost summary statistics
        sections.push(self.format_cost_statistics(&cost_data.summary_stats));
        
        sections.join("\n\n")
    }
}
```

#### Integration Workflow
1. **Issue Completion Trigger**: `mark_complete(number)` called
2. **Cost Data Lookup**: Search CostTracker for sessions matching issue 
3. **Data Processing**: Calculate totals, format timestamps, generate stats
4. **Markdown Generation**: Create cost section with proper formatting
5. **File Update**: Append cost section to issue content
6. **Atomic Move**: Move updated file to completed directory

### 4. Configuration Options

```rust
pub struct CostFormattingConfig {
    pub enabled: bool,                    // Enable/disable cost sections
    pub detail_level: DetailLevel,        // Summary, Full, or Breakdown
    pub currency_precision: usize,        // Decimal places for currency
    pub show_breakdown_table: bool,       // Include API call table
    pub date_format: String,              // ISO 8601 or custom format
    pub thousands_separator: String,      // For token count formatting  
    pub include_metadata: bool,           // Session metadata in output
}
```

### 5. Error Handling & Edge Cases

- **No Cost Data**: Skip cost section generation, log info message  
- **Partial Data**: Generate section with available data, note missing info
- **Multiple Sessions**: Aggregate data across sessions for same issue
- **File System Errors**: Ensure atomic operations, rollback on failure
- **Format Validation**: Validate markdown generation, check table structure

### 6. Testing Strategy

#### Unit Tests
- Cost section markdown generation accuracy
- Currency and number formatting validation  
- Date/time formatting correctness
- Table structure and alignment
- Edge cases (zero costs, single calls, etc.)

#### Integration Tests  
- Issue completion with cost data end-to-end
- Backward compatibility with existing issues
- Cost section updates and regeneration
- Error handling for missing cost data
- Multi-session aggregation

#### Performance Tests
- Large API call breakdown table generation
- Multiple issue completion batch operations
- Memory usage with extensive cost data

## Success Criteria

- [x] Comprehensive solution design completed
- [ ] Complete cost section markdown generation
- [ ] Seamless integration with issue storage system  
- [ ] Support for both paid and max plan formatting
- [ ] Configurable detail levels and formatting options
- [ ] Backward compatibility with existing issue format
- [ ] Comprehensive test coverage for all formatting scenarios
- [ ] Human-readable and well-formatted cost reports

## Notes

- Follow existing issue storage patterns and conventions
- Ensure cost sections don't interfere with existing issue parsing
- Support internationalization for future localization
- Consider different cost precision requirements
- Test with various cost data scenarios (small/large costs, many/few API calls)
- Maintain consistency with existing markdown formatting in issues
- Handle edge cases like zero costs, single API calls, etc.