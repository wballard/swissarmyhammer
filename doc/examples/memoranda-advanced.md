# Memoranda Advanced Usage Examples

Advanced patterns, automation techniques, and sophisticated workflows for power users of SwissArmyHammer's memoranda system.

## Table of Contents

- [Complex Search Queries](#complex-search-queries)
- [AI Assistant Integration Patterns](#ai-assistant-integration-patterns)
- [Automation and Scripting](#automation-and-scripting)
- [Team Collaboration Workflows](#team-collaboration-workflows)
- [Data Analysis and Reports](#data-analysis-and-reports)
- [Integration with Development Tools](#integration-with-development-tools)
- [Advanced Organization Strategies](#advanced-organization-strategies)
- [Performance Optimization](#performance-optimization)
- [Backup and Migration Strategies](#backup-and-migration-strategies)

## Complex Search Queries

### Multi-Term Search Strategies

```bash
# Search for multiple related concepts
swissarmyhammer memo search "API authentication security"
swissarmyhammer memo search "database migration performance"
swissarmyhammer memo search "frontend React component"

# Search for specific project phases
swissarmyhammer memo search "sprint planning retrospective"
swissarmyhammer memo search "code review feedback"

# Technical term combinations
swissarmyhammer memo search "async await rust"
swissarmyhammer memo search "docker kubernetes deployment"
```

### Time-Based Search Patterns

```bash
# Find recent memos by searching for current dates
swissarmyhammer memo search "2024-01"      # January 2024 memos
swissarmyhammer memo search "$(date +%Y-%m)"  # Current month
swissarmyhammer memo search "$(date +%Y)"     # Current year

# Search for specific meeting patterns
swissarmyhammer memo search "standup $(date +%B)"  # This month's standups
swissarmyhammer memo search "review Q$((($(date +%m)-1)/3+1))"  # Current quarter
```

### Category and Tag-Based Search

```bash
# Implement pseudo-tagging with prefixes
swissarmyhammer memo create "PROJ-Alpha: Database Design" -c "[content]"
swissarmyhammer memo create "LEARN-Rust: Async Programming" -c "[content]"
swissarmyhammer memo create "MEETING-Weekly: Team Sync" -c "[content]"

# Search by category
swissarmyhammer memo search "PROJ-Alpha"
swissarmyhammer memo search "LEARN-"
swissarmyhammer memo search "MEETING-Weekly"
```

### Content-Specific Search

```bash
# Search for action items and TODOs
swissarmyhammer memo search "- [ ]"
swissarmyhammer memo search "TODO:"
swissarmyhammer memo search "action item"
swissarmyhammer memo search "follow up"

# Search for code patterns
swissarmyhammer memo search "```rust"
swissarmyhammer memo search "```python"
swissarmyhammer memo search "function"
swissarmyhammer memo search "class"

# Search for decision records
swissarmyhammer memo search "decision:"
swissarmyhammer memo search "ADR-"
swissarmyhammer memo search "architecture"
```

## AI Assistant Integration Patterns

### Context-Aware Memo Creation

Create an AI assistant that automatically generates memos from conversations:

**MCP Integration Pattern:**
```javascript
// In your AI assistant workflow
async function createMemoFromConversation(topic, discussion) {
    const summary = await analyzeDiscussion(discussion);
    
    const memoContent = `# Discussion Summary: ${topic}

## Key Points
${summary.keyPoints.map(point => `- ${point}`).join('\n')}

## Decisions Made
${summary.decisions.map(decision => `- **${decision.topic}**: ${decision.resolution}`).join('\n')}

## Action Items
${summary.actionItems.map(item => `- [ ] ${item.task} (assigned to: ${item.assignee})`).join('\n')}

## Follow-up Questions
${summary.questions.map(q => `- ${q}`).join('\n')}

## Context
Generated from conversation on ${new Date().toISOString().split('T')[0]}
`;

    return await callTool('memo_create', {
        title: `Discussion: ${topic}`,
        content: memoContent
    });
}
```

### Smart Memo Retrieval

```javascript
// Contextual memo retrieval for AI responses
async function getRelevantMemos(userQuery, maxMemos = 5) {
    // First, try specific search
    let relevantMemos = await callTool('memo_search', {
        query: userQuery
    });
    
    if (relevantMemos.length === 0) {
        // Fall back to keyword extraction
        const keywords = extractKeywords(userQuery);
        for (const keyword of keywords) {
            const results = await callTool('memo_search', { query: keyword });
            relevantMemos = relevantMemos.concat(results);
        }
    }
    
    // Limit results and format for AI context
    return relevantMemos.slice(0, maxMemos).map(memo => ({
        title: memo.title,
        relevance: memo.relevance_score,
        content: memo.content.substring(0, 500) + '...'
    }));
}
```

### AI-Powered Memo Analysis

```javascript
// Analyze memo collection for insights
async function analyzeMemoTrends() {
    const allMemos = await callTool('memo_get_all_context', {});
    
    const analysis = await analyzeWithAI(allMemos, `
    Analyze this memo collection and provide:
    1. Most common themes and topics
    2. Recurring action items or blockers
    3. Project progress patterns
    4. Knowledge gaps that need attention
    5. Suggested improvements for note-taking
    `);
    
    // Create analysis memo
    return await callTool('memo_create', {
        title: `Memo Collection Analysis - ${new Date().toISOString().split('T')[0]}`,
        content: `# Memo Collection Analysis

${analysis}

## Recommendations
${analysis.recommendations.map(r => `- ${r}`).join('\n')}

## Action Items
${analysis.actionItems.map(a => `- [ ] ${a}`).join('\n')}
`
    });
}
```

## Automation and Scripting

### Daily Memo Automation

```bash
#!/bin/bash
# daily-memo-setup.sh - Create daily memo template

DATE=$(date +%Y-%m-%d)
DAY_NAME=$(date +%A)

swissarmyhammer memo create "Daily Log - $DATE" -c "# Daily Log - $DAY_NAME, $DATE

## Morning Planning
- [ ] Review calendar and priorities
- [ ] Check action items from yesterday
- [ ] Set daily goals

## Work Focus Areas
### Priority 1
- [ ] 

### Priority 2  
- [ ] 

### Priority 3
- [ ] 

## Meetings Today
- 

## Learning/Reading
- 

## Evening Reflection
### What went well?
- 

### What could be improved?
- 

### Tomorrow's priorities
- [ ] 
- [ ] 
- [ ] 

## Random Notes
- 
"

echo "Created daily log for $DATE"

# Optional: Open in your preferred editor
# code .swissarmyhammer/memos/$(swissarmyhammer memo list | head -2 | tail -1 | grep -o '01[A-Z0-9]\{25\}').json
```

### Git Integration

```bash
#!/bin/bash
# git-memo-hook.sh - Post-commit hook for significant changes

COMMIT_MSG=$(git log -1 --pretty=%B)
COMMIT_HASH=$(git rev-parse --short HEAD)
BRANCH=$(git branch --show-current)

# Create memo for significant commits
if [[ "$COMMIT_MSG" == *"feat:"* ]] || [[ "$COMMIT_MSG" == *"BREAKING:"* ]] || [[ "$COMMIT_MSG" == *"refactor:"* ]]; then
    
    FILES_CHANGED=$(git show --stat $COMMIT_HASH | head -n -1)
    DIFF_SUMMARY=$(git show --stat --oneline $COMMIT_HASH)
    
    swissarmyhammer memo create "Git Commit - $COMMIT_HASH" -c "# Significant Commit: $COMMIT_HASH

## Branch
$BRANCH

## Commit Message
\`$COMMIT_MSG\`

## Files Changed
\`\`\`
$FILES_CHANGED
\`\`\`

## Summary
\`\`\`
$DIFF_SUMMARY
\`\`\`

## Context
- Date: $(date)
- Author: $(git config user.name)
- Branch: $BRANCH

## Related
- [ ] Update documentation if needed
- [ ] Consider migration requirements
- [ ] Notify team of changes
"

    echo "Created memo for commit $COMMIT_HASH"
fi
```

### Automated Meeting Notes

```python
#!/usr/bin/env python3
# meeting-memo-creator.py - Create structured meeting notes

import subprocess
import sys
from datetime import datetime, timedelta
import json

def create_meeting_memo(title, attendees, agenda_items):
    date_str = datetime.now().strftime("%Y-%m-%d")
    time_str = datetime.now().strftime("%H:%M")
    
    # Generate attendee list
    attendee_list = "\n".join([f"- {attendee}" for attendee in attendees])
    
    # Generate agenda
    agenda_list = "\n".join([f"{i+1}. {item}" for i, item in enumerate(agenda_items)])
    
    content = f"""# {title} - {date_str}

## Meeting Details
- **Date**: {date_str}
- **Time**: {time_str}
- **Duration**: [TBD]

## Attendees
{attendee_list}

## Agenda
{agenda_list}

## Discussion Notes

### Item 1: {agenda_items[0] if agenda_items else '[Topic]'}
- **Discussion**: 
- **Decisions**: 
- **Action Items**: 
  - [ ] 

### Item 2: {agenda_items[1] if len(agenda_items) > 1 else '[Topic]'}
- **Discussion**: 
- **Decisions**: 
- **Action Items**: 
  - [ ] 

## Overall Action Items
- [ ] 
- [ ] 

## Next Steps
- **Next Meeting**: 
- **Follow-up Required**: 

## Post-Meeting Notes
- **Key Outcomes**: 
- **Blockers Identified**: 
- **Decisions That Need Approval**: 
"""

    # Create the memo
    cmd = ["swissarmyhammer", "memo", "create", title, "-c", content]
    result = subprocess.run(cmd, capture_output=True, text=True)
    
    if result.returncode == 0:
        print(f"✅ Created meeting memo: {title}")
        print(result.stdout)
        
        # Extract memo ID for follow-up
        lines = result.stdout.split('\n')
        for line in lines:
            if "🆔 ID:" in line:
                memo_id = line.split(": ")[1].strip()
                print(f"📝 Memo ID for updates: {memo_id}")
                break
    else:
        print(f"❌ Error creating memo: {result.stderr}")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 meeting-memo-creator.py 'Meeting Title' attendee1,attendee2 'agenda1,agenda2'")
        sys.exit(1)
    
    title = sys.argv[1]
    attendees = sys.argv[2].split(',') if len(sys.argv) > 2 else []
    agenda_items = sys.argv[3].split(',') if len(sys.argv) > 3 else []
    
    create_meeting_memo(title, attendees, agenda_items)
```

### Bulk Operations Script

```bash
#!/bin/bash
# memo-bulk-operations.sh - Batch operations on memos

# Function to backup all memos
backup_memos() {
    local backup_dir="memo-backup-$(date +%Y%m%d-%H%M%S)"
    mkdir -p "$backup_dir"
    
    # Copy memo files
    cp -r .swissarmyhammer/memos/ "$backup_dir/"
    
    # Export context
    swissarmyhammer memo context > "$backup_dir/all-memos.md"
    
    echo "✅ Backup created in $backup_dir"
}

# Function to find and archive old memos
archive_old_memos() {
    local cutoff_date=$1  # Format: YYYY-MM-DD
    local archive_queries=("2023-" "old" "archive" "deprecated")
    
    echo "🔍 Searching for memos to archive (older than $cutoff_date)..."
    
    for query in "${archive_queries[@]}"; do
        echo "Searching for: $query"
        swissarmyhammer memo search "$query"
    done
    
    read -p "Continue with archiving? (y/N) " -n 1 -r
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "🗂️ Archive process would begin here"
        # Implementation would go here
    fi
}

# Function to generate memo statistics
memo_stats() {
    local total_memos=$(swissarmyhammer memo list | grep "🆔" | wc -l)
    local total_context=$(swissarmyhammer memo context | wc -w)
    
    echo "📊 Memo Collection Statistics"
    echo "================================"
    echo "Total Memos: $total_memos"
    echo "Total Words: $total_context"
    echo "Average Words per Memo: $((total_context / total_memos))"
    echo
    
    echo "📈 Most Common Terms:"
    swissarmyhammer memo context | tr ' ' '\n' | tr '[:upper:]' '[:lower:]' | \
        grep -v '^$' | sort | uniq -c | sort -nr | head -10
}

# Function to find potential duplicates
find_duplicates() {
    echo "🔍 Searching for potential duplicate memos..."
    
    # Extract all memo titles
    swissarmyhammer memo list | grep "📄" | sed 's/📄 //' > /tmp/memo_titles.txt
    
    # Find similar titles
    sort /tmp/memo_titles.txt | uniq -d
    
    # Look for similar content patterns
    echo "Content pattern analysis:"
    swissarmyhammer memo search "meeting notes"
    swissarmyhammer memo search "daily"
    swissarmyhammer memo search "standup"
}

# Main menu
case "$1" in
    "backup")
        backup_memos
        ;;
    "archive")
        archive_old_memos "${2:-2023-12-31}"
        ;;
    "stats")
        memo_stats
        ;;
    "duplicates")
        find_duplicates
        ;;
    *)
        echo "Usage: $0 {backup|archive|stats|duplicates} [options]"
        echo "  backup           - Create full backup of memos"
        echo "  archive [date]   - Archive old memos (default: before 2024)"
        echo "  stats            - Display collection statistics"
        echo "  duplicates       - Find potential duplicate memos"
        ;;
esac
```

## Team Collaboration Workflows

### Shared Knowledge Base

```bash
#!/bin/bash
# team-knowledge-sync.sh - Sync team knowledge base

TEAM_REPO="git@github.com:yourteam/team-memos.git"
LOCAL_BACKUP="team-memos-backup"

# Function to export team-relevant memos
export_team_memos() {
    echo "📤 Exporting team-relevant memos..."
    
    # Search for team-related content
    swissarmyhammer memo search "team meeting" > team-meetings.md
    swissarmyhammer memo search "project decision" > team-decisions.md
    swissarmyhammer memo search "architecture" > team-architecture.md
    swissarmyhammer memo search "onboarding" > team-onboarding.md
    
    # Create index
    cat > team-index.md << EOF
# Team Knowledge Base - $(date +%Y-%m-%d)

## Categories
- [Team Meetings](team-meetings.md)
- [Project Decisions](team-decisions.md)
- [Architecture Docs](team-architecture.md)
- [Onboarding Info](team-onboarding.md)

## Last Updated
$(date)
EOF
    
    echo "✅ Team memos exported"
}

# Function to create team memo templates
create_team_templates() {
    # Architecture Decision Record template
    swissarmyhammer memo create "TEMPLATE-ADR" -c "# ADR-[NUMBER]: [Title]

## Status
[Proposed | Accepted | Deprecated | Superseded by ADR-XXX]

## Context
What is the issue that we're seeing that is motivating this decision or change?

## Decision
What is the change that we're proposing and/or doing?

## Consequences
What becomes easier or more difficult to do because of this change?

## Alternatives Considered
What other alternatives were considered?

## Implementation Notes
- [ ] Update documentation
- [ ] Migrate existing code
- [ ] Notify stakeholders
"

    # Team Meeting template
    swissarmyhammer memo create "TEMPLATE-Team-Meeting" -c "# Team Meeting - [DATE]

## Attendees
- [ ] Alice (PM)
- [ ] Bob (Engineering)
- [ ] Carol (Design)

## Agenda
1. Previous action items review
2. Sprint progress update
3. Blockers discussion
4. Upcoming milestones

## Discussion

### Previous Action Items
- [ ] Item 1 - Status
- [ ] Item 2 - Status

### Sprint Progress
**Current Sprint**: Sprint [N]
**Goal**: [Sprint Goal]
**Progress**: [X]% complete

#### Individual Updates
**Alice**:
- Completed: 
- In Progress: 
- Blockers: 

**Bob**:
- Completed: 
- In Progress: 
- Blockers: 

**Carol**:
- Completed: 
- In Progress: 
- Blockers: 

## Action Items
- [ ] [Action] - [Assignee] - [Due Date]
- [ ] [Action] - [Assignee] - [Due Date]

## Next Meeting
- **Date**: 
- **Focus**: 
"

    echo "✅ Team templates created"
}

case "$1" in
    "export")
        export_team_memos
        ;;
    "templates")
        create_team_templates
        ;;
    "sync")
        export_team_memos
        # Additional sync logic would go here
        echo "📤 Would sync to $TEAM_REPO"
        ;;
    *)
        echo "Usage: $0 {export|templates|sync}"
        ;;
esac
```

### Code Review Memo Integration

```bash
#!/bin/bash
# code-review-memo.sh - Create memos from code reviews

PR_NUMBER="$1"
if [ -z "$PR_NUMBER" ]; then
    echo "Usage: $0 <PR_NUMBER>"
    exit 1
fi

# Get PR information (assumes gh CLI is installed)
PR_TITLE=$(gh pr view $PR_NUMBER --json title -q .title)
PR_AUTHOR=$(gh pr view $PR_NUMBER --json author -q .author.login)
PR_URL=$(gh pr view $PR_NUMBER --json url -q .url)

# Get review comments
REVIEWS=$(gh pr view $PR_NUMBER --json reviews -q '.reviews[] | "**" + .author.login + "** (" + .state + "):\n" + .body + "\n"')

# Get file changes
FILES_CHANGED=$(gh pr diff $PR_NUMBER --name-only)

swissarmyhammer memo create "Code Review - PR #$PR_NUMBER" -c "# Code Review: $PR_TITLE

## PR Details
- **Number**: #$PR_NUMBER
- **Author**: $PR_AUTHOR  
- **URL**: $PR_URL
- **Date**: $(date +%Y-%m-%d)

## Files Changed
\`\`\`
$FILES_CHANGED
\`\`\`

## Review Comments
$REVIEWS

## Key Discussion Points
- 

## Decisions Made
- 

## Follow-up Actions
- [ ] 

## Learnings
- 

## Related Issues
- 
"

echo "✅ Created code review memo for PR #$PR_NUMBER"
```

## Data Analysis and Reports

### Memo Analytics Script

```python
#!/usr/bin/env python3
# memo-analytics.py - Analyze memo collection for insights

import subprocess
import json
import re
from collections import Counter, defaultdict
from datetime import datetime, timedelta
import matplotlib.pyplot as plt
import pandas as pd

class MemoAnalytics:
    def __init__(self):
        self.memos = self.load_all_memos()
    
    def load_all_memos(self):
        """Load all memos using SwissArmyHammer CLI"""
        result = subprocess.run(['swissarmyhammer', 'memo', 'context'], 
                              capture_output=True, text=True)
        
        if result.returncode != 0:
            raise Exception(f"Failed to load memos: {result.stderr}")
        
        return self.parse_context_output(result.stdout)
    
    def parse_context_output(self, context_text):
        """Parse the context output into structured data"""
        memos = []
        current_memo = None
        
        for line in context_text.split('\n'):
            if line.startswith('## ') and '(ID:' in line:
                if current_memo:
                    memos.append(current_memo)
                
                # Extract title and ID
                title_id = line[3:]  # Remove '## '
                title, id_part = title_id.rsplit(' (ID:', 1)
                memo_id = id_part.rstrip(')')
                
                current_memo = {
                    'id': memo_id,
                    'title': title,
                    'content': '',
                    'created_at': None,
                    'updated_at': None
                }
            elif line.startswith('Created:') and current_memo:
                current_memo['created_at'] = self.parse_date(line[9:])
            elif line.startswith('Updated:') and current_memo:
                current_memo['updated_at'] = self.parse_date(line[9:])
            elif current_memo and line not in ['===', '']:
                current_memo['content'] += line + '\n'
        
        if current_memo:
            memos.append(current_memo)
        
        return memos
    
    def parse_date(self, date_str):
        """Parse date string from memo context"""
        try:
            return datetime.strptime(date_str.strip(), '%Y-%m-%d %H:%M')
        except ValueError:
            return None
    
    def analyze_creation_patterns(self):
        """Analyze memo creation patterns over time"""
        creation_dates = [memo['created_at'] for memo in self.memos if memo['created_at']]
        
        # Group by day of week
        day_counts = Counter(date.strftime('%A') for date in creation_dates)
        
        # Group by hour of day
        hour_counts = Counter(date.hour for date in creation_dates)
        
        print("📊 Memo Creation Patterns")
        print("=" * 50)
        print("By Day of Week:")
        for day, count in day_counts.most_common():
            print(f"  {day}: {count} memos")
        
        print("\nBy Hour of Day:")
        for hour, count in sorted(hour_counts.items()):
            bar = '█' * (count * 20 // max(hour_counts.values()))
            print(f"  {hour:2d}:00 {bar} ({count})")
    
    def analyze_content_themes(self):
        """Analyze common themes in memo content"""
        all_content = ' '.join(memo['content'].lower() for memo in self.memos)
        
        # Extract words (simple tokenization)
        words = re.findall(r'\b\w+\b', all_content)
        
        # Filter out common stop words
        stop_words = {'the', 'a', 'an', 'and', 'or', 'but', 'in', 'on', 'at', 
                     'to', 'for', 'of', 'with', 'by', 'is', 'are', 'was', 'were',
                     'be', 'been', 'have', 'has', 'had', 'do', 'does', 'did',
                     'will', 'would', 'could', 'should', 'this', 'that', 'these',
                     'those', 'i', 'you', 'he', 'she', 'it', 'we', 'they'}
        
        filtered_words = [word for word in words if len(word) > 3 and word not in stop_words]
        
        word_counts = Counter(filtered_words)
        
        print("\n🏷️ Most Common Themes")
        print("=" * 50)
        for word, count in word_counts.most_common(20):
            print(f"  {word}: {count} occurrences")
    
    def analyze_title_patterns(self):
        """Analyze patterns in memo titles"""
        titles = [memo['title'] for memo in self.memos]
        
        # Look for common prefixes
        prefixes = defaultdict(int)
        for title in titles:
            parts = title.split(' ')
            if len(parts) > 1:
                prefixes[parts[0]] += 1
        
        # Look for common patterns
        patterns = {
            'meetings': len([t for t in titles if any(word in t.lower() for word in ['meeting', 'standup', 'sync'])]),
            'projects': len([t for t in titles if any(word in t.lower() for word in ['project', 'feature', 'epic'])]),
            'learning': len([t for t in titles if any(word in t.lower() for word in ['learn', 'tutorial', 'guide'])]),
            'bugs': len([t for t in titles if any(word in t.lower() for word in ['bug', 'fix', 'issue', 'problem'])]),
            'ideas': len([t for t in titles if any(word in t.lower() for word in ['idea', 'thought', 'brainstorm'])])
        }
        
        print("\n📝 Title Patterns")
        print("=" * 50)
        print("Common Categories:")
        for category, count in sorted(patterns.items(), key=lambda x: x[1], reverse=True):
            percentage = (count / len(titles)) * 100
            print(f"  {category.title()}: {count} ({percentage:.1f}%)")
        
        print(f"\nCommon Prefixes (>1 occurrence):")
        for prefix, count in sorted(prefixes.items(), key=lambda x: x[1], reverse=True):
            if count > 1:
                print(f"  '{prefix}': {count} memos")
    
    def analyze_productivity_patterns(self):
        """Analyze productivity patterns based on memo creation"""
        creation_dates = [memo['created_at'] for memo in self.memos if memo['created_at']]
        
        if not creation_dates:
            return
        
        # Group by date
        daily_counts = defaultdict(int)
        for date in creation_dates:
            daily_counts[date.date()] += 1
        
        # Calculate streaks
        sorted_dates = sorted(daily_counts.keys())
        current_streak = 0
        longest_streak = 0
        
        for i, date in enumerate(sorted_dates):
            if i == 0 or (date - sorted_dates[i-1]).days == 1:
                current_streak += 1
            else:
                longest_streak = max(longest_streak, current_streak)
                current_streak = 1
        
        longest_streak = max(longest_streak, current_streak)
        
        print("\n⚡ Productivity Patterns")
        print("=" * 50)
        print(f"Total memo days: {len(daily_counts)}")
        print(f"Average memos per active day: {sum(daily_counts.values()) / len(daily_counts):.1f}")
        print(f"Most productive day: {max(daily_counts.items(), key=lambda x: x[1])[1]} memos")
        print(f"Longest streak: {longest_streak} consecutive days")
    
    def generate_report(self):
        """Generate comprehensive analytics report"""
        report_content = f"""# Memo Collection Analytics Report
Generated on: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

## Summary Statistics
- **Total Memos**: {len(self.memos)}
- **Date Range**: {min(memo['created_at'] for memo in self.memos if memo['created_at'])} to {max(memo['created_at'] for memo in self.memos if memo['created_at'])}
- **Average Title Length**: {sum(len(memo['title']) for memo in self.memos) / len(self.memos):.1f} characters
- **Average Content Length**: {sum(len(memo['content']) for memo in self.memos) / len(self.memos):.0f} characters

## Analysis Results
This report was generated automatically by analyzing your memo collection.
"""
        
        # Create the report memo
        subprocess.run([
            'swissarmyhammer', 'memo', 'create',
            f"Analytics Report - {datetime.now().strftime('%Y-%m-%d')}",
            '-c', report_content
        ])
        
        print(f"📊 Analytics report created as memo")

def main():
    analytics = MemoAnalytics()
    
    print(f"📚 Analyzing {len(analytics.memos)} memos...\n")
    
    analytics.analyze_creation_patterns()
    analytics.analyze_content_themes()
    analytics.analyze_title_patterns()
    analytics.analyze_productivity_patterns()
    analytics.generate_report()

if __name__ == "__main__":
    main()
```

## Integration with Development Tools

### VSCode Extension Integration

```json
// .vscode/tasks.json - VSCode task integration
{
    "version": "2.0.0",
    "tasks": [
        {
            "label": "Create Memo",
            "type": "shell",
            "command": "swissarmyhammer",
            "args": ["memo", "create", "${input:memoTitle}", "-c", "${input:memoContent}"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": false,
                "panel": "shared"
            }
        },
        {
            "label": "Search Memos",
            "type": "shell",
            "command": "swissarmyhammer",
            "args": ["memo", "search", "${input:searchQuery}"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always",
                "focus": true,
                "panel": "shared"
            }
        },
        {
            "label": "Create Code Review Memo",
            "type": "shell",
            "command": "${workspaceFolder}/scripts/create-review-memo.sh",
            "args": ["${input:prNumber}"],
            "group": "build",
            "presentation": {
                "echo": true,
                "reveal": "always"
            }
        }
    ],
    "inputs": [
        {
            "id": "memoTitle",
            "description": "Memo title",
            "default": "Quick Note",
            "type": "promptString"
        },
        {
            "id": "memoContent",
            "description": "Memo content",
            "default": "",
            "type": "promptString"
        },
        {
            "id": "searchQuery",
            "description": "Search query",
            "type": "promptString"
        },
        {
            "id": "prNumber",
            "description": "Pull Request Number",
            "type": "promptString"
        }
    ]
}
```

### GitHub Actions Integration

```yaml
# .github/workflows/memo-automation.yml
name: Memo Automation

on:
  pull_request:
    types: [opened, closed]
  issues:
    types: [opened, closed]

jobs:
  create-memos:
    runs-on: ubuntu-latest
    steps:
    - name: Checkout
      uses: actions/checkout@v3
    
    - name: Install SwissArmyHammer
      run: |
        cargo install --git https://github.com/wballard/swissarmyhammer.git swissarmyhammer-cli
    
    - name: Create PR Memo
      if: github.event_name == 'pull_request' && github.event.action == 'opened'
      run: |
        swissarmyhammer memo create "PR #${{ github.event.number }}: ${{ github.event.pull_request.title }}" -c "# Pull Request #${{ github.event.number }}

        ## Details
        - **Title**: ${{ github.event.pull_request.title }}
        - **Author**: ${{ github.event.pull_request.user.login }}
        - **URL**: ${{ github.event.pull_request.html_url }}
        - **Created**: $(date)

        ## Description
        ${{ github.event.pull_request.body }}

        ## Files Changed
        $(gh pr diff ${{ github.event.number }} --name-only)

        ## Status
        - [ ] Review in progress
        - [ ] Tests passing
        - [ ] Ready for merge
        "
    
    - name: Archive Completed PR
      if: github.event_name == 'pull_request' && github.event.action == 'closed'
      run: |
        # Search for existing PR memo and update it
        MEMO_SEARCH=$(swissarmyhammer memo search "PR #${{ github.event.number }}")
        if [ ! -z "$MEMO_SEARCH" ]; then
          echo "Updating existing PR memo with completion status"
          # Implementation would extract memo ID and update it
        fi
```

### Jupyter Notebook Integration

```python
# memo_jupyter_extension.py - Jupyter notebook extension for memos

import subprocess
import json
from IPython.core.magic import Magics, magics_class, line_magic, cell_magic

@magics_class
class MemoMagics(Magics):
    
    @line_magic
    def memo_create(self, line):
        """Create a memo from command line
        Usage: %memo_create "Title" "Content"
        """
        parts = line.split('"')
        if len(parts) >= 4:
            title = parts[1]
            content = parts[3]
            
            result = subprocess.run([
                'swissarmyhammer', 'memo', 'create', title, '-c', content
            ], capture_output=True, text=True)
            
            if result.returncode == 0:
                print(f"✅ Created memo: {title}")
            else:
                print(f"❌ Error: {result.stderr}")
    
    @cell_magic
    def memo_cell(self, line, cell):
        """Create a memo from cell content
        Usage: 
        %%memo_cell "Title"
        Cell content becomes memo content
        """
        title = line.strip('"')
        
        # Add notebook context
        content = f"# Jupyter Notebook Memo\n\nFrom notebook session on {datetime.now().strftime('%Y-%m-%d %H:%M')}\n\n{cell}"
        
        result = subprocess.run([
            'swissarmyhammer', 'memo', 'create', title, '-c', content
        ], capture_output=True, text=True)
        
        if result.returncode == 0:
            print(f"✅ Created memo: {title}")
            # Execute the cell content as well
            self.shell.run_cell(cell)
        else:
            print(f"❌ Error: {result.stderr}")
    
    @line_magic
    def memo_search(self, line):
        """Search memos and display results
        Usage: %memo_search "search terms"
        """
        query = line.strip('"')
        
        result = subprocess.run([
            'swissarmyhammer', 'memo', 'search', query
        ], capture_output=True, text=True)
        
        if result.returncode == 0:
            print(result.stdout)
        else:
            print(f"❌ Error: {result.stderr}")

# Load the extension
def load_ipython_extension(ipython):
    ipython.register_magic_functions(MemoMagics)

# Usage in notebook:
# %load_ext memo_jupyter_extension
# %memo_create "Analysis Results" "Found interesting correlation in dataset"
# %%memo_cell "Code Snippet"
# def analyze_data(df):
#     return df.groupby('category').mean()
```

## Performance Optimization

### Large Collection Management

```bash
#!/bin/bash
# memo-performance-optimizer.sh - Optimize performance for large collections

# Function to analyze collection size
analyze_collection_size() {
    local memo_dir=".swissarmyhammer/memos"
    
    if [ ! -d "$memo_dir" ]; then
        echo "❌ Memo directory not found"
        return 1
    fi
    
    local total_files=$(find "$memo_dir" -name "*.json" | wc -l)
    local total_size=$(du -sh "$memo_dir" | cut -f1)
    local avg_size=$(du -s "$memo_dir"/*.json 2>/dev/null | awk '{sum+=$1} END {print sum/NR}' | xargs -I {} echo "scale=2; {}/1024" | bc)
    
    echo "📊 Collection Analysis"
    echo "====================="
    echo "Total memos: $total_files"
    echo "Total size: $total_size"
    echo "Average memo size: ${avg_size}KB"
    
    # Identify large memos
    echo -e "\n📏 Largest memos:"
    find "$memo_dir" -name "*.json" -exec du -k {} + | sort -nr | head -5 | \
    while read size file; do
        echo "  ${size}KB - $(basename "$file" .json)"
    done
}

# Function to optimize search performance
optimize_search() {
    echo "🔍 Search Performance Optimization"
    echo "=================================="
    
    # Create search index for common terms
    local index_file=".swissarmyhammer/search_index.txt"
    
    echo "Building search index..."
    swissarmyhammer memo context | tr '[:upper:]' '[:lower:]' | \
        grep -oE '\w+' | sort | uniq -c | sort -nr > "$index_file"
    
    echo "✅ Search index created: $index_file"
    
    # Show most common terms
    echo -e "\nMost frequent terms:"
    head -10 "$index_file" | while read count term; do
        echo "  $term ($count occurrences)"
    done
}

# Function to archive old memos
archive_old_memos() {
    local cutoff_months=${1:-12}  # Archive memos older than X months
    local archive_dir="archive-$(date +%Y%m%d)"
    
    echo "📦 Archiving memos older than $cutoff_months months"
    
    mkdir -p "$archive_dir"
    
    # Find old memos (this is a simplified approach)
    # In a real implementation, you'd parse memo dates more precisely
    find .swissarmyhammer/memos -name "*.json" -mtime +$((cutoff_months * 30)) -exec mv {} "$archive_dir/" \;
    
    local archived_count=$(ls -1 "$archive_dir" 2>/dev/null | wc -l)
    echo "✅ Archived $archived_count memos to $archive_dir"
}

# Function to compress memo content
compress_memos() {
    echo "🗜️ Compressing memo storage"
    
    # Create compressed backup
    tar -czf "memos-backup-$(date +%Y%m%d).tar.gz" .swissarmyhammer/memos/
    
    echo "✅ Created compressed backup"
    
    # Suggestion for content optimization
    echo -e "\n💡 Content Optimization Suggestions:"
    echo "  - Remove duplicate whitespace"
    echo "  - Compress old memo content"
    echo "  - Consider using memo summaries for old entries"
}

# Main menu
case "$1" in
    "analyze")
        analyze_collection_size
        ;;
    "optimize-search")
        optimize_search
        ;;
    "archive")
        archive_old_memos "${2:-12}"
        ;;
    "compress")
        compress_memos
        ;;
    "all")
        analyze_collection_size
        optimize_search
        archive_old_memos 12
        compress_memos
        ;;
    *)
        echo "Usage: $0 {analyze|optimize-search|archive|compress|all}"
        echo "  analyze         - Analyze collection size and performance"
        echo "  optimize-search - Build search indexes and optimize"
        echo "  archive [months] - Archive memos older than N months (default: 12)"
        echo "  compress        - Create compressed backup"
        echo "  all             - Run all optimization steps"
        ;;
esac
```

## Backup and Migration Strategies

### Comprehensive Backup System

```bash
#!/bin/bash
# memo-backup-system.sh - Comprehensive backup and recovery system

BACKUP_BASE_DIR="$HOME/.memo-backups"
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_DIR="$BACKUP_BASE_DIR/$DATE"

# Function to create full backup
create_full_backup() {
    echo "🔄 Creating full backup..."
    
    mkdir -p "$BACKUP_DIR"
    
    # Copy memo files
    if [ -d ".swissarmyhammer/memos" ]; then
        cp -r .swissarmyhammer/memos/ "$BACKUP_DIR/memos/"
        echo "✅ Copied memo files"
    fi
    
    # Export context
    swissarmyhammer memo context > "$BACKUP_DIR/all-memos.md" 2>/dev/null
    echo "✅ Exported memo context"
    
    # Create manifest
    cat > "$BACKUP_DIR/manifest.json" << EOF
{
    "backup_date": "$(date -Iseconds)",
    "memo_count": $(find "$BACKUP_DIR/memos" -name "*.json" 2>/dev/null | wc -l),
    "total_size": "$(du -sh "$BACKUP_DIR" | cut -f1)",
    "swissarmyhammer_version": "$(swissarmyhammer --version 2>/dev/null || echo 'unknown')",
    "backup_type": "full"
}
EOF
    
    # Create archive
    cd "$BACKUP_BASE_DIR"
    tar -czf "${DATE}-full-backup.tar.gz" "$DATE/"
    
    echo "✅ Full backup completed: $BACKUP_DIR"
    echo "📦 Archive created: $BACKUP_BASE_DIR/${DATE}-full-backup.tar.gz"
}

# Function to create incremental backup
create_incremental_backup() {
    local last_backup=$(ls -1 "$BACKUP_BASE_DIR" | grep "^20" | tail -1)
    
    if [ -z "$last_backup" ]; then
        echo "❌ No previous backup found. Creating full backup instead."
        create_full_backup
        return
    fi
    
    echo "🔄 Creating incremental backup from $last_backup..."
    
    mkdir -p "$BACKUP_DIR"
    
    # Find changed memos since last backup
    find .swissarmyhammer/memos -name "*.json" -newer "$BACKUP_BASE_DIR/$last_backup" -exec cp {} "$BACKUP_DIR/" \;
    
    local changed_count=$(ls -1 "$BACKUP_DIR" 2>/dev/null | wc -l)
    
    # Create manifest
    cat > "$BACKUP_DIR/manifest.json" << EOF
{
    "backup_date": "$(date -Iseconds)",
    "backup_type": "incremental",
    "base_backup": "$last_backup",
    "changed_memos": $changed_count,
    "total_size": "$(du -sh "$BACKUP_DIR" | cut -f1)"
}
EOF
    
    echo "✅ Incremental backup completed: $changed_count changed memos"
}

# Function to restore from backup
restore_backup() {
    local backup_name="$1"
    
    if [ -z "$backup_name" ]; then
        echo "Available backups:"
        ls -1 "$BACKUP_BASE_DIR" | grep "^20"
        read -p "Enter backup name to restore: " backup_name
    fi
    
    local backup_path="$BACKUP_BASE_DIR/$backup_name"
    
    if [ ! -d "$backup_path" ]; then
        echo "❌ Backup not found: $backup_path"
        return 1
    fi
    
    echo "⚠️  This will overwrite current memos. Continue? (y/N)"
    read -n 1 -r
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Cancelled."
        return 0
    fi
    
    echo -e "\n🔄 Restoring from backup..."
    
    # Backup current state first
    if [ -d ".swissarmyhammer/memos" ]; then
        mv .swissarmyhammer/memos .swissarmyhammer/memos.backup.$(date +%Y%m%d-%H%M%S)
        echo "✅ Current memos backed up"
    fi
    
    # Restore from backup
    mkdir -p .swissarmyhammer/
    cp -r "$backup_path/memos" .swissarmyhammer/
    
    echo "✅ Backup restored from $backup_name"
}

# Function to list all backups
list_backups() {
    echo "📋 Available Backups"
    echo "==================="
    
    for backup in $(ls -1 "$BACKUP_BASE_DIR" | grep "^20" | sort -r); do
        local manifest="$BACKUP_BASE_DIR/$backup/manifest.json"
        if [ -f "$manifest" ]; then
            local backup_date=$(jq -r '.backup_date' "$manifest" 2>/dev/null || echo "unknown")
            local memo_count=$(jq -r '.memo_count // .changed_memos' "$manifest" 2>/dev/null || echo "unknown")
            local backup_type=$(jq -r '.backup_type' "$manifest" 2>/dev/null || echo "full")
            local total_size=$(jq -r '.total_size' "$manifest" 2>/dev/null || echo "unknown")
            
            echo "$backup - $backup_type ($memo_count memos, $total_size) - $backup_date"
        else
            echo "$backup - legacy backup"
        fi
    done
}

# Function to verify backup integrity
verify_backup() {
    local backup_name="$1"
    local backup_path="$BACKUP_BASE_DIR/$backup_name"
    
    if [ ! -d "$backup_path" ]; then
        echo "❌ Backup not found: $backup_path"
        return 1
    fi
    
    echo "🔍 Verifying backup: $backup_name"
    
    # Check manifest
    if [ -f "$backup_path/manifest.json" ]; then
        echo "✅ Manifest found"
        jq . "$backup_path/manifest.json" | head -10
    else
        echo "⚠️  No manifest found"
    fi
    
    # Check memo files
    local memo_count=$(find "$backup_path/memos" -name "*.json" 2>/dev/null | wc -l)
    echo "📁 Found $memo_count memo files"
    
    # Check context export
    if [ -f "$backup_path/all-memos.md" ]; then
        local context_size=$(wc -l < "$backup_path/all-memos.md")
        echo "📄 Context export: $context_size lines"
    else
        echo "⚠️  No context export found"
    fi
    
    # Sample file integrity check
    echo "🔍 Sample file integrity:"
    find "$backup_path/memos" -name "*.json" | head -3 | while read file; do
        if jq . "$file" >/dev/null 2>&1; then
            echo "  ✅ $(basename "$file")"
        else
            echo "  ❌ $(basename "$file") - corrupted JSON"
        fi
    done
}

# Function to cleanup old backups
cleanup_backups() {
    local keep_days=${1:-30}
    
    echo "🧹 Cleaning up backups older than $keep_days days..."
    
    find "$BACKUP_BASE_DIR" -maxdepth 1 -type d -name "20*" -mtime +$keep_days | while read old_backup; do
        echo "Removing: $(basename "$old_backup")"
        rm -rf "$old_backup"
    done
    
    # Also cleanup old tar.gz files
    find "$BACKUP_BASE_DIR" -name "*.tar.gz" -mtime +$keep_days -delete
    
    echo "✅ Cleanup completed"
}

# Migration function
migrate_memos() {
    local source_dir="$1"
    local target_dir="${2:-.swissarmyhammer/memos}"
    
    if [ -z "$source_dir" ]; then
        echo "Usage: migrate_memos <source_directory> [target_directory]"
        return 1
    fi
    
    echo "🚀 Migrating memos from $source_dir to $target_dir"
    
    mkdir -p "$target_dir"
    
    # Copy memo files
    cp -r "$source_dir"/* "$target_dir/" 2>/dev/null
    
    local migrated_count=$(find "$target_dir" -name "*.json" | wc -l)
    echo "✅ Migrated $migrated_count memo files"
    
    # Verify migration
    echo "🔍 Verifying migrated memos..."
    swissarmyhammer memo list | head -5
}

# Main command handling
case "$1" in
    "full")
        create_full_backup
        ;;
    "incremental")
        create_incremental_backup
        ;;
    "restore")
        restore_backup "$2"
        ;;
    "list")
        list_backups
        ;;
    "verify")
        verify_backup "$2"
        ;;
    "cleanup")
        cleanup_backups "${2:-30}"
        ;;
    "migrate")
        migrate_memos "$2" "$3"
        ;;
    *)
        echo "Usage: $0 {full|incremental|restore|list|verify|cleanup|migrate}"
        echo "  full                    - Create full backup"
        echo "  incremental             - Create incremental backup"
        echo "  restore [backup_name]   - Restore from backup"
        echo "  list                    - List all available backups"
        echo "  verify <backup_name>    - Verify backup integrity"
        echo "  cleanup [days]          - Remove backups older than N days (default: 30)"
        echo "  migrate <source> [dest] - Migrate memos from another directory"
        ;;
esac
```

## Conclusion

These advanced usage examples demonstrate the full potential of SwissArmyHammer's memoranda system. By combining CLI automation, AI integration, and sophisticated workflows, you can create a powerful knowledge management system tailored to your specific needs.

**Key Takeaways:**

- **Search Strategy**: Use specific, targeted queries and implement category-based organization
- **AI Integration**: Leverage MCP tools for context-aware memo creation and analysis  
- **Automation**: Build scripts for routine tasks and integrate with your development workflow
- **Team Collaboration**: Share knowledge through structured templates and synchronized workflows
- **Performance**: Monitor collection size and implement archiving strategies for large datasets
- **Backup & Recovery**: Implement comprehensive backup strategies to protect your knowledge base

For more information, see:
- [Memoranda Quickstart Guide](memoranda-quickstart.md) - Basic usage tutorial
- [CLI Reference](../src/cli-memoranda.md) - Complete command documentation
- [MCP Tools Reference](../src/mcp-memoranda.md) - AI assistant integration
- [API Documentation](../src/api-reference.md) - Programmatic usage patterns