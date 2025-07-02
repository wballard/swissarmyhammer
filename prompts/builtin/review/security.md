---
name: review-security
title: Security Code Review
description: Perform a comprehensive security review of code to identify vulnerabilities
arguments:
  - name: code
    description: The code to review for security issues
    required: true
  - name: context
    description: Context about the code (e.g., "handles user authentication")
    required: false
    default: "general purpose code"
  - name: language
    description: Programming language
    required: false
    default: "auto-detect"
  - name: severity_threshold
    description: Minimum severity to report (critical, high, medium, low)
    required: false
    default: "low"
---

# Security Review: {{context}}

## Code Under Review
```{{language}}
{{ code }}
```

## Security Analysis

### 1. Vulnerability Scan
Analyzing for common security vulnerabilities:

#### Input Validation
- SQL Injection risks
- Command Injection vulnerabilities
- Path Traversal attacks
- Cross-Site Scripting (XSS)
- XML/XXE injection

#### Authentication & Authorization
- Weak authentication mechanisms
- Missing authorization checks
- Session management issues
- Insecure password handling

#### Data Protection
- Sensitive data exposure
- Insecure cryptography usage
- Missing encryption for data in transit
- Hardcoded secrets or credentials

#### Security Misconfigurations
- Debug mode in production
- Verbose error messages
- Insecure defaults
- Missing security headers

### 2. Code-Specific Vulnerabilities
Based on the language and context:
- Race conditions
- Memory safety issues
- Resource leaks
- Denial of Service vectors

### 3. Severity Classification
Rate findings by severity ({{severity_threshold}} and above):
- **Critical**: Immediate exploitation risk
- **High**: Significant security impact
- **Medium**: Moderate risk requiring attention
- **Low**: Best practice violations

### 4. Recommendations

#### Immediate Actions
- Critical fixes that must be addressed
- Security patches to apply
- Configuration changes needed

#### Best Practices
- Secure coding patterns to adopt
- Libraries or frameworks that help
- Security testing to implement

#### Long-term Improvements
- Architectural changes for better security
- Security training recommendations
- Process improvements

### 5. Secure Code Example
Provide refactored code that addresses the security issues identified.