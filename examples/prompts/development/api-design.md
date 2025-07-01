---
title: API Design Assistant
description: Help design REST APIs with best practices
arguments:
  - name: resource
    description: The main resource or entity for the API
    required: true
  - name: operations
    description: What operations should be supported (CRUD, search, etc.)
    required: false
    default: "CRUD operations"
  - name: format
    description: Response format preference
    required: false
    default: "JSON"
---

# API Design for {{resource}}

Please help me design a REST API for managing {{resource}} resources.

## Requirements

- **Operations**: {{operations}}
- **Response Format**: {{format}}
- **RESTful principles**: Follow REST conventions
- **HTTP status codes**: Use appropriate status codes
- **Error handling**: Consistent error responses

## Please provide:

### 1. Resource Endpoints
List all the endpoints with HTTP methods:
- `GET /api/{{resource}}` - List resources
- `GET /api/{{resource}}/{id}` - Get specific resource
- `POST /api/{{resource}}` - Create new resource
- `PUT /api/{{resource}}/{id}` - Update resource
- `DELETE /api/{{resource}}/{id}` - Delete resource

### 2. Request/Response Examples
For each endpoint, provide:
- Sample request body (if applicable)
- Sample response body
- Possible HTTP status codes

### 3. Data Schema
Define the resource schema including:
- Required fields
- Optional fields
- Data types
- Validation rules

### 4. Error Handling
Standard error response format and common error scenarios.

### 5. Best Practices
Recommendations for:
- Pagination (for list endpoints)
- Filtering and sorting
- Versioning strategy
- Authentication/authorization considerations
- Rate limiting

Focus on creating a {{format}} API that follows RESTful principles and industry best practices.