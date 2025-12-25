#!/bin/bash
# Long-running self-play for vacation
# Prevents sleep and logs progress

GAME_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
WORKSHOP_MTS="/Users/jackswitzer/Library/Application Support/Steam/steamapps/workshop/content/646570/1605060445/ModTheSpire.jar"
LOG_DIR="/Users/jackswitzer/Desktop/StSRLSolver/logs"
PROGRESS_FILE="$LOG_DIR/selfplay_progress.txt"

mkdir -p "$LOG_DIR"

echo "=== Vacation Self-Play Launcher ===" | tee -a "$LOG_DIR/vacation_launch.log"
echo "Started: $(date)" | tee -a "$LOG_DIR/vacation_launch.log"
echo ""

# Kill any existing caffeinate
pkill -f "caffeinate.*selfplay" 2>/dev/null

# Prevent sleep for the duration (24 hours * 7 days = 604800 seconds)
caffeinate -dimsu -w $$ &
CAFFEINE_PID=$!
echo "Sleep prevention enabled (PID: $CAFFEINE_PID)"

# Track start time
echo "$(date +%s)" > "$LOG_DIR/selfplay_start_time.txt"
echo "0" > "$PROGRESS_FILE"

# Use Workshop ModTheSpire if available
if [ -f "$WORKSHOP_MTS" ]; then
    MTS_JAR="$WORKSHOP_MTS"
else
    MTS_JAR="$GAME_DIR/ModTheSpire.jar"
fi

echo "Launching Slay the Spire with self-play agent..."
echo "Games target: 10000"
echo "Exploration rate: 15%"
echo ""
echo "Monitor progress with:"
echo "  tail -f $LOG_DIR/selfplay_*.log"
echo "  cat $PROGRESS_FILE"
echo ""
echo "Stop with: pkill -f 'java.*ModTheSpire'"
echo ""

cd "$GAME_DIR"

# Run the game (this will connect to our self-play agent)
./jre/bin/java -jar "$MTS_JAR" --mods basemod,stslib,CommunicationMod --skip-intro 2>&1 | tee -a "$LOG_DIR/game_output.log"

# Cleanup
echo "Game exited at: $(date)" | tee -a "$LOG_DIR/vacation_launch.log"
kill $CAFFEINE_PID 2>/dev/null
