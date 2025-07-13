#!/bin/bash
# smart-review.sh

FILE=$1
EXTENSION="${FILE##*.}"

case $EXTENSION in
  py)
    PROMPT="review/code-dynamic"
    ARGS="--language python --focus_areas style,typing"
    ;;
  js|ts)
    PROMPT="review/code-dynamic"
    ARGS="--language javascript --focus_areas async,security"
    ;;
  html)
    PROMPT="review/accessibility"
    ARGS="--wcag_level AA"
    ;;
  sql)
    PROMPT="database-query-optimizer"
    ARGS="--database postgres"
    ;;
  *)
    PROMPT="review/code"
    ARGS=""
    ;;
esac

swissarmyhammer test $PROMPT --file_path "$FILE" $ARGS