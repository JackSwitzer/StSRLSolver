#!/usr/bin/env bash
# Mint a Java golden trace: build TraceLab, install to game mods, launch a
# scripted seeded run, collect per-action JSONL. docs/goal/TOOLING.md T2.
# Usage: scripts/trace_java.sh <script.json> <out.jsonl> [--no-build]
# Launch procedure per docs/vault/headless-launch.md (ModTheSpire 3.30.3 CLI).
set -euo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
GAME="$HOME/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
HARNESS="$REPO/packages/harness-java"
MVN=/opt/homebrew/bin/mvn
BUILD_JAVA_HOME="${TRACELAB_JAVA_HOME:-/opt/homebrew/opt/openjdk@11}"
TIMEOUT_SECS="${TRACELAB_TIMEOUT:-420}"

if [[ $# -lt 2 ]]; then
  echo "usage: $0 <script.json> <out.jsonl> [--no-build]" >&2
  exit 2
fi

SCRIPT="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
OUT_DIR="$(mkdir -p "$(dirname "$2")" && cd "$(dirname "$2")" && pwd)"
OUT="$OUT_DIR/$(basename "$2")"
NO_BUILD="${3:-}"

[[ -f "$SCRIPT" ]] || { echo "script not found: $SCRIPT" >&2; exit 2; }
[[ -d "$GAME" ]] || { echo "game dir not found: $GAME" >&2; exit 2; }

if [[ "$NO_BUILD" != "--no-build" ]]; then
  echo "[trace_java] building TraceLab.jar"
  JAVA_HOME="$BUILD_JAVA_HOME" "$MVN" -q -f "$HARNESS/pom.xml" clean package
fi
cp "$HARNESS/target/TraceLab.jar" "$GAME/mods/TraceLab.jar"

LOG="$OUT_DIR/$(basename "${OUT%.jsonl}").game.log"
echo "[trace_java] launching game (script=$(basename "$SCRIPT"), timeout ${TIMEOUT_SECS}s, log=$LOG)"
cd "$GAME"
./jre/bin/java -Xmx2G \
  -Djava.awt.headless=false \
  -Dtracelab.script="$SCRIPT" \
  -Dtracelab.out="$OUT" \
  -jar ModTheSpire.jar --mods basemod,tracelab --skip-intro \
  >"$LOG" 2>&1 &
GAME_PID=$!

ELAPSED=0
while kill -0 "$GAME_PID" 2>/dev/null; do
  if (( ELAPSED >= TIMEOUT_SECS )); then
    echo "[trace_java] timeout after ${TIMEOUT_SECS}s, killing game" >&2
    kill "$GAME_PID" 2>/dev/null || true
    exit 4
  fi
  sleep 2
  ELAPSED=$((ELAPSED + 2))
done
wait "$GAME_PID" || true

if [[ ! -s "$OUT" ]]; then
  echo "[trace_java] no trace written; see $LOG" >&2
  exit 5
fi
LINES=$(wc -l <"$OUT" | tr -d ' ')
STATUS=$(tail -1 "$OUT")
echo "[trace_java] wrote $LINES records to $OUT"
echo "[trace_java] final: $STATUS"
