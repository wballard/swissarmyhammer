---
name: database-query-optimizer
title: Database Query Optimizer
description: Optimize SQL queries for better performance
arguments:
  - name: query
    description: The SQL query to optimize
    required: true
  - name: database
    description: Database type (postgres, mysql, sqlite)
    required: false
    default: postgres
  - name: table_sizes
    description: Approximate table sizes (small, medium, large)
    required: false
    default: medium
  - name: indexes
    description: Available indexes (comma-separated)
    required: false
    default: ""
---

# SQL Query Optimization

## Original Query
```sql
{{query}}
```

## Database: {{database | capitalize}}

{% if database == "postgres" %}
### PostgreSQL Specific Optimizations
- Consider using EXPLAIN ANALYZE
- Check for missing indexes on JOIN columns
- Use CTEs for complex queries
- Consider partial indexes for WHERE conditions
{% elsif database == "mysql" %}
### MySQL Specific Optimizations
- Use EXPLAIN to check execution plan
- Consider covering indexes
- Optimize GROUP BY queries
- Check buffer pool size
{% else %}
### SQLite Specific Optimizations
- Use EXPLAIN QUERY PLAN
- Consider table order in JOINs
- Minimize use of LIKE with wildcards
{% endif %}

## Table Size Considerations
{% case table_sizes %}
{% when "small" %}
- Full table scans might be acceptable
- Focus on query simplicity
{% when "large" %}
- Indexes are critical
- Consider partitioning
- Avoid SELECT *
{% else %}
- Balance between indexes and write performance
- Monitor query execution time
{% endcase %}

{% if indexes %}
## Available Indexes
{% assign index_list = indexes | split: "," %}
{% for index in index_list %}
- {{ index | strip }}
{% endfor %}
{% endif %}

Provide:
1. Optimized query
2. Explanation of changes
3. Expected performance improvement
4. Additional index recommendations