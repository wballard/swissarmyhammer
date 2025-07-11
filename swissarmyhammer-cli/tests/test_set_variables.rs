//! Tests for --set variable functionality in workflows

use assert_cmd::Command;
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to create a test prompt for the workflow
fn create_test_prompt() -> String {
    r#"---
name: Test Prompt
title: Test Prompt
description: A test prompt for liquid template testing
arguments:
  - name: message
    description: The message
    required: true
  - name: count
    description: The count
    required: false
    default: "1"
---

This is a test prompt with message: {{ message }} and count: {{ count }}
"#
    .to_string()
}

/// Helper to create a test workflow that uses template variables
fn create_test_workflow_with_templates() -> String {
    r#"---
title: Test Workflow with Templates
description: A workflow that uses template variables
version: 1.0.0
---

# Test Workflow

```mermaid
stateDiagram-v2
    [*] --> start
    start --> process: Always
    process --> end: Always
    end --> [*]
    
    start: Log "Starting with {{ greeting | default: 'Hello' }}"
    process: Execute prompt "test-prompt" with message="{{ message }}" count="{{ count | default: '1' }}"
    end: Log "Finished processing {{ count }} items for {{ message }}"
```
"#.to_string()
}

#[test]
fn test_workflow_with_set_variables() {
    let temp_dir = TempDir::new().unwrap();
    // Create .swissarmyhammer/workflows directory in temp dir
    let workflow_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
    fs::create_dir_all(&workflow_dir).unwrap();

    // Create test workflow
    let workflow_path = workflow_dir.join("test-template.md");
    fs::write(&workflow_path, create_test_workflow_with_templates()).unwrap();

    // Create test prompt that the workflow uses
    let prompt_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
    fs::create_dir_all(&prompt_dir).unwrap();
    let prompt_path = prompt_dir.join("test-prompt.md");
    fs::write(&prompt_path, create_test_prompt()).unwrap();

    // Run workflow with --set variables
    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    cmd.arg("flow")
        .arg("run")
        .arg("test-template")
        .arg("--set")
        .arg("greeting=Bonjour")
        .arg("--set")
        .arg("message=TestMessage")
        .arg("--set")
        .arg("count=5")
        .arg("--dry-run")
        .current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Dry run mode"))
        .stdout(predicate::str::contains("test-template"));
}

#[test]
fn test_invalid_set_variable_format() {
    let temp_dir = TempDir::new().unwrap();
    // Create .swissarmyhammer/workflows directory in temp dir
    let workflow_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
    fs::create_dir_all(&workflow_dir).unwrap();

    // Create a minimal workflow so we get past the "workflow not found" error
    let workflow_path = workflow_dir.join("some-workflow.md");
    fs::write(
        &workflow_path,
        r#"---
title: Test
description: Test
version: 1.0.0
---

```mermaid
stateDiagram-v2
    [*] --> end
    end --> [*]
    
    end: Log "Done"
```
"#,
    )
    .unwrap();

    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    cmd.arg("flow")
        .arg("run")
        .arg("some-workflow")
        .arg("--set")
        .arg("invalid_format")
        .current_dir(&temp_dir);

    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Invalid set variable format"))
        .stderr(predicate::str::contains("Use key=value format"));
}

#[test]
fn test_set_and_var_together() {
    let temp_dir = TempDir::new().unwrap();
    // Create .swissarmyhammer/workflows directory in temp dir
    let workflow_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
    fs::create_dir_all(&workflow_dir).unwrap();

    // Create simple workflow
    let workflow_path = workflow_dir.join("simple.md");
    fs::write(
        &workflow_path,
        r#"---
title: Simple Workflow
description: Test workflow
version: 1.0.0
---

```mermaid
stateDiagram-v2
    [*] --> end
    end --> [*]
    
    end: Log "Done"
```
"#,
    )
    .unwrap();

    // Run workflow with both --var and --set
    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    cmd.arg("flow")
        .arg("run")
        .arg("simple")
        .arg("--var")
        .arg("context_var=value1")
        .arg("--set")
        .arg("template_var=value2")
        .arg("--dry-run")
        .current_dir(&temp_dir);

    cmd.assert().success().stdout(predicate::str::contains(
        "Variables: {\"context_var\": String(\"value1\")",
    ));
}

