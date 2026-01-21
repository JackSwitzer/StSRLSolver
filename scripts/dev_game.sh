#!/bin/bash
# Slay the Spire Dev Launch Script
# Rebuilds mod, deploys, and launches game with logging
#
# Usage:
#   ./dev_game.sh           - Full: kill game, rebuild, deploy, launch
#   ./dev_game.sh -r        - Reload: rebuild and deploy only
#   ./dev_game.sh -s        - Stop: kill game only
#   ./dev_game.sh -b        - Build: rebuild mod only
#   ./dev_game.sh -l        - Launch: launch game only (no rebuild)
#   ./dev_game.sh -v patch  - Increment patch version (1.0.5 -> 1.0.6)
#   ./dev_game.sh -v minor  - Increment minor version (1.0.5 -> 1.1.0)
#   ./dev_game.sh -v major  - Increment major version (1.0.5 -> 2.0.0)

set -e

# Paths
PROJECT_ROOT="/Users/jackswitzer/Desktop/SlayTheSpireRL"
MOD_DIR="$PROJECT_ROOT/mod"
GAME_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
MODS_DIR="$GAME_DIR/mods"
LOG_DIR="$PROJECT_ROOT/logs"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

# Parse arguments
MODE="full"
VERSION_BUMP=""
while [[ $# -gt 0 ]]; do
    case $1 in
        -r|--reload) MODE="reload"; shift ;;
        -s|--stop) MODE="stop"; shift ;;
        -b|--build) MODE="build"; shift ;;
        -l|--launch) MODE="launch"; shift ;;
        -v|--version) VERSION_BUMP="$2"; shift 2 ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo "  (none)        Full: kill, rebuild, deploy, launch"
            echo "  -r, --reload  Rebuild and deploy only"
            echo "  -s, --stop    Kill game only"
            echo "  -b, --build   Build mod only"
            echo "  -l, --launch  Launch only (no rebuild)"
            echo "  -v TYPE       Bump version (patch|minor|major)"
            exit 0
            ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

echo -e "${CYAN}=== STS Dev Launcher ===${NC}"

# Version increment
increment_version() {
    local type=$1
    local pom="$MOD_DIR/pom.xml"
    local json="$MOD_DIR/src/main/resources/ModTheSpire.json"

    local current=$(grep -o '<version>[0-9]*\.[0-9]*\.[0-9]*</version>' "$pom" | head -1 | sed 's/<[^>]*>//g')
    local major=$(echo "$current" | cut -d. -f1)
    local minor=$(echo "$current" | cut -d. -f2)
    local patch=$(echo "$current" | cut -d. -f3)

    case $type in
        patch) patch=$((patch + 1)) ;;
        minor) minor=$((minor + 1)); patch=0 ;;
        major) major=$((major + 1)); minor=0; patch=0 ;;
        *) echo -e "${RED}Invalid: $type${NC}"; exit 1 ;;
    esac

    local new_version="$major.$minor.$patch"
    echo -e "${YELLOW}Version: $current -> $new_version${NC}"
    sed -i '' "s/<version>$current<\/version>/<version>$new_version<\/version>/" "$pom"
    sed -i '' "s/\"version\": \"$current\"/\"version\": \"$new_version\"/" "$json"
}

kill_game() {
    echo -e "${YELLOW}Killing game...${NC}"
    pkill -9 -f "ModTheSpire" 2>/dev/null || true
    pkill -9 -f "desktop-1.0.jar" 2>/dev/null || true
    pkill -9 -f "SlayTheSpire" 2>/dev/null || true
    sleep 2
    echo -e "${GREEN}Done${NC}"
}

build_mod() {
    echo -e "${YELLOW}Building mod...${NC}"
    cd "$MOD_DIR"
    if mvn package -q; then
        local ver=$(grep -o '<version>[^<]*</version>' pom.xml | head -1 | sed 's/<[^>]*>//g')
        echo -e "${GREEN}Built v${ver}${NC}"
    else
        echo -e "${RED}Build failed!${NC}"
        exit 1
    fi
}

deploy_mod() {
    echo -e "${YELLOW}Deploying...${NC}"
    mkdir -p "$MODS_DIR"
    cp "$MOD_DIR/target/EVTracker.jar" "$MODS_DIR/"
    echo -e "${GREEN}Deployed${NC}"
}

launch_game() {
    mkdir -p "$LOG_DIR"
    local LOG_FILE="$LOG_DIR/game_$(date +%Y%m%d_%H%M%S).log"

    echo -e "${YELLOW}Launching game...${NC}"
    echo -e "Log: $LOG_FILE"

    cd "$GAME_DIR"

    # Launch game in background, redirect all output to log
    nohup ./jre/bin/java -Xmx1G -jar ModTheSpire.jar \
        --skip-launcher \
        --mods basemod,stslib,evtracker \
        > "$LOG_FILE" 2>&1 &

    local PID=$!
    echo -e "${GREEN}Launched (PID: $PID)${NC}"

    # Wait for game to start (check for window or log activity)
    echo -e "${YELLOW}Waiting for game to initialize...${NC}"
    local count=0
    while [ $count -lt 30 ]; do
        sleep 1
        count=$((count + 1))

        # Check if process died
        if ! kill -0 $PID 2>/dev/null; then
            echo -e "${RED}Game process died!${NC}"
            cat "$LOG_FILE"
            exit 1
        fi

        # Check for successful launch (Starting game appears after mod init)
        if grep -q "Starting game" "$LOG_FILE" 2>/dev/null; then
            echo -e "${GREEN}Game started!${NC}"
            break
        fi

        # Check for patch errors (specific patterns, not general ERROR logs)
        if grep -q "java.lang.*Exception\|NoSuchMethodException\|NoSuchFieldException\|SpirePatches.*failed" "$LOG_FILE" 2>/dev/null; then
            echo -e "${RED}Patch error detected:${NC}"
            grep -E "Exception|NoSuch|failed|at com\." "$LOG_FILE" | head -20
            exit 1
        fi

        echo -n "."
    done
    echo ""

    echo -e "${GREEN}Game running!${NC}"
    echo -e "${CYAN}Tail log with: tail -f $LOG_FILE${NC}"
}

# Handle version bump
[[ -n "$VERSION_BUMP" ]] && increment_version "$VERSION_BUMP"

# Execute
case $MODE in
    "stop") kill_game ;;
    "build") build_mod ;;
    "reload") build_mod; deploy_mod ;;
    "launch") deploy_mod; launch_game ;;
    "full") kill_game; build_mod; deploy_mod; launch_game ;;
esac

echo -e "${CYAN}Done!${NC}"
