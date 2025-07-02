---
name: refactor-extract
title: Extract Method/Function
description: Extract code into well-named, reusable methods or functions
arguments:
  - name: code
    description: The code containing logic to extract
    required: true
  - name: extract_purpose
    description: What the extracted method should do
    required: true
  - name: method_name
    description: Suggested name for the extracted method
    required: false
    default: "auto-suggest"
  - name: scope
    description: Scope for the extraction (method, function, class, module)
    required: false
    default: "method"
---

# Extract {{scope}}: {{extract_purpose}}

## Original Code
```
{{ code }}
```

## Extraction Goal
- **Purpose**: {{extract_purpose}}
- **Suggested Name**: {{method_name}}
- **Scope**: {{scope}}

## Extraction Analysis

### 1. Code to Extract
Identify the specific code block that:
- Represents a single responsibility
- Can be reused
- Makes the code more readable
- Has clear inputs and outputs

### 2. Method Signature Design
- **Name**: Choose a descriptive verb-noun combination
- **Parameters**: Identify required inputs
- **Return Value**: Determine what needs to be returned
- **Side Effects**: Document any side effects

### 3. Refactoring Steps

#### Step 1: Identify Dependencies
- Variables used from outer scope
- Objects or services accessed
- State modifications

#### Step 2: Define Interface
```
function {{method_name}}(param1, param2, ...) {
    // Extracted logic here
    return result;
}
```

#### Step 3: Extract and Replace
- Move code to new method
- Replace original code with method call
- Pass necessary parameters
- Handle return values

#### Step 4: Optimize
- Remove duplication
- Simplify parameter passing
- Consider default values
- Add type annotations

### 4. Best Practices

#### Naming
- Use intention-revealing names
- Avoid abbreviations
- Be consistent with codebase

#### Size
- Keep methods small and focused
- Extract until it reads like prose
- One level of abstraction

#### Testing
- Write tests for extracted method
- Ensure behavior unchanged
- Test edge cases

### 5. Example Refactoring
Show the complete refactored code with:
- Extracted method definition
- Updated original code
- Improved readability
- Maintained functionality