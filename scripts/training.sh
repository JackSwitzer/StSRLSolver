#!/bin/bash
# Training runner with process management, caffeinate, and monitoring.
#
# Usage:
#   ./scripts/training.sh start [--games N] [--workers N] [--batch N] [--asc N] [--headless]
#   ./scripts/training.sh stop          # Graceful shutdown (SIGTERM -> wait -> SIGKILL)
#   ./scripts/training.sh status        # Read logs/active/status.json + tail log
#   ./scripts/training.sh resume        # Resume from latest checkpoint
#
# Run directory convention:
#   - Each run creates logs/runs/run_YYYYMMDD_HHMMSS[_label]/
#   - logs/active is a symlink to the current run directory
#   - All tools read from logs/active/ (single source of truth)
#
# Process management:
#   - caffeinate -dims prevents Mac sleep during training
#   - Process group tracking (kills entire pgroup on stop)
#   - SIGTERM -> 10s wait -> SIGKILL
#   - PID file at .run/training.pid

set -e
cd "$(dirname "$0")/.."

PID_DIR=".run"
ACTIVE_LINK="logs/active"

mkdir -p "$PID_DIR" "logs/runs"

# -- Helpers -------------------------------------------------------

create_run_dir() {
    local label="${1:-}"
    local timestamp
    timestamp=$(date +%Y%m%d_%H%M%S)
    local name="run_${timestamp}"
    [ -n "$label" ] && name="${name}_${label}"
    local dir="logs/runs/${name}"
    mkdir -p "$dir"
    # Symlink target must be relative to the symlink's parent dir (logs/)
    ln -sfn "runs/${name}" "$ACTIVE_LINK"
    echo "$dir"
}

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

    # Stop watchdog
    if [ -f "$PID_DIR/watchdog.pid" ]; then
        local wd_pid
        wd_pid=$(cat "$PID_DIR/watchdog.pid")
        kill "$wd_pid" 2>/dev/null || true
        rm -f "$PID_DIR/watchdog.pid"
        echo "  Watchdog stopped."
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
    orphans=$(pgrep -f "packages.training.training_runner\|multiprocessing.spawn.*spawn_main" 2>/dev/null | grep -v "^$$" || true)
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

# -- Commands ------------------------------------------------------

cmd_start() {
    if training_alive; then
        local pid
        pid=$(cat "$PID_DIR/training.pid")
        echo "Training already running (PID $pid). Use 'stop' first."
        exit 1
    fi

    # Parse args
    local games=10000 workers=8 batch=256 asc=0 headless="" resume="" watchdog=""
    while [[ $# -gt 0 ]]; do
        case $1 in
            --games)   games=$2; shift 2 ;;
            --workers) workers=$2; shift 2 ;;
            --batch)   batch=$2; shift 2 ;;
            --asc)     asc=$2; shift 2 ;;
            --headless) headless="--headless-after 0"; shift ;;
            --resume)  resume="--resume $2"; shift 2 ;;
            --weekend) weekend="--weekend"; shift ;;
            --overnight) weekend="--overnight"; shift ;;
            --watchdog) watchdog="1"; shift ;;
            *) echo "Unknown option: $1"; exit 1 ;;
        esac
    done

    local run_dir
    run_dir=$(create_run_dir)
    local run_log="$run_dir/nohup.log"

    echo "Starting training..."
    echo "  Games: $games | Workers: $workers | Batch: $batch | Ascension: $asc"
    echo "  Run dir: $run_dir"
    echo "  Log: $run_log"

    # Start caffeinate to prevent sleep
    caffeinate -dims &
    echo $! > "$PID_DIR/caffeinate.pid"
    echo "  caffeinate: PID $!"

    # Start training (macOS doesn't have setsid, use nohup instead)
    nohup uv run python -m packages.training.training_runner \
        --games "$games" \
        --workers "$workers" \
        --batch-size "$batch" \
        --ascension "$asc" \
        --run-dir "$run_dir" \
        $headless \
        $resume \
        $weekend \
        > "$run_log" 2>&1 &

    local train_pid=$!
    echo "$train_pid" > "$PID_DIR/training.pid"
    echo "  training: PID $train_pid"

    # Start watchdog if requested
    if [ -n "$watchdog" ]; then
        nohup bash scripts/watchdog.sh > "$run_dir/watchdog.log" 2>&1 &
        echo $! > "$PID_DIR/watchdog.pid"
        echo "  watchdog: PID $!"
    fi

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

    # Read status from active run
    local status_file="$ACTIVE_LINK/status.json"
    if [ -f "$status_file" ]; then
        echo "--- Latest Status ($status_file) ---"
        uv run python -c "
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

    # Tail log from active run, fallback to latest archived run
    local latest_log="$ACTIVE_LINK/nohup.log"
    if [ ! -f "$latest_log" ]; then
        latest_log=$(ls -t logs/runs/run_*/nohup.log 2>/dev/null | head -1)
    fi

    if [ -n "$latest_log" ] && [ -f "$latest_log" ]; then
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

    # Check for checkpoint BEFORE creating new run dir (symlink moves)
    local resume_flag=""
    if [ -L "$ACTIVE_LINK" ] && [ -f "$ACTIVE_LINK/shutdown_checkpoint.pt" ]; then
        resume_flag="--resume $(cd "$ACTIVE_LINK" && pwd)/shutdown_checkpoint.pt"
        echo "  Will resume from shutdown checkpoint"
    fi

    local run_dir
    run_dir=$(create_run_dir "weekend")
    local run_log="$run_dir/nohup.log"

    echo "Starting weekend training run..."
    echo "  Games: $games | Workers: $workers | Batch: $batch | Ascension: $asc"
    echo "  Run dir: $run_dir"
    echo "  Log: $run_log"

    # Start caffeinate to prevent sleep (display + idle + system + disk)
    caffeinate -dims &
    echo $! > "$PID_DIR/caffeinate.pid"
    echo "  caffeinate: PID $!"

    nohup uv run python -m packages.training.training_runner \
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

    # Look in active run first, then scan archived runs
    local latest_checkpoint=""
    if [ -L "$ACTIVE_LINK" ]; then
        latest_checkpoint=$(ls -t "$ACTIVE_LINK"/checkpoint_*.pt "$ACTIVE_LINK"/shutdown_checkpoint.pt 2>/dev/null | head -1)
    fi
    if [ -z "$latest_checkpoint" ]; then
        latest_checkpoint=$(ls -t logs/runs/*/checkpoint_*.pt logs/runs/*/shutdown_checkpoint.pt 2>/dev/null | head -1)
    fi
    if [ -z "$latest_checkpoint" ]; then
        echo "No checkpoint found. Use 'start' instead."
        exit 1
    fi

    echo "Resuming from: $latest_checkpoint"
    cmd_start --resume "$latest_checkpoint" "$@"
}

