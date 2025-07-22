# Memoranda Quickstart Guide

A step-by-step tutorial to get you started with SwissArmyHammer's memoranda system for structured note-taking and knowledge management.

## Overview

By the end of this guide, you'll know how to:
- Create and manage memos via CLI and MCP tools
- Search through your memo collection efficiently
- Integrate memos with AI assistants like Claude
- Organize memos for different workflows

**Time to complete**: 10-15 minutes  
**Prerequisites**: SwissArmyHammer CLI installed

## Step 1: Your First Memo

Let's start by creating your first memo using the command line:

```bash
# Create a simple memo
swissarmyhammer memo create "My First Memo" -c "This is my first memo using SwissArmyHammer!"
```

**Expected output:**
```
✅ Created memo: My First Memo
🆔 ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV
📅 Created: 2024-01-15 14:30:25 UTC
```

**Key points:**
- Each memo gets a unique ULID (26 characters, sortable by time)
- Memos are stored in `.swissarmyhammer/memos/` directory
- Creation timestamp is automatically tracked

## Step 2: Creating Memos with Rich Content

Now let's create a memo with markdown content:

```bash
swissarmyhammer memo create "Project Ideas" -c "# Project Ideas

## Web Development
- [ ] Personal blog with dark mode
- [ ] Recipe sharing platform
- [ ] Developer portfolio site

## Learning Goals
- Master async/await patterns
- Learn container orchestration
- Improve UI/UX skills

## Resources
- [Rust Book](https://doc.rust-lang.org/book/)
- [MDN Web Docs](https://developer.mozilla.org/)
"
```

**Why use markdown?**
- Structured formatting for better readability
- Support for lists, headings, links, and code blocks
- Great for technical documentation and planning

## Step 3: Interactive Content Input

For longer memos, use interactive input mode:

```bash
# Create memo without -c flag for interactive mode
swissarmyhammer memo create "Meeting Notes - Team Standup"
```

When prompted, enter your content:
```
📝 Enter memo content:
   💡 Type or paste your content, then press Ctrl+D (or Cmd+D on Mac) when finished

# Team Standup - January 15, 2024

## Attendees
- Alice (PM)
- Bob (Engineering) 
- Carol (Design)

## Updates
### Alice
- Completed user story prioritization
- Next: Stakeholder review meeting

### Bob
- Fixed authentication bug (#123)
- Working on API rate limiting
- Blocker: Need database schema approval

### Carol
- Finished wireframes for dashboard
- Starting prototype this week

## Action Items
- [ ] Alice: Schedule stakeholder review (by Friday)
- [ ] Bob: Submit schema proposal (by Wednesday)
- [ ] Carol: Share prototype with team (next Monday)

## Next Meeting
- Date: January 22, 2024
- Time: 9:00 AM
- Location: Conference Room B
^D
```

Press `Ctrl+D` (or `Cmd+D` on Mac) to finish input.

## Step 4: Browsing Your Memos

List all your memos to see what you've created:

```bash
swissarmyhammer memo list
```

**Expected output:**
```
📝 Found 2 memos

🆔 01BRZ3NDEKTSV4RRFFQ69G5FAW
📄 Meeting Notes - Team Standup
📅 2024-01-15 14:35:12 UTC
💬 # Team Standup - January 15, 2024\n\n## Attendees\n- Alice (PM)\n- Bob (Engineering)\n- Carol...

🆔 01ARZ3NDEKTSV4RRFFQ69G5FAV
📄 Project Ideas
📅 2024-01-15 14:32:08 UTC
💬 # Project Ideas\n\n## Web Development\n- [ ] Personal blog with dark mode\n- [ ] Recipe sharing...
```

**Understanding the output:**
- Memos listed newest first (ULID sorting)
- Shows preview of first 100 characters
- Each memo has title, ID, and creation time

## Step 5: Retrieving Specific Memos

Get the full content of a specific memo using its ID:

```bash
# Copy the ID from the list command above
swissarmyhammer memo get 01BRZ3NDEKTSV4RRFFQ69G5FAW
```

**Expected output:**
```
📝 Memo: Meeting Notes - Team Standup
🆔 ID: 01BRZ3NDEKTSV4RRFFQ69G5FAW
📅 Created: 2024-01-15 14:35:12 UTC
🔄 Updated: 2024-01-15 14:35:12 UTC

Content:
# Team Standup - January 15, 2024

## Attendees
- Alice (PM)
- Bob (Engineering) 
- Carol (Design)

[... full memo content ...]
```

## Step 6: Searching Your Memos

Find memos by keywords:

```bash
# Search for memos containing "project"
swissarmyhammer memo search "project"

# Search for specific terms
swissarmyhammer memo search "meeting standup"

# Search for technical terms
swissarmyhammer memo search "API authentication"
```

