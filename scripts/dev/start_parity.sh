#!/bin/bash
# Start parity testing - launch STS and prepare for comparison
#
# Usage:
#   ./start_parity.sh                        # Default: WATCHER A20 seed 1234567890
#   ./start_parity.sh --seed MY_SEED         # Custom seed
#   ./start_parity.sh --character IRONCLAD   # Different character
#   ./start_parity.sh --ascension 15         # Different ascension
#   ./start_parity.sh --auto                 # Auto-start via CommunicationMod (if configured)
#   ./start_parity.sh --watch                # Watch for save file and run parity check

set -e

# Paths
PROJECT_ROOT="/Users/jackswitzer/Desktop/SlayTheSpireRL"
GAME_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
SAVE_DIR="$GAME_DIR/saves"
COMM_MOD_CONFIG="/Users/jackswitzer/Library/Preferences/ModTheSpire/CommunicationMod/config.properties"
LOG_DIR="$PROJECT_ROOT/logs"

# Defaults
SEED="1234567890"
CHARACTER="WATCHER"
ASCENSION=20
AUTO_START=false
WATCH_MODE=false
SKIP_LAUNCH=false
PREDICTIONS_ONLY=false

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
BOLD='\033[1m'
NC='\033[0m'

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --seed|-s)
            SEED="$2"
            shift 2
            ;;
        --character|-c)
            CHARACTER="${2^^}"  # Uppercase
            shift 2
            ;;
        --ascension|-a)
            ASCENSION="$2"
            shift 2
            ;;
        --auto)
            AUTO_START=true
            shift
            ;;
        --watch|-w)
            WATCH_MODE=true
            shift
            ;;
        --skip-launch)
            SKIP_LAUNCH=true
            shift
            ;;
        --predictions-only|-p)
            PREDICTIONS_ONLY=true
            shift
            ;;
        -h|--help)
            echo "Parity Testing - Launch STS and compare predictions"
            echo ""
            echo "Usage: $(basename "$0") [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --seed, -s SEED        Seed to use (default: 1234567890)"
            echo "  --character, -c CHAR   Character class (default: WATCHER)"
            echo "  --ascension, -a LEVEL  Ascension level (default: 20)"
            echo "  --auto                 Auto-start run via CommunicationMod"
            echo "  --watch, -w            Watch for save file and run parity"
            echo "  --skip-launch          Don't launch STS (use existing)"
            echo "  --predictions-only, -p Just generate predictions"
            echo "  -h, --help             Show this help"
            echo ""
            echo "Examples:"
            echo "  $(basename "$0") --seed TEST123"
            echo "  $(basename "$0") --seed 99999 --ascension 0"
            echo "  $(basename "$0") --auto --watch"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage"
            exit 1
            ;;
    esac
done

# Banner
echo -e "${CYAN}================================================================${NC}"
echo -e "${CYAN}  PARITY TESTING - STS Seed Prediction Verification${NC}"
echo -e "${CYAN}================================================================${NC}"
echo ""
echo -e "${YELLOW}Seed:${NC}      $SEED"
echo -e "${YELLOW}Character:${NC} $CHARACTER"
echo -e "${YELLOW}Ascension:${NC} $ASCENSION"
echo ""

# Create log directory
mkdir -p "$LOG_DIR"

# Generate predictions
echo -e "${GREEN}[1/3] Generating predictions...${NC}"
cd "$PROJECT_ROOT"

PREDICTION_FILE="/tmp/parity_predictions_${SEED}.txt"
uv run scripts/dev/test_parity.py --seed "$SEED" --character "$CHARACTER" --ascension "$ASCENSION" 2>&1 | tee "$PREDICTION_FILE"

if $PREDICTIONS_ONLY; then
    echo ""
    echo -e "${GREEN}Predictions saved to: $PREDICTION_FILE${NC}"
    exit 0
fi

# Launch STS if not skipping
if ! $SKIP_LAUNCH; then
    echo ""
    echo -e "${GREEN}[2/3] Launching Slay the Spire...${NC}"

    # Check if auto-start is requested and CommunicationMod is configured
    if $AUTO_START; then
        # Create a temporary bot script that will start the run
        BOT_SCRIPT="/tmp/parity_auto_start.py"
        cat > "$BOT_SCRIPT" << 'PYEOF'
#!/usr/bin/env python3
"""Auto-start bot for parity testing via CommunicationMod."""
import json
import sys
import os

# Read config from environment
SEED = os.environ.get('PARITY_SEED', '1234567890')
CHARACTER = os.environ.get('PARITY_CHARACTER', 'WATCHER')
ASCENSION = os.environ.get('PARITY_ASCENSION', '20')