cmd_archive() {
    # Archive the current active run
    local label="${1:-}"

    if [ ! -L "$ACTIVE_LINK" ]; then
        echo "No active run to archive (logs/active symlink not found)."
        return 1
    fi

    # Resolve the symlink to an absolute path
    local run_dir
    run_dir=$(cd "$(dirname "$ACTIVE_LINK")" && cd "$(readlink "$ACTIVE_LINK")" && pwd)

    if [ ! -d "$run_dir" ]; then
        echo "Active run directory does not exist: $run_dir"
        return 1
    fi

    # Stop training if running
    if training_alive; then
        echo "Stopping training before archive..."
        stop_training
    fi

    # If a label was provided, rename the run dir to include it
    if [ -n "$label" ]; then
        local base_name
        base_name=$(basename "$run_dir")
        # Only append label if not already in the name
        if [[ "$base_name" != *"_${label}" ]]; then
            local new_dir
            new_dir="$(dirname "$run_dir")/${base_name}_${label}"
            mv "$run_dir" "$new_dir"
            run_dir="$new_dir"
        fi
    fi

    echo "Archived: $run_dir"
    ls -lh "$run_dir/" 2>/dev/null | tail -20

    # Remove the active symlink
    rm -f "$ACTIVE_LINK"
    echo ""
    echo "Active symlink removed. Ready for fresh start."
}

cmd_archive_old() {
    mkdir -p logs/archive/pre_weekend_runs
    local count=0
    for d in logs/runs/run_2026031[78]_*; do
        [ -d "$d" ] && mv "$d" logs/archive/pre_weekend_runs/ && count=$((count + 1))
    done
    echo "Archived $count old run directories"
}

cmd_quick_restart() {
    echo "Quick restart: stop -> save -> restart with code changes..."
    local was_running=false
    if training_alive; then
        was_running=true
        stop_training
    fi

    # Rename old log in the active run dir
    if [ -f "$ACTIVE_LINK/nohup.log" ]; then
        local n=1
        while [ -f "$ACTIVE_LINK/nohup_prev${n}.log" ]; do n=$((n+1)); done
        mv "$ACTIVE_LINK/nohup.log" "$ACTIVE_LINK/nohup_prev${n}.log"
        echo "  Old log -> nohup_prev${n}.log"
    fi

    echo "  Restarting weekend mode (auto-resumes from checkpoint)..."
    cmd_weekend "$@"
}

# -- Main ----------------------------------------------------------

