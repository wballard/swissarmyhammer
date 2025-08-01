---
title: "review code"
description: "Improved the current code changes"
---

## Code Under Review

Please review the all code modified on the current branch.

{% render "principals" %}
{% render "coding_standards" %}

## Guidelines

DO NOT code now, the goal is just to create `./CODE_REVIEW.md` todo items.

## Process

{% render "issue/on_worktree" %}
- If there is an existing ./CODE_REVIEW.md
  - Remove any done todo items
- Focus on the files that have changed just on the current branch, this is your working set
- use the issue_current tool to determine which issue to review
  - Think deeply about the issue, does the code do a good job resolving the issue?
- Review the file changes in the working set, did you do a good job? -- Think deeply!
- Append any improvements to do to ./CODE_REVIEW.md
- Run a language appropriate lint
  - Append any lint warnings or errors to do to ./CODE_REVIEW.md
- Look for any TODO comments
  - If you find any, we need to add actually DOING them to ./CODE_REVIEW.md
- Look for any placeholders or comments about placeholders
  - If you find any, we need to add actually DOING them to ./CODE_REVIEW.md
- Report your progress

{% render "review_format" %}
