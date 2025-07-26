---
title: Review Issue
description: Autonomously code review an correct the current open issue.
tags:
  - auto
---

## Usage

```bash
swissarmyhammer flow run review_branch

```

## States

```mermaid
stateDiagram-v2
    [*] --> start
    start --> review
    review --> correct
    correct --> test
    test --> commit
    commit --> [*]
```

## Actions

- start: log "Reviewing an issue"
- review: execute prompt "review/branch"
- correct: execute prompt "code/review"
- test: run workflow "tdd"
- commit: execute prompt "commit"

## Description

This workflow reviews a working branch and then implements that review.
