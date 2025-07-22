# Cost Tracking Overview

SwissArmyHammer's cost tracking system provides comprehensive monitoring and analysis of Claude API usage costs during issue workflow execution. This feature helps you understand, optimize, and budget for AI assistance costs in your development workflow.

## What is Cost Tracking?

Cost tracking automatically monitors every interaction with Claude's API during issue processing, recording:

- **Token Usage**: Input and output token consumption for each API call
- **API Costs**: Real-time cost calculation based on current Claude pricing
- **Session Analytics**: Complete cost breakdown per issue workflow
- **Performance Metrics**: API response times and success rates
- **Aggregated Reports**: Cross-issue cost analysis and trends

## Key Benefits

### **🔍 Visibility**
Get detailed insights into your AI usage patterns with comprehensive reporting that shows exactly where your Claude API costs are coming from.

### **📊 Optimization**  
Identify high-cost operations and optimize your prompts and workflows to reduce unnecessary API usage while maintaining effectiveness.

### **💰 Budget Planning**
Accurate cost tracking enables informed budgeting decisions and helps predict monthly API costs based on actual usage patterns.

### **📈 Performance Analytics**
Track not just costs but also performance metrics like token efficiency ratios and API response times to optimize your overall workflow.

### **🔧 Flexible Configuration**
Support for both paid plans (per-token billing) and unlimited plans (cost estimation for planning) with customizable pricing models.

## How It Works

1. **Automatic Session Creation**: When you start working on an issue, a cost tracking session begins automatically
2. **API Call Recording**: Every Claude API interaction is captured with detailed token usage and timing
3. **Real-time Calculation**: Costs are calculated immediately using the latest pricing information
4. **Session Completion**: When issue work completes, a comprehensive cost report is generated
5. **Issue Integration**: Cost analysis is automatically added to completed issue documentation

## Pricing Models Supported

### Paid Plans
For users on Claude's paid tiers, cost tracking provides accurate per-token billing calculations:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "paid"
  rates:
    input_token_cost: 0.000015   # $0.000015 per input token
    output_token_cost: 0.000075  # $0.000075 per output token
```

### Unlimited Plans  
For users with unlimited access, cost tracking estimates hypothetical costs for budgeting and optimization:

```yaml
cost_tracking:
  enabled: true
  pricing_model: "max"  # Unlimited plan with cost estimation
```

## Sample Cost Report

When an issue is completed, you'll see detailed cost analysis like this:

```markdown
## Cost Analysis

**Total Cost**: $0.34
**Total API Calls**: 4  
**Total Input Tokens**: 2,100
**Total Output Tokens**: 3,250
**Session Duration**: 3m 45s
**Completed**: 2024-01-15 14:30:25 UTC

### Cost Breakdown
- **API Call #1**: $0.08 (500 input, 750 output tokens) - `/v1/messages`
- **API Call #2**: $0.15 (800 input, 1,200 output tokens) - `/v1/messages`  
- **API Call #3**: $0.07 (450 input, 680 output tokens) - `/v1/messages`
- **API Call #4**: $0.04 (350 input, 620 output tokens) - `/v1/messages`

### Performance Metrics
- **Average cost per call**: $0.085
- **Token efficiency**: 1.55 (output/input ratio)
- **Success rate**: 100% (4/4 successful)
- **Average response time**: 1.2s
```

## Getting Started

Cost tracking is disabled by default to ensure it doesn't interfere with existing workflows. To enable it:

1. **Enable in Configuration**: Add cost tracking configuration to your `swissarmyhammer.yaml`
2. **Choose Pricing Model**: Select "paid" for actual billing or "max" for cost estimation  
3. **Configure Rates**: Set appropriate token costs for your Claude plan
4. **Start Tracking**: Begin working on issues - tracking happens automatically

## Data Privacy and Storage

- **In-Memory by Default**: Cost data is stored in memory during execution with automatic cleanup
- **Optional Database**: Enable SQLite backend for persistent storage and advanced analytics
- **No External Transmission**: All cost tracking data stays local to your system
- **Configurable Retention**: Control how long cost data is retained (default 7 days for in-memory)

## What's Next?

- [Getting Started Guide](./cost-tracking-getting-started.md) - Set up cost tracking in 5 minutes
- [Configuration Reference](./cost-tracking-configuration.md) - Complete configuration options
- [Troubleshooting](./cost-tracking-troubleshooting.md) - Common issues and solutions
- [Advanced Examples](./cost-tracking-examples.md) - Production configurations and custom reporting