#!/bin/bash
# Query training status via WebSocket server.
# Usage: ./scripts/training-status.sh [--port 8080]
# Outputs JSON status. Returns {"running": false} if server not reachable.

PORT=8080
while [[ $# -gt 0 ]]; do
    case $1 in
        --port) PORT=$2; shift 2 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

cd "$(dirname "$0")/.."

uv run python -c "
import asyncio, websockets, json, sys

async def get_status():
    try:
        async with websockets.connect('ws://localhost:$PORT') as ws:
            await ws.send(json.dumps({'type': 'get_status'}))
            try:
                msg = await asyncio.wait_for(ws.recv(), timeout=2)
                data = json.loads(msg)
                # If server replies with error or unknown type, still report running
                if data.get('type') == 'error':
                    print(json.dumps({'running': True, 'note': data.get('error', 'server running')}))
                else:
                    print(msg)
            except asyncio.TimeoutError:
                print(json.dumps({'running': True, 'note': 'no status response in 2s'}))
    except Exception as e:
        print(json.dumps({'running': False}))

asyncio.run(get_status())
"