def main():
    print("ready", flush=True)
    started = False
    at_neow = False

    while True:
        try:
            line = sys.stdin.readline()
            if not line:
                break

            state = json.loads(line)

            if "error" in state:
                print("state", flush=True)
                continue

            in_game = state.get("in_game", False)
            game_state = state.get("game_state", {})
            available = state.get("available_commands", [])

            # If not in game, start the run
            if not in_game and not started:
                cmd = f"start {CHARACTER} {ASCENSION} {SEED}"
                print(f"[BOT] Starting: {cmd}", file=sys.stderr)
                print(cmd, flush=True)
                started = True
                continue

            # Check if we're at Neow (floor 0)
            floor = game_state.get("floor", -1)
            screen_type = game_state.get("screen_type", "")

            if floor == 0 and screen_type == "EVENT":
                # We're at Neow - mission accomplished
                print("[BOT] At Neow (floor 0). Stopping auto-start bot.", file=sys.stderr)
                print(f"[BOT] Predictions file: /tmp/parity_predictions_{SEED}.txt", file=sys.stderr)
                # Exit gracefully - let user take over
                sys.exit(0)

            # If in game but not at neow yet, wait
            if in_game:
                print("state", flush=True)

        except json.JSONDecodeError:
            print("state", flush=True)
            continue
        except Exception as e:
            print(f"[BOT] Error: {e}", file=sys.stderr)
            print("state", flush=True)

if __name__ == "__main__":
    main()
PYEOF

        # Update CommunicationMod config
        echo -e "${YELLOW}Configuring CommunicationMod for auto-start...${NC}"

        # Backup existing config
        if [ -f "$COMM_MOD_CONFIG" ]; then
            cp "$COMM_MOD_CONFIG" "$COMM_MOD_CONFIG.bak"
        fi

        # Write new config with environment variables
        mkdir -p "$(dirname "$COMM_MOD_CONFIG")"
        cat > "$COMM_MOD_CONFIG" << EOF
#Parity Test Auto-Start
command=PARITY_SEED=$SEED PARITY_CHARACTER=$CHARACTER PARITY_ASCENSION=$ASCENSION python3 $BOT_SCRIPT
EOF

        echo -e "${GREEN}CommunicationMod will auto-start: $CHARACTER A$ASCENSION seed $SEED${NC}"
    fi

    # Launch the game
    "$PROJECT_ROOT/scripts/dev/launch_sts.sh"
    LAUNCH_PID=$!

    echo ""
    echo -e "${GREEN}STS launched!${NC}"
fi

# Print instructions
echo ""
echo -e "${CYAN}================================================================${NC}"
if $AUTO_START; then
    echo -e "${BOLD}AUTO-START ENABLED${NC}"
    echo ""
    echo "CommunicationMod will automatically:"
    echo "  1. Start a new $CHARACTER run"
    echo "  2. Set Ascension $ASCENSION"
    echo "  3. Use seed: $SEED"
    echo "  4. Stop at Neow (floor 0)"
    echo ""
    echo -e "${YELLOW}After the run starts, you can:${NC}"
else
    echo -e "${BOLD}MANUAL START REQUIRED${NC}"
    echo ""
    echo "In Slay the Spire:"
    echo "  1. Press ~ (tilde) to open console"
    echo "  2. Type: evseed $SEED"
    echo "  3. Start new run: $CHARACTER, Ascension $ASCENSION"
    echo ""
    echo -e "${YELLOW}Alternatively, use custom seed option:${NC}"
    echo "  Main Menu -> Standard -> Seeded -> Enter: $SEED"
    echo ""
fi
echo -e "${CYAN}================================================================${NC}"

# Watch mode
if $WATCH_MODE; then
    echo ""
    echo -e "${GREEN}[3/3] Watching for save file...${NC}"

    SAVE_FILE="$SAVE_DIR/${CHARACTER}.autosave"
    echo "Watching: $SAVE_FILE"
    echo "Press Ctrl+C to stop watching"
    echo ""

    # Wait for save file to appear or be updated
    INITIAL_MTIME=""
    if [ -f "$SAVE_FILE" ]; then
        INITIAL_MTIME=$(stat -f "%m" "$SAVE_FILE" 2>/dev/null || echo "0")
    fi

    while true; do
        if [ -f "$SAVE_FILE" ]; then
            CURRENT_MTIME=$(stat -f "%m" "$SAVE_FILE" 2>/dev/null || echo "0")
            if [ "$CURRENT_MTIME" != "$INITIAL_MTIME" ]; then
                echo ""
                echo -e "${GREEN}Save file updated! Running parity check...${NC}"
                echo ""

                # Run parity check
                cd "$PROJECT_ROOT"
                uv run scripts/dev/test_parity.py --character "$CHARACTER"

                echo ""
                echo -e "${CYAN}Parity check complete.${NC}"
                echo "Continuing to watch for changes..."
                INITIAL_MTIME="$CURRENT_MTIME"
            fi
        fi
        sleep 2
    done
else
    echo ""
    echo -e "${YELLOW}To run parity check after starting the game:${NC}"
    echo "  cd $PROJECT_ROOT"
    echo "  uv run scripts/dev/parity.sh"
    echo ""
    echo "Or run the Python script directly:"
    echo "  uv run scripts/dev/test_parity.py --character $CHARACTER"
fi
