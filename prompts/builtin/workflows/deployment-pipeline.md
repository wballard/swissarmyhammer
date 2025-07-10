---
name: deployment-pipeline
title: Deployment Pipeline
description: Interactive deployment workflow with environment selection and rollback options
category: workflows
tags:
  - deployment
  - devops
  - interactive
  - example
arguments:
  - name: app_name
    description: Name of the application to deploy
    required: false
    default: "myapp"
    type_hint: string
  - name: build_type
    description: Type of build to create (development, staging, production)
    required: false
    default: "production"
    type_hint: string
  - name: auto_rollback
    description: Whether to automatically rollback on health check failure
    required: false
    default: "true"
    type_hint: string
---

# Deployment Pipeline Workflow

This workflow demonstrates an interactive deployment process with user choices,
environment selection, and rollback capabilities.

```mermaid
stateDiagram-v2
    [*] --> BuildApp: Start Deployment
    BuildApp --> TestBuild: Build successful
    BuildApp --> BuildFailed: Build failed
    TestBuild --> SelectEnvironment: Tests passed
    TestBuild --> TestsFailed: Tests failed
    SelectEnvironment --> DeployStaging: Choose staging
    SelectEnvironment --> DeployProduction: Choose production
    SelectEnvironment --> DeployDev: Choose development
    DeployStaging --> VerifyDeployment: Deployed
    DeployProduction --> VerifyDeployment: Deployed
    DeployDev --> VerifyDeployment: Deployed
    VerifyDeployment --> HealthCheck: Verification complete
    HealthCheck --> Success: Healthy
    HealthCheck --> RollbackDecision: Unhealthy
    RollbackDecision --> Rollback: Choose rollback
    RollbackDecision --> KeepCurrent: Choose keep
    Rollback --> RolledBack: Rollback complete
    BuildFailed --> [*]: Build failed
    TestsFailed --> [*]: Tests failed
    Success --> [*]: Deployment successful
    RolledBack --> [*]: Rolled back
    KeepCurrent --> [*]: Kept current version
    
    BuildApp: Build Application
    BuildApp: action: execute_prompt
    BuildApp: prompt: devops/build-app
    BuildApp: variables:
    BuildApp:   name: "{{ app_name }}"
    BuildApp:   type: "{{ build_type }}"
    
    TestBuild: Run Tests
    TestBuild: action: execute_prompt
    TestBuild: prompt: devops/run-tests
    TestBuild: variables:
    TestBuild:   build_id: "{{ BuildApp.build_id }}"
    TestBuild:   test_suite: "full"
    
    SelectEnvironment: Select Target Environment
    SelectEnvironment: action: user_choice
    SelectEnvironment: prompt: "Select deployment environment:"
    SelectEnvironment: choices:
    SelectEnvironment:   - "development"
    SelectEnvironment:   - "staging"
    SelectEnvironment:   - "production"
    
    DeployStaging: Deploy to Staging
    DeployStaging: action: execute_prompt
    DeployStaging: prompt: devops/deploy
    DeployStaging: variables:
    DeployStaging:   environment: "staging"
    DeployStaging:   build_id: "{{ BuildApp.build_id }}"
    
    DeployProduction: Deploy to Production
    DeployProduction: action: execute_prompt
    DeployProduction: prompt: devops/deploy
    DeployProduction: variables:
    DeployProduction:   environment: "production"
    DeployProduction:   build_id: "{{ BuildApp.build_id }}"
    DeployProduction:   require_approval: "true"
    
    DeployDev: Deploy to Development
    DeployDev: action: execute_prompt
    DeployDev: prompt: devops/deploy
    DeployDev: variables:
    DeployDev:   environment: "development"
    DeployDev:   build_id: "{{ BuildApp.build_id }}"
    
    VerifyDeployment: Verify Deployment
    VerifyDeployment: action: execute_prompt
    VerifyDeployment: prompt: devops/verify-deployment
    VerifyDeployment: variables:
    VerifyDeployment:   deployment_id: "{{ previous.deployment_id }}"
    
    HealthCheck: Health Check
    HealthCheck: action: execute_prompt
    HealthCheck: prompt: devops/health-check
    HealthCheck: variables:
    HealthCheck:   endpoint: "{{ previous.endpoint }}"
    HealthCheck:   timeout: "30"
    
    RollbackDecision: Rollback Decision
    RollbackDecision: action: user_choice
    RollbackDecision: prompt: "Health check failed! What would you like to do?"
    RollbackDecision: choices:
    RollbackDecision:   - "Rollback to previous version"
    RollbackDecision:   - "Keep current version and investigate"
    RollbackDecision: condition: "{{ auto_rollback == 'false' }}"
    
    Rollback: Execute Rollback
    Rollback: action: execute_prompt
    Rollback: prompt: devops/rollback
    Rollback: variables:
    Rollback:   deployment_id: "{{ VerifyDeployment.deployment_id }}"
    
    BuildFailed: Build Failed
    BuildFailed: action: set_variable
    BuildFailed: variable: status
    BuildFailed: value: "build_failed"
    BuildFailed: output: "Build failed: {{ BuildApp.error }}"
    
    TestsFailed: Tests Failed
    TestsFailed: action: set_variable
    TestsFailed: variable: status
    TestsFailed: value: "tests_failed"
    TestsFailed: output: "Tests failed: {{ TestBuild.failures }}"
    
    Success: Deployment Success
    Success: action: set_variable
    Success: variable: status
    Success: value: "deployed"
    Success: output: "Successfully deployed {{ app_name }} to {{ SelectEnvironment.choice }}!"
    
    RolledBack: Rollback Complete
    RolledBack: action: set_variable
    RolledBack: variable: status
    RolledBack: value: "rolled_back"
    RolledBack: output: "Successfully rolled back to previous version"
    
    KeepCurrent: Keeping Current Version
    KeepCurrent: action: set_variable
    KeepCurrent: variable: status
    KeepCurrent: value: "investigating"
    KeepCurrent: output: "Keeping current version for investigation"
```

## Usage

Run this workflow with:

```bash
# Interactive deployment
swissarmyhammer workflow run deployment-pipeline --set app_name=myservice

# Automated production deployment
swissarmyhammer workflow run deployment-pipeline \
  --set app_name=myservice \
  --set build_type=production \
  --set auto_rollback=true
```

## Features Demonstrated

1. **User Choices**: Environment selection and rollback decisions
2. **Conditional Transitions**: Different paths based on build/test results
3. **Error Handling**: Graceful handling of build and test failures
4. **State Variables**: Tracking deployment status throughout
5. **Dynamic Routing**: Different deployment paths for different environments