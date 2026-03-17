#!/bin/bash
# Training runner with process management, caffeinate, and monitoring.
#
# Usage:
#   ./scripts/training.sh start [--games N] [--workers N] [--batch N] [--asc N] [--headless]
#   ./scripts/training.sh stop          # Graceful shutdown (SIGTERM → wait → SIGKILL)
#   ./scripts/training.sh status        # Read status.json + tail log
#   ./scripts/training.sh resume        # Resume from latest checkpoint
#
# Process management:
#   - caffeinate -dims prevents Mac sleep during training
#   - Process group tracking (kills entire pgroup on stop)
#   - SIGTERM → 10s wait → SIGKILL
#   - PID file at .run/training.pid

set -e
cd "$(dirname "$0")/.."

PID_DIR=".run"
LOG_DIR="logs/training"
STATUS_FILE="$LOG_DIR/status.json"

mkdir -p "$PID_DIR" "$LOG_DIR"

# ── Helpers ─────────────────────────────────────────────

training_alive() {
    [ -f "$PID_DIR/training.pid" ] && kill -0 "$(cat "$PID_DIR/training.pid")" 2>/dev/null
}

stop_training() {
    if [ ! -f "$PID_DIR/training.pid" ]; then
        echo "Training: not running"
        return 0
    fi

    local pid
    pid=$(cat "$PID_DIR/training.pid")

    if ! kill -0 "$pid" 2>/dev/null; then
        echo "Training: not running (stale PID file)"
        rm -f "$PID_DIR/training.pid" "$PID_DIR/caffeinate.pid"
        return 0
    fi

    echo "Stopping training (PID $pid)..."

    local pgid
    pgid=$(ps -o pgid= -p "$pid" 2>/dev/null | tr -d ' ')

    # Send SIGTERM to main process (triggers graceful shutdown + checkpoint save)
    kill -TERM "$pid" 2>/dev/null

    # Wait up to 30 seconds for graceful shutdown
    echo "  Waiting up to 30s for graceful shutdown..."
    for i in $(seq 1 30); do
        kill -0 "$pid" 2>/dev/null || break
        sleep 1
    done

    # Force kill entire process group if still alive
    if kill -0 "$pid" 2>/dev/null; then
        echo "  Force killing process group..."
        if [ -n "$pgid" ] && [ "$pgid" != "0" ]; then
            kill -9 -- -"$pgid" 2>/dev/null || true
        fi
        kill -9 "$pid" 2>/dev/null || true
    fi

    # Stop caffeinate
    if [ -f "$PID_DIR/caffeinate.pid" ]; then
        local caf_pid
        caf_pid=$(cat "$PID_DIR/caffeinate.pid")
        kill "$caf_pid" 2>/dev/null || true
        rm -f "$PID_DIR/caffeinate.pid"
    fi

    rm -f "$PID_DIR/training.pid"

    # Clean up any orphaned worker processes from this project
    local orphans
    orphans=$(pgrep -f "packages.training.overnight\|multiprocessing.spawn.*spawn_main" 2>/dev/null | grep -v "^$$" || true)
    if [ -n "$orphans" ]; then
        echo "  Cleaning up $(echo "$orphans" | wc -l | tr -d ' ') orphaned workers..."
        echo "$orphans" | xargs kill 2>/dev/null || true
        sleep 1
        echo "$orphans" | xargs kill -9 2>/dev/null || true
    fi

    # Kill any stale caffeinate processes (keep system ones)
    pkill -f "caffeinate -dims" 2>/dev/null || true

    echo "Training stopped."
}

# ── Commands ────────────────────────────────────────────

