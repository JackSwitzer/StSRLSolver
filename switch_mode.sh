#!/bin/bash
# Switch between regular agent and self-play mode

CONFIG_FILE="$HOME/Library/Preferences/ModTheSpire/CommunicationMod/config.properties"
AGENT_SCRIPT="/Users/jackswitzer/Desktop/StSRLSolver/run_agent.sh"
SELFPLAY_SCRIPT="/Users/jackswitzer/Desktop/StSRLSolver/run_selfplay.sh"

case "$1" in
    "agent")
        echo "command=$AGENT_SCRIPT" > "$CONFIG_FILE"
        echo "Switched to AGENT mode (regular Watcher agent)"
        ;;
    "selfplay")
        echo "command=$SELFPLAY_SCRIPT" > "$CONFIG_FILE"
        echo "Switched to SELF-PLAY mode (learning agent)"
        ;;
    "status")
        echo "Current config:"
        cat "$CONFIG_FILE"
        ;;
    *)
        echo "Usage: $0 {agent|selfplay|status}"
        echo ""
        echo "  agent    - Use regular Watcher agent"
        echo "  selfplay - Use self-play learning agent"
        echo "  status   - Show current config"
        exit 1
        ;;
esac
