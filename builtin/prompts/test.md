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
- Corrections should be constructive, meaning 'just deleting code' is not an acceptable fix
- YOU MUST debug the code to make all tests pass, only change tests as a last resort
- Feel free to refactor
- Do not make any editorial comments about why tests are failing, just fix them already. I can pay humans if I want excuses.

If tests take a 'long time' -- more than 5 minutes, check if any tests are hanging and correct them.

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
