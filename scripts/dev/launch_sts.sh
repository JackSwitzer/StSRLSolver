#!/bin/bash
# Launch Slay the Spire with CommunicationMod
#
# Usage:
#   ./launch_sts.sh              # Launch with CommunicationMod (default)
#   ./launch_sts.sh --no-mods    # Launch vanilla (no mods)
#   ./launch_sts.sh --mods "basemod,stslib,evtracker"  # Custom mod list
#   ./launch_sts.sh --no-skip-intro  # Show intro video
#   ./launch_sts.sh --memory 2G  # Custom memory allocation
#   ./launch_sts.sh --fg         # Run in foreground (blocking)

set -e

# Paths
GAME_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
PROJECT_ROOT="/Users/jackswitzer/Desktop/SlayTheSpireRL"
LOG_DIR="$PROJECT_ROOT/logs"

# Defaults
MODS="basemod,CommunicationMod"
SKIP_INTRO=true
SKIP_LAUNCHER=true
MEMORY="1G"
FOREGROUND=false
VANILLA=false

# Colors
CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-mods|--vanilla)
            VANILLA=true
            shift
            ;;
        --mods)
            MODS="$2"
            shift 2
            ;;
        --no-skip-intro)
            SKIP_INTRO=false
            shift
            ;;
        --no-skip-launcher)
            SKIP_LAUNCHER=false
            shift
            ;;
        --memory|-m)
            MEMORY="$2"
            shift 2
            ;;
        --fg|--foreground)
            FOREGROUND=true
            shift
            ;;
        -h|--help)
            echo "Launch Slay the Spire with mods"
            echo ""
            echo "Usage: $(basename "$0") [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-mods, --vanilla    Launch without mods (vanilla game)"
            echo "  --mods \"mod1,mod2\"      Custom mod list (default: basemod,CommunicationMod)"
            echo "  --no-skip-intro         Show intro video"
            echo "  --no-skip-launcher      Show ModTheSpire launcher GUI"
            echo "  --memory, -m SIZE       Memory allocation (default: 1G)"
            echo "  --fg, --foreground      Run in foreground (blocking)"
            echo "  -h, --help              Show this help"
            echo ""
            echo "Examples:"
            echo "  $(basename "$0")                           # Default: CommunicationMod"
            echo "  $(basename "$0") --mods basemod,evtracker  # Custom mods"
            echo "  $(basename "$0") --no-mods                 # Vanilla game"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage"
            exit 1
            ;;
    esac
done

# Check game directory exists
if [ ! -d "$GAME_DIR" ]; then
    echo -e "${RED}Error: Game directory not found${NC}"
    echo "Expected: $GAME_DIR"
    exit 1
fi

# Check ModTheSpire.jar exists
if [ ! -f "$GAME_DIR/ModTheSpire.jar" ]; then
    echo -e "${RED}Error: ModTheSpire.jar not found${NC}"
    echo "Make sure ModTheSpire is installed"
    exit 1
fi

# Build launch command
JAVA_CMD="./jre/bin/java"
JAVA_ARGS="-Xmx${MEMORY}"

if $VANILLA; then
    # Vanilla launch (direct desktop jar, no ModTheSpire)
    echo -e "${CYAN}=== Launching Slay the Spire (Vanilla) ===${NC}"
    JAR_FILE="desktop-1.0.jar"
    if [ ! -f "$GAME_DIR/$JAR_FILE" ]; then
        echo -e "${RED}Error: $JAR_FILE not found${NC}"
        exit 1
    fi
    LAUNCH_CMD="$JAVA_CMD $JAVA_ARGS -jar $JAR_FILE"
    echo -e "${YELLOW}Mode:${NC} Vanilla (no mods)"
else
    echo -e "${CYAN}=== Launching Slay the Spire (Modded) ===${NC}"
    LAUNCH_CMD="$JAVA_CMD $JAVA_ARGS -jar ModTheSpire.jar"

    # Add flags
    if $SKIP_LAUNCHER; then
        LAUNCH_CMD="$LAUNCH_CMD --skip-launcher"
    fi
    if $SKIP_INTRO; then
        LAUNCH_CMD="$LAUNCH_CMD --skip-intro"
    fi
    LAUNCH_CMD="$LAUNCH_CMD --mods $MODS"

    echo -e "${YELLOW}Mods:${NC} $MODS"
fi

echo -e "${YELLOW}Memory:${NC} $MEMORY"
echo -e "${YELLOW}Skip Intro:${NC} $SKIP_INTRO"
echo -e "${YELLOW}Skip Launcher:${NC} $SKIP_LAUNCHER"
echo ""

# Create log directory
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/sts_$(date +%Y%m%d_%H%M%S).log"

cd "$GAME_DIR"

if $FOREGROUND; then
    echo -e "${GREEN}Launching in foreground...${NC}"
    echo -e "${YELLOW}Command:${NC} $LAUNCH_CMD"
    echo ""
    exec $LAUNCH_CMD 2>&1 | tee "$LOG_FILE"
else
    echo -e "${GREEN}Launching in background...${NC}"
    echo -e "${YELLOW}Log:${NC} $LOG_FILE"
    echo ""

    nohup $LAUNCH_CMD > "$LOG_FILE" 2>&1 &
    PID=$!

    echo -e "${GREEN}Launched!${NC} (PID: $PID)"
    echo ""
    echo "Monitor with: tail -f \"$LOG_FILE\""
    echo "Kill with:    kill $PID"
fi
