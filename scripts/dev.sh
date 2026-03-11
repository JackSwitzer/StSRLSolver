#!/bin/bash
# Start development environment (viz + WS server + optional training)
# Usage:
#   ./scripts/dev.sh              # Viz + WS server only (for watching)
#   ./scripts/dev.sh --train      # + training with defaults
#   ./scripts/dev.sh --train --workers 8 --sims 32 --ascension 0

set -e
cd "$(dirname "$0")/.."

TRAIN=false
WORKERS=8
SIMS=32
ASCENSION=0
EPISODES=50000
WS_PORT=8080
VIZ_PORT=5174

while [[ $# -gt 0 ]]; do
    case $1 in
        --train) TRAIN=true; shift ;;
        --workers) WORKERS=$2; shift 2 ;;
        --sims) SIMS=$2; shift 2 ;;
        --ascension) ASCENSION=$2; shift 2 ;;
        --episodes) EPISODES=$2; shift 2 ;;
        --ws-port) WS_PORT=$2; shift 2 ;;
        --viz-port) VIZ_PORT=$2; shift 2 ;;
        -h|--help)
            echo "Usage: ./scripts/dev.sh [--train] [--workers N] [--sims N] [--ascension N]"
            exit 0 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

# Cleanup on exit
cleanup() {
    echo ""
    echo "Shutting down..."
    kill $(jobs -p) 2>/dev/null || true
    wait 2>/dev/null
    echo "Done."
}
trap cleanup EXIT INT TERM

# Kill stale processes on our ports
for port in $WS_PORT $VIZ_PORT; do
    pid=$(lsof -ti:$port 2>/dev/null)
    if [ -n "$pid" ]; then
        echo "Killing stale process on port $port (PID $pid)"
        kill $pid 2>/dev/null || true
    fi
done
sleep 1

mkdir -p logs

echo "╔══════════════════════════════════════════╗"
echo "║   Slay the Spire RL — Dev Environment    ║"
echo "╠══════════════════════════════════════════╣"
echo "║  Viz:     http://localhost:$VIZ_PORT          ║"
echo "║  WS:      ws://localhost:$WS_PORT              ║"
if [ "$TRAIN" = true ]; then
echo "║  Training: $WORKERS workers, $SIMS sims, A$ASCENSION      ║"
fi
echo "╚══════════════════════════════════════════╝"
echo ""

# Start viz
echo "[1/3] Starting viz server on port $VIZ_PORT..."
(cd packages/viz && bun dev --port $VIZ_PORT) &
sleep 2

# Start WS server
echo "[2/3] Starting WebSocket server on port $WS_PORT..."
uv run python -m packages.server --port $WS_PORT &
sleep 2

if [ "$TRAIN" = true ]; then
    echo "[3/3] Starting training ($WORKERS workers, $SIMS sims, A$ASCENSION)..."
    uv run python -c "
from packages.training.self_play import SelfPlayTrainer
import logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s | %(message)s', datefmt='%H:%M:%S',
                    handlers=[logging.StreamHandler(), logging.FileHandler('logs/training.log')])
trainer = SelfPlayTrainer(num_workers=$WORKERS, combat_sims=$SIMS, deep_sims=$((SIMS*2)), ascension=$ASCENSION)
print(f'Model: {sum(p.numel() for p in trainer.model.parameters()):,} params on {trainer.device}')
trainer.run(total_episodes=$EPISODES, games_per_batch=$((WORKERS*2)))
" &
fi

echo ""
echo "Press Ctrl+C to stop all services."
wait
