#!/bin/bash
# analyze-codebase.sh

# Step 1: Get overview of the codebase
echo "=== Codebase Overview ==="
swissarmyhammer test help --topic "codebase structure" --detail_level "detailed" > analysis/overview.md

# Step 2: Review critical files
echo "=== Security Review ==="
for file in auth.py payment.py user.py; do
  echo "Reviewing $file..."
  swissarmyhammer test review/security \
    --code "$(cat src/$file)" \
    --context "handles sensitive data" \
    --severity_threshold "medium" > "analysis/security-$file.md"
done

# Step 3: Generate tests for uncovered code
echo "=== Test Generation ==="
swissarmyhammer test test/unit \
  --code "$(cat src/utils.py)" \
  --framework "pytest" \
  --style "BDD" \
  --coverage_target "90" > tests/test_utils_generated.py

# Step 4: Create documentation
echo "=== Documentation ==="
swissarmyhammer test docs/api \
  --code "$(cat src/api.py)" \
  --api_type "REST" \
  --format "openapi" > docs/api-spec.yaml

echo "Analysis complete! Check the analysis/ directory for results."