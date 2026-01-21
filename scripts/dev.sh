#!/bin/bash
# Full Development Environment Launcher
# Starts Python search server + builds mod + launches game
#
# Usage:
#   ./dev.sh              - Full: server + build + launch
#   ./dev.sh --no-server  - Skip Python server
#   ./dev.sh --server     - Start search server only
#   ./dev.sh --stop       - Stop everything

set -e

# Paths
PROJECT_ROOT="/Users/jackswitzer/Desktop/SlayTheSpireRL"
MOD_DIR="$PROJECT_ROOT/mod"
GAME_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
MODS_DIR="$GAME_DIR/mods"
LOG_DIR="$PROJECT_ROOT/logs"
VENV="$PROJECT_ROOT/.venv"

# Server config
SEARCH_SERVER_PORT=9998
SEARCH_SERVER_PID_FILE="$LOG_DIR/.search_server.pid"
SEARCH_SERVER_LOG="$LOG_DIR/search_server.log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m'

# Parse arguments
START_SERVER=true
START_GAME=true
STOP_ALL=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --no-server) START_SERVER=false; shift ;;
        --server) START_GAME=false; shift ;;
        --stop) STOP_ALL=true; shift ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  (none)        Full: start server + build mod + launch game"
            echo "  --no-server   Skip Python search server"
            echo "  --server      Start search server only (no game)"
            echo "  --stop        Stop everything (server + game)"
            echo ""
            echo "The search server runs on port $SEARCH_SERVER_PORT"
            echo "Logs are written to: $LOG_DIR"
            exit 0
            ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

echo -e "${BOLD}${CYAN}╔═══════════════════════════════════════╗${NC}"
echo -e "${BOLD}${CYAN}║      STS RL Development Launcher      ║${NC}"
echo -e "${BOLD}${CYAN}╚═══════════════════════════════════════╝${NC}"
echo ""

mkdir -p "$LOG_DIR"

# ─────────────────────────────────────────────────────────────
# Stop functions
# ─────────────────────────────────────────────────────────────

stop_search_server() {
    echo -e "${YELLOW}Stopping search server...${NC}"

    # Kill by PID file
    if [ -f "$SEARCH_SERVER_PID_FILE" ]; then
        local pid=$(cat "$SEARCH_SERVER_PID_FILE")
        if kill -0 "$pid" 2>/dev/null; then
            kill "$pid" 2>/dev/null || true
            sleep 1
        fi
        rm -f "$SEARCH_SERVER_PID_FILE"
    fi

    # Kill any remaining python server
    pkill -f "search_server.py" 2>/dev/null || true
    pkill -f "run_search_server.py" 2>/dev/null || true

    echo -e "${GREEN}Search server stopped${NC}"
}

stop_game() {
    echo -e "${YELLOW}Stopping game...${NC}"
    pkill -9 -f "ModTheSpire" 2>/dev/null || true
    pkill -9 -f "desktop-1.0.jar" 2>/dev/null || true
    sleep 2
    echo -e "${GREEN}Game stopped${NC}"
}

if [ "$STOP_ALL" = true ]; then
    stop_search_server
    stop_game
    echo -e "${GREEN}All stopped${NC}"
    exit 0
fi

# ─────────────────────────────────────────────────────────────
# Start Search Server
# ─────────────────────────────────────────────────────────────

start_search_server() {
    echo -e "${CYAN}[1/4] Starting Python Search Server...${NC}"

    # Stop any existing server
    stop_search_server 2>/dev/null || true

    # Check if port is available
    if lsof -i :$SEARCH_SERVER_PORT >/dev/null 2>&1; then
        echo -e "${RED}Port $SEARCH_SERVER_PORT is in use!${NC}"
        lsof -i :$SEARCH_SERVER_PORT
        exit 1
    fi

    # Activate venv and start server
    cd "$PROJECT_ROOT"

    # Start server in background
    source "$VENV/bin/activate"
    python3 -u run_search_server.py \
        --port $SEARCH_SERVER_PORT \
        --budget 1000 \
        > "$SEARCH_SERVER_LOG" 2>&1 &

    local pid=$!
    echo $pid > "$SEARCH_SERVER_PID_FILE"

    # Wait for server to start
    echo -n "       Waiting for server"
    local count=0
    while [ $count -lt 10 ]; do
        sleep 0.5
        count=$((count + 1))

        # Check if process died
        if ! kill -0 $pid 2>/dev/null; then
            echo ""
            echo -e "${RED}Server failed to start!${NC}"
            cat "$SEARCH_SERVER_LOG"
            exit 1
        fi

        # Check if port is listening
        if lsof -i :$SEARCH_SERVER_PORT >/dev/null 2>&1; then
            echo ""
            echo -e "${GREEN}       Server running on port $SEARCH_SERVER_PORT (PID: $pid)${NC}"
            return 0
        fi

        echo -n "."
    done

    echo ""
    echo -e "${YELLOW}       Server may still be starting (PID: $pid)${NC}"
}

