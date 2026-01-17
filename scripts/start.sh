#!/bin/bash
# Master startup script for STS RL project
# Usage: ./start.sh [--build] [--debug]

set -e

PROJECT_DIR="/Users/jackswitzer/Desktop/SlayTheSpireRL"
GAME_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
MODS_DIR="$GAME_DIR/mods"
LOG_DIR="$PROJECT_DIR/logs"
MTS_PREFS="/Users/jackswitzer/Library/Preferences/ModTheSpire"

BUILD=false
DEBUG=false

for arg in "$@"; do
    case $arg in
        --build) BUILD=true ;;
        --debug) DEBUG=true ;;
    esac
done

echo "=== STS RL Startup ==="

# 1. Kill any existing game process
echo "[1/7] Cleaning up..."
pkill -f "ModTheSpire\|desktop-1.0" 2>/dev/null || true
sleep 1

# 2. Build mod if requested
if [ "$BUILD" = true ] || [ ! -f "$PROJECT_DIR/mod/target/EVTracker.jar" ]; then
    echo "[2/7] Building EVTracker..."
    cd "$PROJECT_DIR/mod"
    if mvn clean package -q 2>&1; then
        echo "       Done"
    else
        echo "       FAILED"
        exit 1
    fi
else
    echo "[2/7] Skip build (use --build)"
fi

# 3. Install mod
echo "[3/7] Installing mod..."
mkdir -p "$MODS_DIR"
cp "$PROJECT_DIR/mod/target/EVTracker.jar" "$MODS_DIR/"

# 4. Configure ModTheSpire to auto-load our mods
echo "[4/7] Configuring mods..."
mkdir -p "$MTS_PREFS"
cat > "$MTS_PREFS/mod_lists.json" << 'EOF'
{
  "defaultList": "<Default>",
  "lists": {
    "<Default>": [
      "BaseMod.jar",
      "StSLib.jar",
      "EVTracker.jar"
    ]
  }
}
EOF

# 5. Verify mods in folder
echo "[5/7] Mods installed:"
ls "$MODS_DIR"/*.jar 2>/dev/null | xargs -I {} basename {} | sed 's/^/       /'

# 6. Prepare logs
echo "[6/7] Log dir: $LOG_DIR"
mkdir -p "$LOG_DIR"

# 7. Launch game
echo "[7/7] Launching..."
cd "$GAME_DIR"

# Use --mods with MOD IDs (not JAR names) to skip launcher and auto-load
# Mod IDs from ModTheSpire.json in each JAR: basemod, stslib, evtracker
# Requires ModTheSpire v3.30.3 (built from source, installed in game dir)
./jre/bin/java \
    -Xmx1G -Xms512m \
    -Dorg.lwjgl.util.Debug=false \
    -jar ModTheSpire.jar \
    --mods basemod,stslib,evtracker \
    --skip-intro \
    > "$LOG_DIR/game_stdout.log" 2>&1 &

GAME_PID=$!
echo ""
echo "=== Game PID: $GAME_PID ==="
echo "Mods should auto-load. Start A20 Watcher run."
echo ""
echo "Monitor: python3 $PROJECT_DIR/scripts/monitor.py --watch"
echo "Logs:    tail -f $LOG_DIR/evlog_*.jsonl"