cmd_start() {
    if training_alive; then
        local pid
        pid=$(cat "$PID_DIR/training.pid")
        echo "Training already running (PID $pid). Use 'stop' first."
        exit 1
    fi

    # Parse args
    local games=10000 workers=8 batch=256 asc=0 headless="" resume=""
    while [[ $# -gt 0 ]]; do
        case $1 in
            --games)   games=$2; shift 2 ;;
            --workers) workers=$2; shift 2 ;;
            --batch)   batch=$2; shift 2 ;;
            --asc)     asc=$2; shift 2 ;;
            --headless) headless="--headless-after 0"; shift ;;
            --resume)  resume="--resume $2"; shift 2 ;;
            *) echo "Unknown option: $1"; exit 1 ;;
        esac
    done

    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    local run_log="$LOG_DIR/run_${timestamp}.log"

    echo "Starting training..."
    echo "  Games: $games | Workers: $workers | Batch: $batch | Ascension: $asc"
    echo "  Log: $run_log"

    # Start caffeinate to prevent sleep
    caffeinate -dims &
    echo $! > "$PID_DIR/caffeinate.pid"
    echo "  caffeinate: PID $!"

    # Start training (macOS doesn't have setsid, use nohup instead)
    nohup uv run python -m packages.training.overnight \
        --games "$games" \
        --workers "$workers" \
        --batch-size "$batch" \
        --ascension "$asc" \
        $headless \
        $resume \
        > "$run_log" 2>&1 &

    local train_pid=$!
    echo "$train_pid" > "$PID_DIR/training.pid"
    echo "  training: PID $train_pid"
    echo ""
    echo "Monitor with: ./scripts/training.sh status"
    echo "Stop with:    ./scripts/training.sh stop"
}

cmd_stop() {
    stop_training
}

cmd_status() {
    echo "=== Training Status ==="

    if training_alive; then
        local pid
        pid=$(cat "$PID_DIR/training.pid")
        echo "State: RUNNING (PID $pid)"

        # Show caffeinate
        if [ -f "$PID_DIR/caffeinate.pid" ]; then
            local caf_pid
            caf_pid=$(cat "$PID_DIR/caffeinate.pid")
            if kill -0 "$caf_pid" 2>/dev/null; then
                echo "Sleep prevention: active (caffeinate PID $caf_pid)"
            fi
        fi

        # Worker count
        local worker_count
        worker_count=$(pgrep -P "$pid" 2>/dev/null | wc -l | tr -d ' ')
        echo "Workers: $worker_count"
    else
        echo "State: STOPPED"
        rm -f "$PID_DIR/training.pid"
    fi

    echo ""

    # Find the most recent status.json (check weekend-run and training dirs)
    local status_file=""
    for sf in "logs/weekend-run/status.json" "$STATUS_FILE"; do
        if [ -f "$sf" ]; then
            if [ -z "$status_file" ] || [ "$sf" -nt "$status_file" ]; then
                status_file="$sf"
            fi
        fi
    done

    if [ -n "$status_file" ]; then
        echo "--- Latest Status ($status_file) ---"
        python3 -c "
import json, sys
with open('$status_file') as f:
    s = json.load(f)
print(f'Games:       {s.get(\"total_games\", \"?\")}')
print(f'Wins:        {s.get(\"total_wins\", \"?\")}')
print(f'Win Rate:    {s.get(\"win_rate_100\", s.get(\"win_rate\", \"?\"))}%')
print(f'Avg Floor:   {s.get(\"avg_floor_100\", s.get(\"avg_floor\", \"?\"))}')
print(f'Train Steps: {s.get(\"train_steps\", \"?\")}')
print(f'Games/min:   {s.get(\"games_per_min\", \"?\")}')
print(f'Elapsed:     {s.get(\"elapsed_hours\", \"?\")}h')
print(f'Config:      {s.get(\"config_name\", \"?\")}')
" 2>/dev/null || echo "(status.json parse error)"
    else
        echo "(no status.json yet)"
    fi

    echo ""

    # Tail latest log (check weekend nohup.log and training run logs)
    local latest_log=""
    local weekend_log="logs/weekend-run/nohup.log"
    local training_log
    training_log=$(ls -t "$LOG_DIR"/run_*.log 2>/dev/null | head -1)

    if [ -f "$weekend_log" ] && [ -f "$training_log" ]; then
        # Use whichever was modified more recently
        if [ "$weekend_log" -nt "$training_log" ]; then
            latest_log="$weekend_log"
        else
            latest_log="$training_log"
        fi
    elif [ -f "$weekend_log" ]; then
        latest_log="$weekend_log"
    elif [ -n "$training_log" ]; then
        latest_log="$training_log"
    fi

    if [ -n "$latest_log" ]; then
        echo "--- Last 10 log lines ($latest_log) ---"
        tail -10 "$latest_log"
    fi
}