**Search features:**
- Case-insensitive matching
- Searches both titles and content
- Shows relevance scores when available
- Highlights matching terms in results

## Step 7: Updating Memos

Update a memo's content while keeping the title and metadata:

```bash
# Update the meeting notes with action item progress
swissarmyhammer memo update 01BRZ3NDEKTSV4RRFFQ69G5FAW -c "# Team Standup - January 15, 2024

## Attendees
- Alice (PM)
- Bob (Engineering) 
- Carol (Design)

## Updates
[... previous content ...]

## Action Items Progress (Updated)
- [x] Alice: Schedule stakeholder review (✅ Done - meeting set for Friday)
- [ ] Bob: Submit schema proposal (in progress)
- [ ] Carol: Share prototype with team (on track for Monday)

## Follow-up Notes
- Stakeholder review scheduled for Friday 2:00 PM
- Bob will present schema options tomorrow
- Carol's prototype looking great so far
"
```

**What happens:**
- Content is completely replaced
- Title stays the same
- `updated_at` timestamp is refreshed
- Original `created_at` and ID remain unchanged

## Step 8: Using Memos with AI Assistants

Export all your memo context for AI assistants:

```bash
swissarmyhammer memo context
```

This creates a formatted output perfect for AI consumption:

```
📄 All memo context (2 memos)

## Meeting Notes - Team Standup (ID: 01BRZ3NDEKTSV4RRFFQ69G5FAW)

Created: 2024-01-15 14:35
Updated: 2024-01-15 16:22

[... full memo content ...]

===

## Project Ideas (ID: 01ARZ3NDEKTSV4RRFFQ69G5FAV)

Created: 2024-01-15 14:32
Updated: 2024-01-15 14:32

[... full memo content ...]

===
```

**Use cases for AI integration:**
- Provide context for project discussions
- Ask AI to analyze meeting patterns
- Generate summaries or action item reports
- Get suggestions based on your notes

## Step 9: Working with Claude Code (MCP)

If you're using Claude Code, memoranda tools are available as MCP tools:

### Creating Memos via MCP
```json
{
  "tool": "memo_create",
  "arguments": {
    "title": "Claude Conversation Summary",
    "content": "# Discussion about API Design\n\n- Talked about REST vs GraphQL\n- Decided on pagination strategy\n- Need to implement rate limiting"
  }
}
```

### Searching via MCP
```json
{
  "tool": "memo_search", 
  "arguments": {
    "query": "API design pagination"
  }
}
```

### Getting All Context for AI
```json
{
  "tool": "memo_get_all_context",
  "arguments": {}
}
```

## Step 10: Practical Workflows

### Daily Journaling
```bash
# Morning routine
swissarmyhammer memo create "Daily Journal $(date +%Y-%m-%d)" -c "# Goals for $(date +%B %d)

## Work Tasks
- [ ] Review PRs
- [ ] Team standup
- [ ] Complete feature X

## Personal
- [ ] Exercise
- [ ] Read chapter 3

## Notes
Starting early today to tackle the complex refactoring."

# Evening reflection (update the same memo)
JOURNAL_ID=$(swissarmyhammer memo list | grep "Daily Journal $(date +%Y-%m-%d)" | grep -o '01[A-Z0-9]\{25\}')
swissarmyhammer memo update $JOURNAL_ID -c "# Daily Journal $(date +%B %d)

## Completed ✅
- [x] Reviewed 3 PRs
- [x] Attended standup
- [x] Made progress on feature X (70% done)
- [x] Exercise (30 min run)

## Tomorrow's Focus
- Finish feature X
- Start planning sprint review
- Research new testing framework"
```

### Code Snippet Collection
```bash
swissarmyhammer memo create "Rust Patterns" -c '# Useful Rust Patterns

## Error Handling with ?
```rust
fn read_file() -> Result<String, Box<dyn Error>> {
    let content = std::fs::read_to_string("file.txt")?;
    Ok(content.trim().to_string())
}
```

## Iterator Combinators
```rust
let numbers: Vec<i32> = vec![1, 2, 3, 4, 5]
    .iter()
    .filter(|&&x| x > 2)
    .map(|&x| x * 2)
    .collect();
```
'
```

### Meeting Templates
```bash
# Create reusable meeting template
swissarmyhammer memo create "Weekly Team Meeting Template" -c "# Weekly Team Meeting - [DATE]

## Attendees
- 
- 
- 

## Previous Action Items Review
- [ ] Item 1
- [ ] Item 2

## This Week's Updates
### [Team Member 1]
- Completed:
- Working on:
- Blockers:

### [Team Member 2] 
- Completed:
- Working on:
- Blockers:

## Upcoming Deadlines
- 

## New Action Items
- [ ] 
- [ ] 

## Next Meeting
- Date: 
- Topics: 
"

# Copy template for actual meetings
TEMPLATE_ID="[ID from above]"
swissarmyhammer memo get $TEMPLATE_ID | grep -A 100 "Content:" | tail -n +2 | \
  swissarmyhammer memo create "Weekly Team Meeting $(date +%Y-%m-%d)" -c -
```

