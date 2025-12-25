#!/bin/bash
# Monitor self-play progress with live updates

PROGRESS_FILE="/Users/jackswitzer/Desktop/StSRLSolver/logs/selfplay_progress.txt"
LOG_DIR="/Users/jackswitzer/Desktop/StSRLSolver/logs"

clear
echo "╔════════════════════════════════════════════════════════════╗"
echo "║          SELF-PLAY TRAINING MONITOR                        ║"
echo "╚════════════════════════════════════════════════════════════╝"
echo ""

while true; do
    if [ -f "$PROGRESS_FILE" ]; then
        # Move cursor up and clear
        tput cuu 10 2>/dev/null
        tput ed 2>/dev/null

        echo "┌──────────────────────────────────────────────────────────┐"
        cat "$PROGRESS_FILE" | while read line; do
            printf "│ %-56s │\n" "$line"
        done
        echo "└──────────────────────────────────────────────────────────┘"

        # Progress bar
        GAMES=$(grep "Games:" "$PROGRESS_FILE" | grep -oE "[0-9]+/[0-9]+" | head -1)
        if [ -n "$GAMES" ]; then
            CURRENT=$(echo $GAMES | cut -d'/' -f1)
            TOTAL=$(echo $GAMES | cut -d'/' -f2)
            PCT=$((CURRENT * 50 / TOTAL))
            BAR=""
            for i in $(seq 1 50); do
                if [ $i -le $PCT ]; then
                    BAR="${BAR}█"
                else
                    BAR="${BAR}░"
                fi
            done
            echo ""
            echo "Progress: [$BAR]"
        fi

        echo ""
        echo "Press Ctrl+C to stop monitoring"
    else
        echo "Waiting for progress data..."
    fi

    sleep 5
done
