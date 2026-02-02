#!/usr/bin/env bash
# Launch STS with EVTracker for parity data collection
set -euo pipefail

PROJECT_DIR="$(cd "$(dirname "$0")/.." && pwd)"
STS_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
MODS_DIR="$STS_DIR/mods"
LOG_DIR="$PROJECT_DIR/logs"
MOD_JAR="$PROJECT_DIR/mod/target/EVTracker.jar"

# Ensure log directory exists
mkdir -p "$LOG_DIR"

# Check that the built mod JAR exists
if [[ ! -f "$MOD_JAR" ]]; then
    echo "ERROR: EVTracker.jar not found at $MOD_JAR"
    echo "Build it first: cd mod && mvn package"
    exit 1
fi

# Check STS install
if [[ ! -d "$STS_DIR" ]]; then
    echo "ERROR: Slay the Spire not found at $STS_DIR"
    exit 1
fi

# Copy mod JAR to STS mods directory if newer
if [[ "$MOD_JAR" -nt "$MODS_DIR/EVTracker.jar" ]] || [[ ! -f "$MODS_DIR/EVTracker.jar" ]]; then
    echo "Copying EVTracker.jar to STS mods directory..."
    cp "$MOD_JAR" "$MODS_DIR/EVTracker.jar"
fi

# Check required mods
for mod in BaseMod.jar; do
    if [[ ! -f "$MODS_DIR/$mod" ]]; then
        echo "WARNING: $mod not found in $MODS_DIR"
    fi
done

echo "=== Slay the Spire + EVTracker ==="
echo "Log directory: $LOG_DIR"
echo "Logs will be named: evlog_YYYY-MM-DD_HH-MM-SS.jsonl"
echo ""
echo "Launching..."

cd "$STS_DIR"
./jre/bin/java -Xmx1G \
    -jar ModTheSpire.jar \
    --skip-launcher \
    --mods basemod,EVTracker