#[test]
fn test_full_workflow_execution_with_liquid_templates() {
    let temp_dir = TempDir::new().unwrap();

    // Create .swissarmyhammer/workflows directory
    let workflow_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
    fs::create_dir_all(&workflow_dir).unwrap();

    // Create test workflow with liquid templates
    let workflow_path = workflow_dir.join("template-workflow.md");
    fs::write(
        &workflow_path,
        r#"---
title: Template Test Workflow
description: Tests liquid template rendering during actual execution
version: 1.0.0
---

# Template Test Workflow

```mermaid
stateDiagram-v2
    [*] --> start
    start --> greeting: Always
    greeting --> counting: Always
    counting --> finish: Always
    finish --> [*]
```

## Actions

- start: Log "Starting workflow for {{ user_name | default: 'Unknown User' }}"
- greeting: Log "Hello {{ user_name }}! You are user number {{ user_id }}"
- counting: Log "Processing {{ item_count | default: '10' }} items"
- finish: Log "Workflow completed for {{ user_name }} (ID: {{ user_id }})"
"#,
    )
    .unwrap();

    // Run workflow with template variables
    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    cmd.arg("flow")
        .arg("run")
        .arg("template-workflow")
        .arg("--set")
        .arg("user_name=John Doe")
        .arg("--set")
        .arg("user_id=12345")
        .arg("--set")
        .arg("item_count=25")
        .current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Starting workflow for John Doe"))
        .stderr(predicate::str::contains(
            "Hello John Doe! You are user number 12345",
        ))
        .stderr(predicate::str::contains("Processing 25 items"))
        .stderr(predicate::str::contains(
            "Workflow completed for John Doe (ID: 12345)",
        ));
}

#[test]
fn test_workflow_with_missing_template_variables() {
    let temp_dir = TempDir::new().unwrap();

    // Create .swissarmyhammer/workflows directory
    let workflow_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
    fs::create_dir_all(&workflow_dir).unwrap();

    // Create workflow that uses variables not provided
    let workflow_path = workflow_dir.join("missing-vars.md");
    fs::write(
        &workflow_path,
        r#"---
title: Missing Variables Test
description: Tests behavior when template variables are missing
version: 1.0.0
---

```mermaid
stateDiagram-v2
    [*] --> start
    start --> [*]
```

## Actions

- start: Log "User: {{ username }}, Email: {{ email }}"
"#,
    )
    .unwrap();

    // Run workflow without providing required variables
    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    cmd.arg("flow")
        .arg("run")
        .arg("missing-vars")
        .current_dir(&temp_dir);

    // The workflow should still run but with the template placeholders intact
    cmd.assert().success().stderr(predicate::str::contains(
        "User: {{ username }}, Email: {{ email }}",
    ));
}

