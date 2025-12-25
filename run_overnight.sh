#!/bin/bash
# Overnight BC training - runs until stopped
# Logs to logs/overnight.log
# Checkpoints saved every 50 epochs

cd /Users/jackswitzer/Desktop/StSRLSolver
mkdir -p checkpoints logs

echo "Starting overnight training at $(date)"
echo "Log file: logs/overnight.log"
echo "Checkpoints: checkpoints/overnight/"
echo ""
echo "To monitor: tail -f logs/overnight.log"
echo "To stop: pkill -f train_bc.py"
echo ""

nohup /Users/jackswitzer/.local/bin/uv run python3 train_bc.py \
    --data data/watcher_training/watcher_wins_nov2020.json \
    --output checkpoints/overnight \
    --epochs 2000 \
    --batch-size 128 \
    --lr 0.001 \
    --device mps \
    > logs/overnight.log 2>&1 &

echo "Training PID: $!"
echo "Training started! You can close this terminal."