cmd_weekend() {
    if training_alive; then
        local pid
        pid=$(cat "$PID_DIR/training.pid")
        echo "Training already running (PID $pid). Use 'stop' first."
        exit 1
    fi

    # Parse args (weekend has larger defaults)
    local games=500000 workers=12 batch=256 asc=0 extra_args=""
    while [[ $# -gt 0 ]]; do
        case $1 in
            --games)   games=$2; shift 2 ;;
            --workers) workers=$2; shift 2 ;;
            --batch)   batch=$2; shift 2 ;;
            --asc)     asc=$2; shift 2 ;;
            *) echo "Unknown option: $1"; exit 1 ;;
        esac
    done

    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    local run_dir="logs/weekend-run"
    local run_log="$run_dir/nohup.log"

    mkdir -p "$run_dir"

    echo "Starting weekend training run..."
    echo "  Games: $games | Workers: $workers | Batch: $batch | Ascension: $asc"
    echo "  Run dir: $run_dir"
    echo "  Log: $run_log"

    # Start caffeinate to prevent sleep (display + idle + system + disk)
    caffeinate -dims &
    echo $! > "$PID_DIR/caffeinate.pid"
    echo "  caffeinate: PID $!"

    # Launch training headless with dedicated run-dir
    # Uses larger model (7M params) and bigger inference batches
    # Check for shutdown checkpoint to auto-resume
    local resume_flag=""
    if [ -f "$run_dir/shutdown_checkpoint.pt" ]; then
        resume_flag="--resume $run_dir/shutdown_checkpoint.pt"
        echo "  Resuming from shutdown checkpoint"
    fi

    nohup uv run python -m packages.training.overnight \
        --games "$games" \
        --workers "$workers" \
        --batch 24 \
        --batch-size "$batch" \
        --ascension "$asc" \
        --headless-after 0 \
        --run-dir "$run_dir" \
        --hidden-dim 1024 \
        --num-blocks 6 \
        --max-batch-size 128 \
        $resume_flag \
        > "$run_log" 2>&1 &

    local train_pid=$!
    echo "$train_pid" > "$PID_DIR/training.pid"
    echo "  training: PID $train_pid"
    echo ""
    echo "Monitor with: ./scripts/training.sh status"
    echo "Stop with:    ./scripts/training.sh stop"
    echo ""
    echo "Weekend mode: headless, caffeinated, long-running."
}