## Step 11: Organization and Cleanup

### Archive Old Memos
```bash
# Search for old memos
swissarmyhammer memo search "2023"

# If you want to remove outdated memos
swissarmyhammer memo delete 01OLD3NDEKTSV4RRFFQ69G5FAV
```

### Backup Your Memos
```bash
# Create backup directory
mkdir -p backups/memos-$(date +%Y%m%d)

# Copy memo files
cp -r .swissarmyhammer/memos/ backups/memos-$(date +%Y%m%d)/

# Or export as markdown
swissarmyhammer memo context > backups/all-memos-$(date +%Y%m%d).md
```

### Search and Organization Tips
```bash
# Use specific terms for better search
swissarmyhammer memo search "standup meeting"        # ✅ Good
swissarmyhammer memo search "meeting"                # ⚠️ Too broad

# Create categorized memos
swissarmyhammer memo create "Work - Sprint Planning" -c "[content]"
swissarmyhammer memo create "Personal - Reading List" -c "[content]"
swissarmyhammer memo create "Learning - Docker Notes" -c "[content]"

# Search by category
swissarmyhammer memo search "Work -"
swissarmyhammer memo search "Learning -"
```

## Common Patterns and Best Practices

### 1. Descriptive Titles
```bash
# ✅ Good titles
swissarmyhammer memo create "API Design Meeting - 2024-01-15"
swissarmyhammer memo create "Bug Investigation - Auth Token Expiry"
swissarmyhammer memo create "Learning Notes - Async Rust Patterns"

# ❌ Poor titles
swissarmyhammer memo create "Notes"
swissarmyhammer memo create "Meeting"
swissarmyhammer memo create "Stuff"
```

### 2. Structured Content
```bash
swissarmyhammer memo create "Project Status" -c "# Project Alpha Status

## Current Sprint (Sprint 12)
- **Goal**: Implement user authentication
- **Progress**: 70% complete
- **Blockers**: Database migration pending

## Key Metrics
- **Test Coverage**: 85%
- **Performance**: API response < 100ms
- **User Feedback**: 4.2/5 rating

## Next Steps
1. Complete OAuth integration
2. Add password reset flow  
3. Performance optimization

## Risks
- External API dependency unstable
- Team member on vacation next week
"
```

### 3. Regular Maintenance
```bash
# Weekly memo review
swissarmyhammer memo search "action items"
swissarmyhammer memo search "follow up"
swissarmyhammer memo search "next week"

# Monthly cleanup
swissarmyhammer memo list | head -20  # Review recent memos
swissarmyhammer memo search "TODO"     # Find unfinished items
```

## Troubleshooting

### "Permission denied" error
```bash
# Check directory permissions
ls -la .swissarmyhammer/
chmod 755 .swissarmyhammer/
chmod 644 .swissarmyhammer/memos/*
```

### "Invalid memo ID format" error
```bash
# ULIDs are exactly 26 characters
swissarmyhammer memo get 01ARZ3NDEKTSV4RRFFQ69G5FAV  # ✅ Correct
swissarmyhammer memo get 01ARZ3                        # ❌ Too short
```

### Search returns no results
```bash
# Check if memos exist
swissarmyhammer memo list

# Try broader search terms
swissarmyhammer memo search ""  # Returns all memos
```

## What's Next?

Now that you know the basics, explore these advanced topics:

- **[Advanced Usage Examples](memoranda-advanced.md)** - Complex workflows and automation
- **[CLI Reference](../src/cli-memoranda.md)** - Complete command documentation  
- **[MCP Tools Guide](../src/mcp-memoranda.md)** - AI assistant integration
- **[API Documentation](../src/api-reference.md)** - Programmatic usage

### Integration Ideas
- Set up shell aliases for common memo operations
- Create git hooks to auto-generate memos for significant commits
- Build custom scripts for memo analytics and reporting
- Integrate with your editor for quick note capture

### Advanced Workflows
- Template-based memo generation
- Automated daily/weekly memo creation
- Cross-project memo management
- Team knowledge sharing workflows

## Summary

You've learned how to:
- ✅ Create memos with titles and content
- ✅ Browse and search your memo collection
- ✅ Update and manage memo content
- ✅ Export memos for AI assistant integration
- ✅ Organize memos for different workflows
- ✅ Handle common issues and troubleshooting

**Key takeaways:**
- Memos use ULID identifiers for chronological ordering
- Rich markdown content makes memos more useful
- Search is powerful for finding information quickly
- Integration with AI assistants enhances productivity
- Regular maintenance keeps your memo collection organized

Happy note-taking! 📝