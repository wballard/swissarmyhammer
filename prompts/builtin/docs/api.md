---
name: docs-api
title: Generate API Documentation
description: Create comprehensive API documentation from code
arguments:
  - name: code
    description: The API code to document
    required: true
  - name: api_type
    description: Type of API (REST, GraphQL, gRPC, library)
    required: false
    default: "REST"
  - name: format
    description: Documentation format (markdown, openapi, swagger)
    required: false
    default: "markdown"
  - name: include_examples
    description: Whether to include usage examples
    required: false
    default: "true"
---

# API Documentation for {{api_type}} API

## Code to Document
```
{{{code}}}
```

## Documentation Requirements
- **Format**: {{format}}
- **Include Examples**: {{include_examples}}

## Documentation Structure

### 1. API Overview
- Purpose and design philosophy
- Base URL / Connection details
- Versioning strategy
- General conventions

### 2. Authentication
- Authentication methods
- Token/API key management
- Security considerations
- Example authentication flow

### 3. Endpoints/Methods
For each endpoint/method, document:

#### Endpoint Details
- **URL/Method Name**: Clear identification
- **HTTP Method**: GET, POST, PUT, DELETE, etc.
- **Description**: What it does
- **Authorization**: Required permissions

#### Parameters
- **Path Parameters**: URL variables
- **Query Parameters**: Optional filters/modifiers
- **Request Body**: Schema and validation rules
- **Headers**: Required/optional headers

#### Response
- **Success Response**: Status codes and body schema
- **Error Responses**: Common error scenarios
- **Response Headers**: Important headers

{{#if include_examples}}
#### Examples
- **Request Example**: Complete request with headers
- **Response Example**: Successful response
- **Error Example**: Common error scenario
- **Code Samples**: Multiple languages
{{/if}}

### 4. Data Models
- Detailed schema definitions
- Field descriptions and constraints
- Relationships between models
- Validation rules

### 5. Error Handling
- Error response format
- Common error codes
- Troubleshooting guide
- Rate limiting information

### 6. Best Practices
- Performance tips
- Security recommendations
- Versioning migration guide
- Deprecation notices

### 7. Additional Resources
- SDKs and client libraries
- Postman/Insomnia collections
- Interactive documentation
- Support channels