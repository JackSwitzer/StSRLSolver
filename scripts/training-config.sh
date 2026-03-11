#!/bin/bash
# Send control commands to the training server via WebSocket.
# Usage:
#   ./scripts/training-config.sh --pause
#   ./scripts/training-config.sh --resume
#   ./scripts/training-config.sh --stop
#   ./scripts/training-config.sh --start [--workers N] [--sims N] [--ascension N]
#   ./scripts/training-config.sh --set sims=64
#   ./scripts/training-config.sh --set lr=1e-4 --set sims=32

PORT=8080
CMD=""
WORKERS=4
SIMS=64
ASCENSION=20
SEED="Test123"
EXTRA_FIELDS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --port) PORT=$2; shift 2 ;;
        --pause) CMD="training_pause"; shift ;;
        --resume) CMD="training_resume"; shift ;;
        --stop) CMD="training_stop"; shift ;;
        --start) CMD="training_start"; shift ;;
        --workers) WORKERS=$2; shift 2 ;;
        --sims) SIMS=$2; shift 2 ;;
        --ascension) ASCENSION=$2; shift 2 ;;
        --seed) SEED=$2; shift 2 ;;
        --set)
            KEY="${2%%=*}"
            VAL="${2#*=}"
            EXTRA_FIELDS="$EXTRA_FIELDS, \"$KEY\": \"$VAL\""
            shift 2 ;;
        -h|--help)
            echo "Usage: ./scripts/training-config.sh [--pause|--resume|--stop|--start]"
            echo "       [--workers N] [--sims N] [--ascension N] [--seed STR]"
            echo "       [--set key=value ...]"
            exit 0 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

if [ -z "$CMD" ]; then
    echo '{"error": "No command specified. Use --pause, --resume, --stop, or --start."}'
    exit 1
fi

cd "$(dirname "$0")/.."

# Build the message JSON
if [ "$CMD" = "training_start" ] || [ "$CMD" = "training_resume" ]; then
    MSG="{\"type\": \"$CMD\", \"config\": {\"num_agents\": $WORKERS, \"mcts_sims\": $SIMS, \"ascension\": $ASCENSION, \"seed\": \"$SEED\"$EXTRA_FIELDS}}"
else
    MSG="{\"type\": \"$CMD\"$EXTRA_FIELDS}"
fi

uv run python -c "
import asyncio, websockets, json, sys

async def send_command():
    try:
        async with websockets.connect('ws://localhost:$PORT') as ws:
            await ws.send('''$MSG''')
            try:
                msg = await asyncio.wait_for(ws.recv(), timeout=3)
                print(msg)
            except asyncio.TimeoutError:
                print(json.dumps({'sent': True, 'note': 'no response in 3s'}))
    except Exception as e:
        print(json.dumps({'error': str(e), 'running': False}))

asyncio.run(send_command())
"
