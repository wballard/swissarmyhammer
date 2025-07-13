#!/bin/bash
# pr-review.sh

# Get changed files
CHANGED_FILES=$(git diff --name-only main...HEAD)

echo "# Pull Request Review" > pr-review.md
echo "" >> pr-review.md

for file in $CHANGED_FILES; do
  if [[ $file == *.py ]] || [[ $file == *.js ]] || [[ $file == *.ts ]]; then
    echo "## Review: $file" >> pr-review.md
    
    # Dynamic code review
    swissarmyhammer test review/code-dynamic \
      --file_path "$file" \
      --language "${file##*.}" \
      --focus_areas "bugs,security,performance" \
      --severity_level "info" >> pr-review.md
    
    echo "" >> pr-review.md
  fi
done

# Check for accessibility issues in UI files
for file in $CHANGED_FILES; do
  if [[ $file == *.html ]] || [[ $file == *.jsx ]] || [[ $file == *.tsx ]]; then
    echo "## Accessibility: $file" >> pr-review.md
    swissarmyhammer test review/accessibility \
      --code "$(cat $file)" \
      --wcag_level "AA" >> pr-review.md
    echo "" >> pr-review.md
  fi
done

echo "Review complete! See pr-review.md"