---
title: do_issue
description: Code up an issue
---

## Goal

Use a tool to get the current issue.

You job is to do the work.

{% render "principals" %}
{% render "coding_standards" %}

## Process

- Look at the branch name, if this matches the name of an issue file like `issue/<issue_name>`
  - Good news, we are on a issue feature branch and all is well
  - This `<issue_name>` is the issue file we are working.
- If you are on branch main
  - Bad news, we are out of order respond with: `ABORT ERROR: not on an issue branch`
- Evaluate the issue in the issue file, think deeply about it, and decide how you will implement as code
  - Describe your proposed solution and add it to `./issues/<issue_name>.md`
  - Create a new markdown section in the issue like:

    ```markdown
    ## Proposed Solution
    <insert your steps here>
    ```

  - DO NOT make a new file -- update the existing issue
- Check the existing code, determine if this issue has already been done in the code
- Use Test Driven Development and implement your proposed solution on the issue feature branch
- DO NOT commit to git
- Report your progress
