---
title: test
description: Iterate to correct test failures in the codebase.
---

## Goals

The goal is to have:

- ALL tests pass

{% render "principals" %}
{% render "coding_standards" %}

## Guidelines

- Always run tests using a command line tool appropriate for the project
- Always have an extended timeout running tests of 10 minutes for the first try
- If tests are taking longer than 10 minutes, identify the slow tests and speed them up
- Corrections should be constructive, meaning 'just deleting code' is not an acceptable fix
- YOU MUST debug the code to make all tests pass, only change tests as a last resort
- Feel free to refactor
- If tests take a 'long time' -- more than 2 minutes
  - check if any tests are hanging and correct them
  - identify the slowest test and speed it up

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