cmd_resume() {
    if training_alive; then
        echo "Training already running. Stop first."
        exit 1
    fi

    local latest_checkpoint
    latest_checkpoint=$(ls -t logs/overnight/*/checkpoint_*.pt 2>/dev/null | head -1)
    if [ -z "$latest_checkpoint" ]; then
        echo "No checkpoint found. Use 'start' instead."
        exit 1
    fi

    echo "Resuming from: $latest_checkpoint"
    cmd_start --resume "$latest_checkpoint" "$@"
}

cmd_archive() {
    # Archive the current weekend-run to a timestamped directory
    local run_dir="logs/weekend-run"
    local label="${1:-}"
    local timestamp
    timestamp=$(date +%Y%m%d_%H%M)
    local archive_name="run_${timestamp}"
    [ -n "$label" ] && archive_name="${archive_name}_${label}"
    local archive_dir="logs/runs/${archive_name}"

    if [ ! -d "$run_dir" ]; then
        echo "No weekend-run directory to archive."
        return 1
    fi

    # Stop training if running
    if training_alive; then
        echo "Stopping training before archive..."
        stop_training
    fi

    mkdir -p "$archive_dir"

    # Move checkpoints and model files
    for f in shutdown_checkpoint.pt periodic_checkpoint.pt final_strategic.pt; do
        [ -f "$run_dir/$f" ] && mv "$run_dir/$f" "$archive_dir/"
    done

    # Copy (not move) status, episodes, summary, logs
    for f in status.json recent_episodes.json summary.json episodes.jsonl; do
        [ -f "$run_dir/$f" ] && cp "$run_dir/$f" "$archive_dir/"
    done

    # Archive all logs
    for f in "$run_dir"/nohup*.log; do
        [ -f "$f" ] && mv "$f" "$archive_dir/"
    done

    # Copy best trajectories if they exist (keep originals for next distillation)
    if [ -d "$run_dir/best_trajectories" ]; then
        local traj_count
        traj_count=$(ls "$run_dir/best_trajectories/"*.npz 2>/dev/null | wc -l | tr -d ' ')
        echo "  Trajectories: $traj_count files (kept in place for next distillation)"
    fi

    # Clean stale data for fresh start
    rm -f "$run_dir/status.json" "$run_dir/recent_episodes.json" "$run_dir/summary.json"

    echo "Archived to: $archive_dir"
    ls -lh "$archive_dir/" 2>/dev/null | tail -20
    echo ""
    echo "Ready for fresh start: ./scripts/training.sh weekend"
}

cmd_quick_restart() {
    echo "Quick restart: stop → save → restart with code changes..."
    local was_running=false
    if training_alive; then
        was_running=true
        stop_training
    fi

    # Rename old log
    if [ -f "logs/weekend-run/nohup.log" ]; then
        local n=1
        while [ -f "logs/weekend-run/nohup_prev${n}.log" ]; do n=$((n+1)); done
        mv "logs/weekend-run/nohup.log" "logs/weekend-run/nohup_prev${n}.log"
        echo "  Old log → nohup_prev${n}.log"
    fi

    echo "  Restarting weekend mode (auto-resumes from checkpoint)..."
    cmd_weekend "$@"
}

# ── Main ────────────────────────────────────────────────

case "${1:-status}" in
    start)   shift; cmd_start "$@" ;;
    stop)    cmd_stop ;;
    status)  cmd_status ;;
    resume)  shift; cmd_resume "$@" ;;
    weekend) shift; cmd_weekend "$@" ;;
    restart) shift; cmd_quick_restart "$@" ;;
    archive) shift; cmd_archive "$@" ;;
    fresh)   shift; cmd_archive "${1:-fresh}" && cmd_weekend "${@:2}" ;;
    update)  shift; echo "Pulling latest code..."; git pull --ff-only && cmd_quick_restart "$@" ;;
    hotfix)  shift; ./scripts/hotfix.sh "$@" ;;
    prune)   shift; uv run python scripts/utils/prune_data.py "$@" ;;
    *)
        echo "Usage: $0 {start|stop|status|resume|weekend|restart|update|hotfix|prune} [options]"
        echo ""
        echo "Commands:"
        echo "  start      Start training (default 10K games)"
        echo "  stop       Graceful shutdown (SIGTERM → 30s → SIGKILL)"
        echo "  status     Read status.json + tail log"
        echo "  resume     Resume from latest checkpoint"
        echo "  weekend    Long-running headless mode (default 500K games)"
        echo "  restart    Quick restart: stop, save checkpoint, resume with code changes"
        echo "  archive    Archive current run to logs/runs/run_TIMESTAMP/ (optional label arg)"
        echo "  fresh      Archive + start fresh (cold start with distillation from trajectories)"
        echo "  update     Pull latest code + restart (git pull → restart)"
        echo "  hotfix     Live parameter tuning via SIGUSR1 + reload.json"
        echo "  prune      Prune episodes.jsonl + consolidate top runs (safe while running)"
        echo ""
        echo "Options for start/weekend:"
        echo "  --games N      Total games to play"
        echo "  --workers N    Parallel workers (default: 24)"
        echo "  --batch N      Batch size for PPO (default: 256)"
        echo "  --asc N        Ascension level (default: 0)"
        echo "  --headless     No dashboard output (start only)"
        echo ""
        echo "Options for prune:"
        echo "  --dry-run          Preview without modifying files"
        echo "  --keep N           Keep last N episodes (default: 10000)"
        echo "  --top N            Top N episodes in top_episodes.json (default: 500)"
        echo "  --skip-compress    Skip episode archiving"
        echo "  --skip-top         Skip top episode consolidation"
        exit 1
        ;;
esac
