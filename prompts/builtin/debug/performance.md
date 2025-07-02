---
name: debug-performance
title: Debug Performance Issues
description: Analyze performance problems and suggest optimization strategies
arguments:
  - name: problem_description
    description: Description of the performance issue
    required: true
  - name: metrics
    description: Performance metrics (e.g., "takes 5 seconds, expected <1 second")
    required: false
    default: "not provided"
  - name: code_snippet
    description: Relevant code that might be causing the issue
    required: false
    default: ""
  - name: environment
    description: Environment details (e.g., "production server with 8GB RAM")
    required: false
    default: "development"
---

# Performance Analysis: {{problem_description}}

## Current Performance
- **Issue**: {{problem_description}}
- **Metrics**: {{metrics}}
- **Environment**: {{environment}}

{% if code_snippet %}
## Code Under Analysis
```
{{ code_snippet }}
```
{% endif %}

## Performance Investigation Strategy

### 1. Profiling Approach
- Identify bottlenecks using appropriate profiling tools
- Measure actual vs perceived performance
- Focus on the critical path

### 2. Common Performance Issues to Check
- **Algorithm Complexity**: O(n²) or worse algorithms
- **Memory Issues**: Memory leaks, excessive allocations
- **I/O Bottlenecks**: Database queries, file operations, network calls
- **CPU Bound**: Intensive computations, inefficient loops
- **Concurrency**: Lock contention, synchronization overhead

### 3. Optimization Strategies
Based on the problem description, consider:
- Caching frequently accessed data
- Lazy loading and pagination
- Parallel processing where appropriate
- Algorithm optimization
- Database query optimization
- Resource pooling

### 4. Measurement and Validation
- Benchmark before and after changes
- Profile in production-like environment
- Monitor for regression
- Consider trade-offs (memory vs speed)

## Next Steps
1. Profile the specific operation
2. Identify the biggest bottleneck
3. Apply targeted optimization
4. Measure improvement
5. Iterate if needed