# ─────────────────────────────────────────────────────────────
# Build and Deploy Mod
# ─────────────────────────────────────────────────────────────

build_mod() {
    local step=$1
    echo -e "${CYAN}[$step/4] Building EVTracker mod...${NC}"
    cd "$MOD_DIR"

    if mvn package -q 2>&1; then
        local ver=$(grep -o '<version>[^<]*</version>' pom.xml | head -1 | sed 's/<[^>]*>//g')
        echo -e "${GREEN}       Built v${ver}${NC}"
    else
        echo -e "${RED}       Build failed!${NC}"
        exit 1
    fi
}

deploy_mod() {
    local step=$1
    echo -e "${CYAN}[$step/4] Deploying mod...${NC}"
    mkdir -p "$MODS_DIR"
    cp "$MOD_DIR/target/EVTracker.jar" "$MODS_DIR/"
    echo -e "${GREEN}       Deployed to $MODS_DIR${NC}"
}

# ─────────────────────────────────────────────────────────────
# Launch Game
# ─────────────────────────────────────────────────────────────

launch_game() {
    local step=$1
    echo -e "${CYAN}[$step/4] Launching game...${NC}"

    stop_game 2>/dev/null || true

    local GAME_LOG="$LOG_DIR/game_$(date +%Y%m%d_%H%M%S).log"

    cd "$GAME_DIR"
    nohup ./jre/bin/java -Xmx1G -jar ModTheSpire.jar \
        --skip-launcher \
        --mods basemod,stslib,evtracker \
        > "$GAME_LOG" 2>&1 &

    local game_pid=$!

    echo -n "       Waiting for game"
    local count=0
    while [ $count -lt 30 ]; do
        sleep 1
        count=$((count + 1))

        if ! kill -0 $game_pid 2>/dev/null; then
            echo ""
            echo -e "${RED}Game crashed!${NC}"
            tail -50 "$GAME_LOG"
            exit 1
        fi

        if grep -q "Starting game" "$GAME_LOG" 2>/dev/null; then
            echo ""
            echo -e "${GREEN}       Game running (PID: $game_pid)${NC}"
            break
        fi

        echo -n "."
    done
    echo ""

    echo -e "${GREEN}       Log: $GAME_LOG${NC}"
}

# ─────────────────────────────────────────────────────────────
# Main Execution
# ─────────────────────────────────────────────────────────────

step=1

if [ "$START_SERVER" = true ]; then
    start_search_server
    step=$((step + 1))
else
    echo -e "${YELLOW}[1/4] Skipping search server${NC}"
    step=$((step + 1))
fi

if [ "$START_GAME" = true ]; then
    build_mod $step
    step=$((step + 1))
    deploy_mod $step
    step=$((step + 1))
    launch_game $step
fi

echo ""
echo -e "${BOLD}${GREEN}═══════════════════════════════════════${NC}"
echo -e "${BOLD}${GREEN}            Ready to Play!             ${NC}"
echo -e "${BOLD}${GREEN}═══════════════════════════════════════${NC}"
echo ""

if [ "$START_SERVER" = true ]; then
    echo -e "${CYAN}Search Server:${NC} http://localhost:$SEARCH_SERVER_PORT"
    echo -e "${CYAN}Server Log:${NC}    tail -f $SEARCH_SERVER_LOG"
fi

if [ "$START_GAME" = true ]; then
    echo -e "${CYAN}Game Log:${NC}      tail -f $LOG_DIR/game_*.log"
    echo -e "${CYAN}EV Log:${NC}        tail -f $LOG_DIR/evlog_*.jsonl"
fi

echo ""
echo -e "${YELLOW}Stop all:${NC}      $0 --stop"
echo ""