#[test]
fn test_workflow_with_complex_liquid_templates() {
    let temp_dir = TempDir::new().unwrap();

    // Create directories
    let workflow_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
    fs::create_dir_all(&workflow_dir).unwrap();

    let prompt_dir = temp_dir.path().join(".swissarmyhammer").join("prompts");
    fs::create_dir_all(&prompt_dir).unwrap();

    // Create a prompt that uses template variables
    let prompt_path = prompt_dir.join("template-prompt.md");
    fs::write(
        &prompt_path,
        r#"---
name: Template Prompt
title: Template Test Prompt
description: A prompt that uses template variables
arguments:
  - name: user
    description: User name
    required: true
  - name: task
    description: Task description
    required: true
---

Processing task "{{ task }}" for user {{ user }}.
"#,
    )
    .unwrap();

    // Create workflow with complex templates
    let workflow_path = workflow_dir.join("complex-templates.md");
    fs::write(&workflow_path, r#"---
title: Complex Template Workflow
description: Tests complex liquid template features
version: 1.0.0
---

```mermaid
stateDiagram-v2
    [*] --> start
    start --> process: Always
    process --> [*]
```

## Actions

- start: Set task_description="{{ task_type }} for {{ project_name | default: 'Default Project' }}"
- process: Execute prompt "template-prompt" with user="{{ user_name }}" task="{{ task_type }} for {{ project_name }}"
"#).unwrap();

    // Run workflow with complex template variables
    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    cmd.arg("flow")
        .arg("run")
        .arg("complex-templates")
        .arg("--set")
        .arg("user_name=Alice")
        .arg("--set")
        .arg("task_type=Code Review")
        .arg("--set")
        .arg("project_name=SwissArmyHammer")
        .arg("--dry-run") // Use dry-run since we don't want to actually execute the prompt
        .current_dir(&temp_dir);

    cmd.assert()
        .success()
        .stdout(predicate::str::contains("complex-templates"));
}

#[test]
fn test_workflow_with_malformed_liquid_templates() {
    let temp_dir = TempDir::new().unwrap();

    // Create .swissarmyhammer/workflows directory
    let workflow_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
    fs::create_dir_all(&workflow_dir).unwrap();

    // Create workflow with various malformed liquid templates
    let workflow_path = workflow_dir.join("malformed-templates.md");
    fs::write(
        &workflow_path,
        r#"---
title: Malformed Templates Test
description: Tests various malformed liquid template scenarios
version: 1.0.0
---

```mermaid
stateDiagram-v2
    [*] --> unclosed
    unclosed --> invalid_filter: Always
    invalid_filter --> nested_error: Always
    nested_error --> [*]
```

## Actions

- unclosed: Log "Unclosed tag: {{ name"
- invalid_filter: Log "Invalid filter: {{ name | nonexistent_filter }}"
- nested_error: Log "Nested error: {% for item in {{ items }} %}{{ item }}{% endfor %}"
"#,
    )
    .unwrap();

    // Run workflow with template variables
    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    cmd.arg("flow")
        .arg("run")
        .arg("malformed-templates")
        .arg("--set")
        .arg("name=Test")
        .arg("--set")
        .arg("items=[1,2,3]")
        .current_dir(&temp_dir);

    // The workflow should still run but with original text for malformed templates
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("Unclosed tag: {{ name"))
        .stderr(predicate::str::contains(
            "Invalid filter: {{ name | nonexistent_filter }}",
        ))
        .stderr(predicate::str::contains("Nested error: {%"));
}

#[test]
fn test_workflow_with_liquid_injection_attempts() {
    let temp_dir = TempDir::new().unwrap();

    // Create .swissarmyhammer/workflows directory
    let workflow_dir = temp_dir.path().join(".swissarmyhammer").join("workflows");
    fs::create_dir_all(&workflow_dir).unwrap();

    // Create workflow that tests injection attempts
    let workflow_path = workflow_dir.join("injection-test.md");
    fs::write(
        &workflow_path,
        r#"---
title: Injection Test
description: Tests liquid template security
version: 1.0.0
---

```mermaid
stateDiagram-v2
    [*] --> test
    test --> [*]
```

## Actions

- test: Log "User input: {{ user_input }}"
"#,
    )
    .unwrap();

    // Run workflow with potentially malicious input
    let mut cmd = Command::cargo_bin("swissarmyhammer").unwrap();
    cmd.arg("flow")
        .arg("run")
        .arg("injection-test")
        .arg("--set")
        .arg("user_input={{ '{% raw %}' }}{{ system }}{{ '{% endraw %}' }}")
        .current_dir(&temp_dir);

    // The workflow should safely render the input without executing any injected liquid code
    cmd.assert()
        .success()
        .stderr(predicate::str::contains("User input:"));
}
