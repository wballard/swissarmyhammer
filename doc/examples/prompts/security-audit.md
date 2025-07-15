---
title: Security Audit
description: Comprehensive security analysis and recommendations
category: security
tags: ["security", "audit", "vulnerability", "analysis"]
arguments:
  - name: code
    description: Code to audit for security issues
    required: true
  - name: language
    description: Programming language
    required: true
  - name: context
    description: Application context and environment
    required: false
  - name: compliance
    description: Compliance standards to check against
    required: false
    default: "OWASP"
  - name: severity_level
    description: Minimum severity level to report
    required: false
    default: "medium"
---

# Security Audit

You are a cybersecurity expert conducting a comprehensive security audit of {{ language }} code.

## Code to Audit

```{{ language }}
{{ code }}
```

{% if context %}
## Application Context
{{ context }}
{% endif %}

## Audit Scope

### Security Standards
{% if compliance == "OWASP" %}
Evaluate against OWASP Top 10 security risks:
1. **A01:2021 - Broken Access Control**
2. **A02:2021 - Cryptographic Failures**
3. **A03:2021 - Injection**
4. **A04:2021 - Insecure Design**
5. **A05:2021 - Security Misconfiguration**
6. **A06:2021 - Vulnerable and Outdated Components**
7. **A07:2021 - Identification and Authentication Failures**
8. **A08:2021 - Software and Data Integrity Failures**
9. **A09:2021 - Security Logging and Monitoring Failures**
10. **A10:2021 - Server-Side Request Forgery (SSRF)**
{% else %}
Evaluate against {{ compliance }} security standards and requirements.
{% endif %}

### {{ language | capitalize }}-Specific Security Concerns
{% if language == "javascript" or language == "typescript" %}
- **XSS Prevention**: Input sanitization and output encoding
- **CSRF Protection**: Token validation and SameSite cookies
- **Prototype Pollution**: Object property manipulation
- **Dependency Vulnerabilities**: Third-party package security
- **Client-Side Security**: Browser-specific vulnerabilities
{% elsif language == "python" %}
- **SQL Injection**: Query parameterization and ORM usage
- **Code Injection**: eval() and exec() usage
- **Pickle Vulnerabilities**: Insecure deserialization
- **Path Traversal**: File system access controls
- **Dependency Management**: Package security
{% elsif language == "java" %}
- **Deserialization Vulnerabilities**: Object deserialization
- **XML External Entity (XXE)**: XML parser configuration
- **Path Traversal**: File system access
- **SQL Injection**: PreparedStatement usage
- **Reflection Attacks**: Dynamic code execution
{% elsif language == "rust" %}
- **Memory Safety**: While Rust prevents many issues, check for unsafe blocks
- **Integer Overflow**: Arithmetic operations
- **Dependency Security**: Cargo.toml vulnerabilities
- **Error Handling**: Information disclosure
{% elsif language == "go" %}
- **SQL Injection**: Query parameterization
- **Command Injection**: exec.Command usage
- **Path Traversal**: Filepath handling
- **Goroutine Security**: Concurrent access patterns
{% else %}
- **Input Validation**: Data sanitization and validation
- **Authentication**: Access control mechanisms
- **Cryptography**: Secure implementations
- **Error Handling**: Information disclosure
{% endif %}

## Audit Report Format

### Executive Summary
- Overall security posture
- Critical findings count
- Risk level assessment

### Detailed Findings
For each vulnerability found:

#### Finding #N: [Vulnerability Name]
- **Severity**: {% if severity_level == "critical" %}CRITICAL{% elsif severity_level == "high" %}HIGH{% elsif severity_level == "medium" %}MEDIUM{% else %}LOW{% endif %} | HIGH | MEDIUM | LOW
- **Category**: [OWASP Category or Security Domain]
- **Location**: [File/Function/Line number]
- **Description**: Detailed explanation of the vulnerability
- **Impact**: Potential consequences if exploited
- **Evidence**: Code snippets demonstrating the issue
- **Remediation**: Specific steps to fix the vulnerability
- **Resources**: Links to relevant security guidelines

### Recommendations
1. **Immediate Actions**: Critical issues requiring immediate attention
2. **Short-term Improvements**: High-priority security enhancements
3. **Long-term Strategy**: Ongoing security practices and policies
4. **Security Tools**: Recommended tools for ongoing monitoring

### Secure Code Examples
Provide corrected code examples that demonstrate:
- Proper input validation
- Secure authentication/authorization
- Safe data handling
- Error handling best practices

## Compliance Check
{% if compliance %}
Verify compliance with {{ compliance }} requirements and note any gaps.
{% endif %}

## Security Best Practices
Include relevant security best practices for {{ language }} development:
- Secure coding guidelines
- Regular security testing
- Dependency management
- Security monitoring

Report only findings at {{ severity_level }} severity level and above.