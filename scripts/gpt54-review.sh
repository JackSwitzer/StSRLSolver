#!/usr/bin/env bash
# GPT 5.4 review via Codex CLI (NOT OpenRouter)
#
# Usage: ./scripts/gpt54-review.sh <prompt_file> [output_file] [effort: high|extra-high]
#
# Reads prompt from file (or stdin if -), writes response to output_file (or stdout).
# Auth: Codex CLI uses its own native auth. Run `codex login` if needed.

set -euo pipefail

CODEX="/Applications/Codex.app/Contents/Resources/codex"
MODEL="gpt-5.4"

# ── Help ──────────────────────────────────────────────────

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo "Usage: $0 <prompt_file|-|--inline 'prompt'> [output_file] [effort]"
  echo ""
  echo "Arguments:"
  echo "  prompt_file   Path to file containing the prompt (use - for stdin)"
  echo "  --inline      Pass prompt as a string argument"
  echo "  output_file   Optional: write result to file (default: stdout)"
  echo "  effort        high (default) or extra-high"
  echo ""
  echo "Examples:"
  echo "  $0 /tmp/prompt.txt /tmp/result.md high"
  echo "  echo 'Review this code' | $0 -"
  echo "  $0 --inline 'Explain PPO' /tmp/result.md extra-high"
  echo ""
  echo "Auth: Uses Codex CLI at $CODEX (NOT OpenRouter)"
  echo "      Run '$CODEX login' if auth fails"
  exit 0
fi

# ── Args ──────────────────────────────────────────────────

PROMPT_FILE="${1:--}"
OUTPUT_FILE="${2:-}"
EFFORT="${3:-high}"

if [[ ! -x "$CODEX" ]]; then
  echo "ERROR: Codex CLI not found at $CODEX" >&2
  echo "  Install the Codex app or check the path." >&2
  exit 1
fi

if [[ "$EFFORT" != "high" && "$EFFORT" != "extra-high" ]]; then
  echo "ERROR: effort must be 'high' or 'extra-high', got '$EFFORT'" >&2
  exit 1
fi

# ── Read prompt ───────────────────────────────────────────

if [[ "$PROMPT_FILE" == "--inline" ]]; then
  # Inline mode: prompt is the second arg, output shifts
  PROMPT="${2:-}"
  OUTPUT_FILE="${3:-}"
  EFFORT="${4:-high}"
  if [[ -z "$PROMPT" ]]; then
    echo "ERROR: --inline requires a prompt string" >&2
    exit 1
  fi
elif [[ "$PROMPT_FILE" == "-" ]]; then
  PROMPT="$(cat)"
else
  if [[ ! -f "$PROMPT_FILE" ]]; then
    echo "ERROR: Prompt file not found: $PROMPT_FILE" >&2
    exit 1
  fi
  PROMPT="$(cat "$PROMPT_FILE")"
fi

if [[ -z "$PROMPT" ]]; then
  echo "ERROR: Empty prompt" >&2
  exit 1
fi

# ── Run via Codex CLI ─────────────────────────────────────

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
