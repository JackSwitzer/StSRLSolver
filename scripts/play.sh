#!/usr/bin/env bash
# Full setup + launch for STS with EVTracker logging
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
STS_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
MODS_DIR="$STS_DIR/mods"
LOG_DIR="$PROJECT_DIR/logs"
MOD_JAR="$PROJECT_DIR/mod/target/EVTracker.jar"
MOD_SRC="$PROJECT_DIR/mod"

mkdir -p "$LOG_DIR"

# --- Check STS install ---
if [[ ! -d "$STS_DIR" ]]; then
    echo "ERROR: Slay the Spire not found at $STS_DIR"
    exit 1
fi

# --- Build mod if source changed or JAR missing ---
if [[ -d "$MOD_SRC/src" ]]; then
    if [[ ! -f "$MOD_JAR" ]] || [[ $(find "$MOD_SRC/src" -newer "$MOD_JAR" -type f 2>/dev/null | head -1) ]]; then
        echo "Building EVTracker mod..."
        (cd "$MOD_SRC" && mvn package -q -DskipTests)
        echo "Build complete."
    fi
fi

if [[ ! -f "$MOD_JAR" ]]; then
    echo "ERROR: EVTracker.jar not found at $MOD_JAR"
    echo "Build it first: cd mod && mvn package"
    exit 1
fi

# --- Check required mods ---
for mod in BaseMod.jar ModTheSpire.jar; do
    if [[ ! -f "$MODS_DIR/$mod" ]] && [[ ! -f "$STS_DIR/$mod" ]]; then
        echo "ERROR: $mod not found. Install via Steam Workshop or manually."
        exit 1
    fi
done

# --- Copy EVTracker if newer ---
if [[ "$MOD_JAR" -nt "$MODS_DIR/EVTracker.jar" ]] || [[ ! -f "$MODS_DIR/EVTracker.jar" ]]; then
    echo "Copying EVTracker.jar -> mods/"
    cp "$MOD_JAR" "$MODS_DIR/EVTracker.jar"
fi

# --- Start log watcher in background ---
echo "=== Slay the Spire + EVTracker ==="
echo "Logs: $LOG_DIR/evlog_*.jsonl"
echo ""

WATCHER_PID=""
if [[ -f "$PROJECT_DIR/scripts/watch_game.py" ]]; then
    echo "Starting log watcher..."
    uv run python "$PROJECT_DIR/scripts/watch_game.py" &
    WATCHER_PID=$!
    trap "kill $WATCHER_PID 2>/dev/null; exit" EXIT INT TERM
fi

echo "Launching Slay the Spire..."
cd "$STS_DIR"
./jre/bin/java -Xmx1G \
    -jar ModTheSpire.jar \
    --skip-launcher \
    --mods basemod,evtracker
