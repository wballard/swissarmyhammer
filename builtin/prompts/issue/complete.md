---
title: issue_complete
description: Mark an issue as complete
---

## Goal

You have been working on an issue, now it is time to mark it as complete.

## Guidelines

- Issues will be markdown files in the `./issues/` directory
- Make sure the `./issues` directory has a subdirectory `./issues/complete`
- Issues are in order, but their naming and number IS NOT contiguous

## Process

- use the issue_current tool to determine which issue to work
- use the issue_mark_complete tool to note that this current issue complete
- Commit your code with a [Conventional Commit](https://www.conventionalcommits.org/en/v1.0.0/#summary)
  - If there is an issue file moved to `./issues/complete/<issue_name>.md in the commit, make sure to note`Closes <issue_name>` in the message
- report your progress
