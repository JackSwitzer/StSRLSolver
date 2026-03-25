#!/bin/bash
# Graceful pause: kill training + workers + shared memory + ports
# Usage: bash scripts/pause.sh [run_name]
# Defaults to latest run via logs/active/

set -e

RUN_NAME="${1:-$(readlink logs/active 2>/dev/null | xargs basename 2>/dev/null || echo 'unknown')}"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

echo "=== GRACEFUL PAUSE: $RUN_NAME ==="
echo ""

# 1. Find and kill main training process
KILLED_MAIN=0
for pidfile in logs/*.pid logs/*/*.pid; do
    [ -f "$pidfile" ] || continue
    PID=$(cat "$pidfile" 2>/dev/null)
    if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
        echo "[1/7] Sending SIGTERM to PID $PID ($(basename $pidfile))..."
        kill -TERM "$PID" 2>/dev/null || true
        KILLED_MAIN=1
    fi
done

if [ $KILLED_MAIN -eq 0 ]; then
    echo "[1/7] No active training PID files found"
fi

# 2. Wait for graceful shutdown (up to 30s)
echo "[2/7] Waiting up to 30s for graceful shutdown..."
for i in $(seq 1 30); do
    ALIVE=0
    for pidfile in logs/*.pid logs/*/*.pid; do
        [ -f "$pidfile" ] || continue
        PID=$(cat "$pidfile" 2>/dev/null)
        if [ -n "$PID" ] && kill -0 "$PID" 2>/dev/null; then
            ALIVE=1
        fi
    done
    [ $ALIVE -eq 0 ] && break
    sleep 1
done

# 3. Force kill any remaining python workers
echo "[3/7] Killing orphan python workers..."
WORKERS=$(pgrep -f "multiprocessing.spawn\|_play_one_game\|InferenceServer" 2>/dev/null || true)
if [ -n "$WORKERS" ]; then
    echo "  Found $(echo "$WORKERS" | wc -l | tr -d ' ') orphan processes"
    echo "$WORKERS" | xargs kill -9 2>/dev/null || true
else
    echo "  No orphans found"
fi

# 4. Clean up shared memory
echo "[4/7] Cleaning shared memory..."
SHM_COUNT=0
for shm in /dev/shm/si_* /tmp/shm_*; do
    if [ -e "$shm" ]; then
        rm -f "$shm" && SHM_COUNT=$((SHM_COUNT + 1))
    fi
done
# Also clean via Python multiprocessing resource tracker
python3 -c "
import multiprocessing.resource_tracker as rt
import os, glob
cleaned = 0
for f in glob.glob('/dev/shm/*'):
    if 'si_' in f:
        try: os.unlink(f); cleaned += 1
        except: pass
print(f'  Cleaned {cleaned} shared memory segments')
" 2>/dev/null || echo "  Cleaned $SHM_COUNT segments via shell"

# 5. Release ports (kill any stale listeners)
echo "[5/7] Checking for stale ports..."
STALE_PORTS=$(lsof -i -P -n 2>/dev/null | grep python | grep LISTEN | awk '{print $2}' | sort -u || true)
if [ -n "$STALE_PORTS" ]; then
    echo "  Found stale port listeners: $STALE_PORTS"
    echo "$STALE_PORTS" | xargs kill -9 2>/dev/null || true
else
    echo "  No stale ports"
fi

# 5b. Update status.json to reflect paused state
echo "[5b] Updating status.json..."
python3 -c "
import json
from pathlib import Path
from datetime import datetime
status_path = Path('logs/active/status.json')
if status_path.exists():
    d = json.loads(status_path.read_text())
    d['timestamp'] = datetime.now().isoformat()
    d['gpu_percent'] = 0
    d['sweep_phase'] = 'paused'
    d['config_name'] = 'PAUSED'
    d['games_per_min'] = 0
    status_path.write_text(json.dumps(d, indent=2))
    print('  status.json updated: paused')
else:
    print('  No status.json found')
" 2>/dev/null || echo "  Could not update status.json"

# 6. Verify checkpoint saved
echo "[6/7] Checking for saved checkpoints..."
LATEST_CKPT=$(ls -t logs/strategic_checkpoints/*.pt logs/*/concurrent_*.pt 2>/dev/null | head -1)
if [ -n "$LATEST_CKPT" ]; then
    echo "  Latest checkpoint: $LATEST_CKPT ($(stat -f%z "$LATEST_CKPT" 2>/dev/null || stat -c%s "$LATEST_CKPT" 2>/dev/null) bytes)"
else
    echo "  WARNING: No checkpoint found!"
fi

# 7. Archive logs
echo "[7/7] Archiving logs..."
if [ -d "logs/active" ] || [ -L "logs/active" ]; then
    ACTIVE_DIR=$(readlink -f logs/active 2>/dev/null || readlink logs/active 2>/dev/null)
    if [ -d "$ACTIVE_DIR" ]; then
        ARCHIVE="logs/archive_${RUN_NAME}_${TIMESTAMP}"
        cp -r "$ACTIVE_DIR" "$ARCHIVE" 2>/dev/null && echo "  Archived to $ARCHIVE" || echo "  Archive failed (dir may be large)"
    fi
fi

# Clean PID files
rm -f logs/*.pid logs/*/*.pid 2>/dev/null

# Final resource report
echo ""
echo "=== RESOURCE REPORT ==="
echo "Python processes: $(pgrep -c python 2>/dev/null || echo 0)"
echo "GPU memory: $(python3 -c 'import torch; print("MPS available" if torch.backends.mps.is_available() else "N/A")' 2>/dev/null || echo 'N/A')"
echo "RAM free: $(vm_stat 2>/dev/null | awk '/Pages free/ {printf "%.0f MB", $3*16384/1048576}')"
echo "Disk (logs): $(du -sh logs/ 2>/dev/null | cut -f1)"
echo ""
echo "=== PAUSE COMPLETE ==="
