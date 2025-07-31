---
title: Run Development Environment
description: Run the development environment following README and documentation instructions
---

## Goal

You need to run the development environment for the project that was just implemented. Your goal is to:
1. Find and read the README and any other relevant documentation
2. Identify the correct commands to run the development environment (frontend and/or backend)
3. Execute these commands
4. Monitor for any errors or issues
5. If errors occur, diagnose and fix them

## Process

1. **Locate Documentation**
   - Read README.md or README files in the project root
   - Look for sections about "Getting Started", "Running", "Development", "Setup", or similar
   - Check for package.json, Makefile, docker-compose.yml, or other build configuration files
   - Look for any scripts or commands that start the development server

2. **Identify Required Components**
   - Determine if there's a frontend, backend, or both
   - Check what technologies are used (Node.js, Python, Go, etc.)
   - Identify any required dependencies or services (databases, Redis, etc.)

3. **Prepare Environment**
   - Ensure all dependencies are installed (npm install, pip install, etc.)
   - Set up any required environment variables
   - Start any required services (databases, etc.)

4. **Run the Development Environment**
   - Execute the identified commands to start the development server(s)
   - If there are multiple components, run them in the correct order
   - Common patterns include:
     - `npm run dev` or `npm start`
     - `python manage.py runserver` or `python app.py`
     - `go run main.go`
     - `docker-compose up`
     - `make run` or `make dev`

5. **Monitor and Fix Issues**
   - Watch for any error messages during startup
   - Common issues to look for:
     - Port already in use
     - Missing dependencies
     - Database connection errors
     - Missing environment variables
     - Permission issues
   - If errors occur, diagnose the issue and fix it
   - After fixing, try running again

6. **Verify Success**
   - Confirm that the application started successfully
   - Note the URLs where the application is running (e.g., http://localhost:3000)
   - Report the status and any important information

## Important Notes

- Always read the project-specific documentation first
- Don't make assumptions about the technology stack
- If multiple run commands exist, prefer development/dev commands over production
- Be prepared to install missing dependencies
- Watch for and report any deprecation warnings or non-critical issues