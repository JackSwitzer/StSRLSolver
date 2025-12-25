#!/bin/bash
# Self-play training wrapper for CommunicationMod
# This script runs the self-play agent that uses line evaluation

cd /Users/jackswitzer/Desktop/StSRLSolver

# Log startup
echo "$(date): Starting self-play trainer" >> logs/selfplay_startup.log

# Run with higher exploration for learning, lower for exploitation
# Default: 15% exploration (try random actions to discover new strategies)
/Users/jackswitzer/.local/bin/uv run python3 self_play_trainer.py \
    --games 10000 \
    --exploration 0.15

echo "$(date): Self-play trainer exited" >> logs/selfplay_startup.log
