#!/bin/bash
FILE=$1
echo "=== Code Review ==="
swissarmyhammer test review/code --file_path "$FILE"

echo -e "\n=== Security Check ==="
swissarmyhammer test review/security --code "$(cat $FILE)"

echo -e "\n=== Test Generation ==="
swissarmyhammer test test/unit --code "$(cat $FILE)"