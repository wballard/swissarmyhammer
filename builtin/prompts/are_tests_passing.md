---
title: are_tests_passing
description: "Check if all tests are passing."
---

## Goal

We want to know if unit tests are passing.

## Rules

If you run tests, on any failure, respond with:
  - the word NO on a single line
  - a yaml list of the failing test names
If you run tests, and they all pass, respond only with YES.

### Rust

- Run tests with `cargo nextest run`
