---
title: Verify Development Environment Running
description: Check if the development environment is running successfully
---

## Goal

Verify that the development environment is running correctly and report the status.

## Process

1. **Check Running Processes**
   - Look for the processes that should be running
   - Verify they haven't crashed or exited with errors

2. **Check Logs**
   - Review the output from the running processes
   - Look for success messages like "Server running on port X"
   - Check for any error messages or warnings

3. **Test Connectivity** (if applicable)
   - If it's a web application, try to access the URLs
   - Check if the ports are open and responding

4. **Report Status**
   - If everything is running correctly, respond with "SUCCESS" and include:
     - The URLs where the application can be accessed
     - Any important information from the logs
   - If there are errors or the application isn't running, respond with "ERROR" and include:
     - The specific error messages
     - What component failed
     - Initial diagnosis of the problem

## Response Format

Your response MUST start with either "SUCCESS" or "ERROR" (case insensitive) followed by details.

Examples:
- "SUCCESS: Application running at http://localhost:3000"
- "ERROR: Backend failed to start - database connection refused"