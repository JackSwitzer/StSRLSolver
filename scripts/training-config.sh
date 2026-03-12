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
ACTION=""
WORKERS=4
WORKERS_SET=0
SIMS=64
SIMS_SET=0
ASCENSION=20
ASCENSION_SET=0
SEED="Test123"
PARAM_ENTRIES=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --port) PORT=$2; shift 2 ;;
        --pause) CMD="command"; ACTION="pause"; shift ;;
        --resume) CMD="command"; ACTION="resume"; shift ;;
        --stop) CMD="command"; ACTION="stop"; shift ;;
        --start) CMD="training_start"; shift ;;
        --workers) WORKERS=$2; WORKERS_SET=1; shift 2 ;;
        --sims) SIMS=$2; SIMS_SET=1; shift 2 ;;
        --ascension) ASCENSION=$2; ASCENSION_SET=1; shift 2 ;;
        --seed) SEED=$2; shift 2 ;;
        --set)
            KEY="${2%%=*}"
            VAL="${2#*=}"
            [ "$KEY" = "num_agents" ] && KEY="workers"
            [ "$KEY" = "mcts_sims" ] && KEY="sims"
            PARAM_ENTRIES="${PARAM_ENTRIES}${KEY}=${VAL}"$'\n'
            if [ -z "$CMD" ]; then
                CMD="command"
                ACTION="set_config"
            fi
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
    echo '{"error": "No command specified. Use --pause, --resume, --stop, --start, or --set."}'
    exit 1
fi

cd "$(dirname "$0")/.."

MSG=$(
    CMD="$CMD" \
    ACTION="$ACTION" \
    WORKERS="$WORKERS" \
    WORKERS_SET="$WORKERS_SET" \
    SIMS="$SIMS" \
    SIMS_SET="$SIMS_SET" \
    ASCENSION="$ASCENSION" \
    ASCENSION_SET="$ASCENSION_SET" \
    SEED="$SEED" \
    PARAM_ENTRIES="$PARAM_ENTRIES" \
    python3 - <<'PY'
import json
import os


def parse_value(raw: str):
    try:
        return json.loads(raw)
    except json.JSONDecodeError:
        return raw


cmd = os.environ["CMD"]
msg = {"type": cmd}

if cmd == "training_start":
    msg["config"] = {
        "num_agents": int(os.environ["WORKERS"]),
        "mcts_sims": int(os.environ["SIMS"]),
        "ascension": int(os.environ["ASCENSION"]),
        "seed": os.environ["SEED"],
    }
elif cmd == "command":
    action = os.environ["ACTION"]
    msg["action"] = action
    if action == "set_config":
        params = {}
        if os.environ["WORKERS_SET"] == "1":
            params["workers"] = int(os.environ["WORKERS"])
        if os.environ["SIMS_SET"] == "1":
            params["sims"] = int(os.environ["SIMS"])
        if os.environ["ASCENSION_SET"] == "1":
            params["ascension"] = int(os.environ["ASCENSION"])

        for entry in os.environ["PARAM_ENTRIES"].splitlines():
            if not entry:
                continue
            key, value = entry.split("=", 1)
            params[key] = parse_value(value)

        msg["params"] = params

print(json.dumps(msg))
PY
)

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