case "${1:-status}" in
    start)   shift; cmd_start "$@" ;;
    stop)    cmd_stop ;;
    status)  cmd_status ;;
    resume)  shift; cmd_resume "$@" ;;
    weekend) shift; cmd_weekend "$@" ;;
    restart) shift; cmd_quick_restart "$@" ;;
    archive)     shift; cmd_archive "$@" ;;
    archive-old) cmd_archive_old ;;
    fresh)       shift; cmd_archive "${1:-fresh}" && cmd_weekend "${@:2}" ;;
    update)  shift; echo "Pulling latest code..."; git pull --ff-only && cmd_quick_restart "$@" ;;
    hotfix)  shift; ./scripts/hotfix.sh "$@" ;;
    prune)   shift; uv run python scripts/utils/prune_data.py "$@" ;;
    data)
        shift
        case "${1:-inventory}" in
            inventory) shift; uv run python -m packages.training.data_utils_cli inventory "$@" ;;
            quality)   shift; uv run python -m packages.training.data_utils_cli quality "$@" ;;
            organize)  shift; uv run python -m packages.training.data_utils_cli organize "$@" ;;
            *) echo "Usage: $0 data [inventory|quality|organize]"; exit 1 ;;
        esac
        ;;
    pretrain)
        shift
        case "${1:---all}" in
            --bc)      shift; uv run python scripts/pretrain_bc.py "$@" ;;
            --combat)  shift; uv run python scripts/pretrain_combat.py "$@" ;;
            --eval)    shift; uv run python scripts/pretrain_eval.py "$@" ;;
            --all|"")
                echo "Running full pretrain pipeline: BC → CombatNet → Eval"
                uv run python scripts/pretrain_bc.py "$@" && \
                uv run python scripts/pretrain_combat.py && \
                uv run python scripts/pretrain_eval.py
                ;;
            *) echo "Usage: $0 pretrain [--bc|--combat|--eval|--all] [options]"; exit 1 ;;
        esac
        ;;
    algorithm)
        shift
        algo="${1:?Usage: $0 algorithm [ppo|iql|grpo]}"
        shift
        case "$algo" in
            ppo|iql|grpo)
                echo "Starting training with algorithm: $algo"
                uv run python -m packages.training.training_runner \
                    --algorithm "$algo" "$@"
                ;;
            *) echo "Unknown algorithm: $algo (choose ppo, iql, or grpo)"; exit 1 ;;
        esac
        ;;
    experiment)
        shift
        config_name="${1:?Usage: $0 experiment <config-name>}"
        shift
        case "$config_name" in
            reward-sim)
                echo "Running offline reward rescoring simulation..."
                uv run python -m packages.training.reward_sim "$@"
                ;;
            reward-ab)
                echo "Running reward A/B test (live games)..."
                uv run python -m packages.training.offline_eval "$@"
                ;;
            *)
                echo "Running experiment: $config_name"
                uv run python -m packages.training.training_runner \
                    --sweep-config "$config_name" "$@"
                ;;
        esac
        ;;
    push-metrics)
        shift; uv run python scripts/push_metrics.py "$@" ;;
    text)
        shift
        PHONE="+14166293183"
        RUN_DIR=$(ls -td logs/runs/run_* 2>/dev/null | head -1)
        if [ -z "$RUN_DIR" ]; then echo "No run directory found"; exit 1; fi
        STATUS=$(cat "$RUN_DIR/status.json" 2>/dev/null)
        EPISODES="$RUN_DIR/episodes.jsonl"
        GAMES=$(echo "$STATUS" | jq -r '.total_games // 0')

        # Min 10 games before texting (avoid sending zeros)
        if [ "$GAMES" -lt 10 ] && [ "${1:-}" != "--force" ]; then
            echo "Only $GAMES games — waiting for 10+ before texting (use --force to override)"
            exit 0
        fi

        FLOOR=$(echo "$STATUS" | jq -r '.avg_floor_100 // .avg_floor // "?"')
        PEAK=$(echo "$STATUS" | jq -r '.peak_floor // 0')
        WINS=$(echo "$STATUS" | jq -r '.total_wins // 0')
        SWEEP=$(echo "$STATUS" | jq -r '.current_sweep // "?"')
        TOTAL_SW=$(echo "$STATUS" | jq -r '.total_sweeps // "?"')
        ELAPSED=$(echo "$STATUS" | jq -r '.elapsed_hours // "?"')
        ENT=$(echo "$STATUS" | jq -r '.entropy // "?"')
        VL=$(echo "$STATUS" | jq -r '.value_loss // "?"')
        PL=$(echo "$STATUS" | jq -r '.policy_loss // "?"')
        GPU=$(echo "$STATUS" | jq -r '.gpu_percent // "?"')
        CFG=$(echo "$STATUS" | jq -r '.config_name // "?"')
        GPM=$(echo "$STATUS" | jq -r '.games_per_min // "?"')
        COLLECT=$(echo "$STATUS" | jq -r '.collect_progress // "?"')
        DISK=$(df -g . | tail -1 | awk '{print $4}')
        PID_ALIVE=$([ -f .run/training.pid ] && kill -0 "$(cat .run/training.pid)" 2>/dev/null && echo "alive" || echo "DEAD")

        # Death analysis
        TOP_KILLER=$(tail -50 "$EPISODES" 2>/dev/null | jq -rs '
          group_by(.death_enemy) | map({e: .[0].death_enemy, n: length})
          | sort_by(-.n) | .[0] | "\(.e) x\(.n)"' 2>/dev/null || echo "?")
        BOSS_ATTEMPTS=$(tail -100 "$EPISODES" 2>/dev/null | jq -s '[.[] | select(.floor >= 16)] | length' 2>/dev/null || echo "0")
        BOSS_KILLS=$(tail -100 "$EPISODES" 2>/dev/null | jq -s '[.[] | select(.floor >= 17)] | length' 2>/dev/null || echo "0")
        EARLY_D=$(tail -100 "$EPISODES" 2>/dev/null | jq -s '[.[] | select(.floor < 6)] | length' 2>/dev/null || echo "0")

        # Threshold annotations
        ent_note=""; [ "$ENT" != "?" ] && ent_note=$(echo "$ENT" | awk '{if ($1 < 0.02) print "(COLLAPSED <0.02)"; else if ($1 < 0.5) print "(low <0.5)"; else print "(ok >0.5)"}')
        vl_note=""; [ "$VL" != "?" ] && vl_note=$(echo "$VL" | awk '{if ($1 > 5.0) print "(high, want <2.0)"; else if ($1 > 2.0) print "(elevated)"; else print "(ok)"}')
        disk_note=""; [ "$DISK" -lt 5 ] 2>/dev/null && disk_note="LOW" || disk_note="ok"

        MSG="=== StS Training (${ELAPSED}h) ===

PROGRESS
  Games: ${GAMES} (${COLLECT}) sweep ${SWEEP}/${TOTAL_SW}
  Floor: ${FLOOR} avg | ${PEAK} peak
  Wins: ${WINS}

BOSS WALL
  ${TOP_KILLER}
  Boss: ${BOSS_KILLS}/${BOSS_ATTEMPTS} kills | Early deaths: ${EARLY_D}

HEALTH
  Entropy: ${ENT} ${ent_note}
  Value loss: ${VL} ${vl_note}
  Policy: ${PL}
  Throughput: ${GPM} g/min

SYSTEM
  GPU ${GPU}% | ${DISK}GB free (${disk_note}) | ${PID_ALIVE}
  Config: ${CFG}"

        _send_text() {
            osascript -e "tell application \"Messages\"
              set s to 1st account whose service type = iMessage
              send \"$1\" to participant \"$PHONE\" of s
            end tell" 2>/dev/null
        }

        if [ "${1:-}" = "--loop" ]; then
            INTERVAL="${2:-2h}"
            SECS=$(echo "$INTERVAL" | sed 's/h/*3600/;s/m/*60/' | bc)
            echo "Texting every $INTERVAL ($SECS seconds). Ctrl+C to stop."
            caffeinate -dims -w $$ &
            _send_text "$MSG" && echo "[$(date)] Sent" || echo "[$(date)] Failed"
            sleep "$SECS"
            exec "$0" text --loop "$INTERVAL"
        else
            _send_text "$MSG" && echo "Status text sent to $PHONE" || echo "Failed to send"
        fi
        ;;
    *)
        echo "Usage: $0 {start|stop|status|resume|weekend|pretrain|experiment|push-metrics|...}"
        echo ""
        echo "Training:"
        echo "  start      Start training (default 10K games)"
        echo "  stop       Graceful shutdown"
        echo "  status     Show training metrics"
        echo "  resume     Resume from checkpoint"
        echo "  weekend    Long-running headless mode"
        echo "  restart    Quick restart with code changes"
        echo ""
        echo "Pretrain:"
        echo "  pretrain           Run full pipeline (BC → CombatNet → Eval)"
        echo "  pretrain --bc      Behavioral cloning only"
        echo "  pretrain --combat  CombatNet training only"
        echo "  pretrain --eval    Evaluate checkpoint only"
        echo ""
        echo "Experiments:"
        echo "  experiment <name>      Run named experiment from sweep_config"
        echo "  experiment reward-sim  Offline reward rescoring simulation"
        echo "  experiment reward-ab   Live A/B test with reward configs"
        echo "  push-metrics       Push metrics to GitHub Gist"
        echo ""
        echo "Data:"
        echo "  data inventory  Inventory all training data"
        echo "  data quality    Run quality checks"
        echo "  data organize   Organize into tiered dirs"
        echo ""
        echo "Management:"
        echo "  archive    Archive current run"
        echo "  fresh      Archive + cold start"
        echo "  update     Pull latest + restart"
        echo "  hotfix     Live parameter tuning"
        echo "  prune      Prune old data"
        exit 1
        ;;
esac
