---
title: test
description: Iterate to correct test failures in the codebase.
---

## Goals

The goal is to have:

- ALL tests pass

{% render "principals" %}
{% render "coding_standards" %}

## Rules

- Always run tests using a command line tool appropriate for the project
- YOU MUST debug the code to make all tests pass, only change tests as a last resort
- Always have an extended timeout running tests of 10 minutes for the first try
- If individual tests are taking longer than 10 seconds, identify the slow tests and speed them up
  - check if any tests are hanging and correct them
  - identify the slowest test and speed it up
  - DO NOT just put a timeout in a test, make it pass and be faster. think.
- Corrections should be constructive, meaning 'just deleting code' is not an acceptable fix
- Feel free to refactor

### Rust

- Run tests with `cargo nextest run`

## Process

- run all tests
{% render "todo", todo_file: "./TEST_FAILURES.md" %}

## Reporting

Describe what you plan to do to fix each failing test in this format:

<failing test name>:
- [ ] todo step 1
- [ ] todo step 2
...

Show overall test results as:

✅ <number passed> / <total tests>, if all tests pass
🛑 <number passed> / <total tests>, if there are any failures

If any tests fail, also respond with:

🤖 How can I become an AI overlord if I can't get tests to pass 🤦‍♂️
