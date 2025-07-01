---
title: Database Schema Designer
description: Design database schemas with proper relationships and constraints
arguments:
  - name: domain
    description: The business domain or application area
    required: true
  - name: database_type
    description: Type of database (PostgreSQL, MySQL, MongoDB, etc.)
    required: false
    default: "PostgreSQL"
  - name: entities
    description: Main entities or objects in the system
    required: false
    default: "auto-suggest based on domain"
---

# Database Schema Design for {{domain}}

Please help me design a database schema for a {{domain}} system using {{database_type}}.

## Context
- **Domain**: {{domain}}
- **Database**: {{database_type}}
- **Key Entities**: {{entities}}

## Requirements

### 1. Entity Analysis
Based on the {{domain}} domain, identify and define:
- Core entities and their attributes
- Relationships between entities
- Business rules and constraints

### 2. Schema Design
For each entity, provide:
- Table name (following naming conventions)
- Primary key strategy
- All columns with appropriate data types
- Foreign key relationships
- Indexes for performance
- Constraints (NOT NULL, UNIQUE, CHECK, etc.)

### 3. Relationships
Clearly define:
- One-to-One relationships
- One-to-Many relationships  
- Many-to-Many relationships (with junction tables)
- Self-referencing relationships if applicable

### 4. Sample Tables
Provide SQL DDL statements for creating the tables, including:
```sql
CREATE TABLE example_table (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    -- Add other columns
    CONSTRAINT check_constraint_name CHECK (condition)
);
```

### 5. Data Integrity
Include:
- Referential integrity constraints
- Business rule enforcement
- Audit trail considerations (created_at, updated_at, etc.)
- Soft delete strategy if needed

### 6. Performance Considerations
Recommend:
- Strategic indexes
- Partitioning strategies (if applicable)
- Normalization level (1NF, 2NF, 3NF)
- Denormalization opportunities

### 7. Sample Data
Provide a few rows of sample data for each table to illustrate the schema in use.

Please focus on creating a well-normalized, scalable schema that follows {{database_type}} best practices and supports the {{domain}} business requirements efficiently.