#!/usr/bin/env bash
# GPT 5.4 review via OpenAI Codex CLI
# Usage: ./scripts/gpt54-review.sh <prompt_file> [output_file] [effort: high|extra-high]
#
# Reads prompt from file (or stdin if -), writes response to output_file (or stdout).

set -euo pipefail

CODEX="/Applications/Codex.app/Contents/Resources/codex"
PROMPT_FILE="${1:--}"
OUTPUT_FILE="${2:-}"
EFFORT="${3:-high}"
MODEL="gpt-5.4"

if [[ ! -x "$CODEX" ]]; then
  echo "ERROR: Codex CLI not found at $CODEX" >&2
  exit 1
fi

# Read prompt
if [[ "$PROMPT_FILE" == "-" ]]; then
  PROMPT="$(cat)"
else
  PROMPT="$(cat "$PROMPT_FILE")"
fi

# Run via codex exec
RESULT="$($CODEX exec \
  -m "$MODEL" \
  -c "reasoning_effort=\"$EFFORT\"" \
  --sandbox read-only \
  "$PROMPT" 2>&1)"

if [[ -n "$OUTPUT_FILE" ]]; then
  mkdir -p "$(dirname "$OUTPUT_FILE")"
  printf '%s\n' "$RESULT" > "$OUTPUT_FILE"
  echo "Written to $OUTPUT_FILE" >&2
else
  printf '%s\n' "$RESULT"
fi
