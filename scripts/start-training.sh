#!/bin/bash
# Start training dashboard + backend
# Usage: ./scripts/start-training.sh [--headless] [--workers N] [--sims N] [--ascension N]

set -e
cd "$(dirname "$0")/.."

WORKERS=8
SIMS=32
ASCENSION=0
HEADLESS=false
EPISODES=50000

while [[ $# -gt 0 ]]; do
    case $1 in
        --headless) HEADLESS=true; shift ;;
        --workers) WORKERS=$2; shift 2 ;;
        --sims) SIMS=$2; shift 2 ;;
        --ascension) ASCENSION=$2; shift 2 ;;
        --episodes) EPISODES=$2; shift 2 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

# Cleanup function
cleanup() {
    echo "Shutting down..."
    kill $(jobs -p) 2>/dev/null
    # Kill any lingering processes on our ports
    lsof -ti:8080 | xargs kill 2>/dev/null || true
    lsof -ti:5174 | xargs kill 2>/dev/null || true
    echo "Done."
}
trap cleanup EXIT

# Kill any existing processes on our ports
lsof -ti:8080 | xargs kill 2>/dev/null || true
lsof -ti:5174 | xargs kill 2>/dev/null || true
sleep 1

mkdir -p logs

echo "=== Slay the Spire RL Training ==="
echo "Workers: $WORKERS | Sims: $SIMS | Ascension: $ASCENSION | Episodes: $EPISODES"
echo ""

if [ "$HEADLESS" = false ]; then
    # Start viz dev server
    echo "Starting viz server..."
    cd packages/viz && bun dev --port 5174 &
    cd ../..
    sleep 2

    # Start WebSocket server
    echo "Starting WebSocket server..."
    uv run python -m packages.server --port 8080 &
    sleep 2

    echo "Dashboard: http://localhost:5174"
    echo ""
fi

# Start training
echo "Starting training..."
uv run python -c "
from packages.training.self_play import SelfPlayTrainer
import logging, sys

logging.basicConfig(level=logging.INFO, format='%(asctime)s | %(message)s', datefmt='%H:%M:%S',
                    handlers=[logging.StreamHandler(), logging.FileHandler('logs/training.log')])

trainer = SelfPlayTrainer(num_workers=$WORKERS, combat_sims=$SIMS, deep_sims=$((SIMS*2)), ascension=$ASCENSION)
print(f'Model: {sum(p.numel() for p in trainer.model.parameters()):,} params on {trainer.device}')
trainer.run(total_episodes=$EPISODES, games_per_batch=$((WORKERS*2)))
print(f'Done! Wins: {trainer.total_wins}/{trainer.total_games}')
"
