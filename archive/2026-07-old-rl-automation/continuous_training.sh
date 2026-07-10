#!/usr/bin/env bash
# Continuous training loop: chains `run-phase1-puct-overnight` iterations,
# each iteration resumes from the previous iteration's checkpoint.
#
# Usage (detached via scripts/training.sh launch):
#   ./scripts/training.sh launch \
#       --log-file logs/active/cont-train-$(date +%Y%m%d-%H%M%S)/launcher.log \
#       --pid-file logs/active/cont-train-$(date +%Y%m%d-%H%M%S)/launcher.pid \
#       -- bash scripts/continuous_training.sh \
#           --root logs/active/cont-train-$(date +%Y%m%d-%H%M%S) \
#           --max-hours 72 --target-cases 50000 --passes 3 --epochs 4
#
# Behavior:
#   - Iteration N writes to <root>/iter-NN/ with its own checkpoint.json.
#   - Iteration N+1 passes --checkpoint <prev>/checkpoint.json to the CLI so
#     MLX picks up the trained weights instead of starting from scratch.
#   - Each iteration fires an iMessage alert via scripts/alert.sh on completion.
#   - Loop stops when wall-clock exceeds --max-hours OR when the CLI errors
#     three times in a row (back-off, not infinite retry).

set -eu

ROOT=""
MAX_HOURS=72
TARGET_CASES=50000
COLLECTION_PASSES=3
EPOCHS=4
BASE_SEED=42

while [[ $# -gt 0 ]]; do
    case "$1" in
        --root) ROOT="$2"; shift 2 ;;
        --max-hours) MAX_HOURS="$2"; shift 2 ;;
        --target-cases) TARGET_CASES="$2"; shift 2 ;;
        --passes) COLLECTION_PASSES="$2"; shift 2 ;;
        --epochs) EPOCHS="$2"; shift 2 ;;
        --base-seed) BASE_SEED="$2"; shift 2 ;;
        *) echo "unknown flag: $1"; exit 2 ;;
    esac
done

if [[ -z "$ROOT" ]]; then
    echo "required: --root <output-dir>"
    exit 2
fi

mkdir -p "$ROOT"
START_EPOCH=$(date +%s)
MAX_SECONDS=$((MAX_HOURS * 3600))
ITER=0
CHECKPOINT=""
CONSECUTIVE_FAILS=0

bash scripts/alert.sh info "continuous-training loop started at $ROOT (max ${MAX_HOURS}h)"

while true; do
    NOW=$(date +%s)
    ELAPSED=$((NOW - START_EPOCH))
    if [[ $ELAPSED -ge $MAX_SECONDS ]]; then
        bash scripts/alert.sh info "continuous-training reached max ${MAX_HOURS}h after ${ITER} iterations; stopping cleanly"
        break
    fi

    ITER=$((ITER + 1))
    ITER_DIR=$(printf "%s/iter-%02d" "$ROOT" "$ITER")
    mkdir -p "$ITER_DIR"
    ITER_SEED=$((BASE_SEED + ITER))

    CMD=(uv run python -m packages.training run-phase1-puct-overnight
         --output-dir "$ITER_DIR"
         --target-cases "$TARGET_CASES"
         --collection-passes "$COLLECTION_PASSES"
         --epochs "$EPOCHS"
         --seed "$ITER_SEED")
    if [[ -n "$CHECKPOINT" && -f "$CHECKPOINT" ]]; then
        CMD+=(--checkpoint "$CHECKPOINT")
    fi

    echo "[$(date -Iseconds)] iter=${ITER} seed=${ITER_SEED} checkpoint=${CHECKPOINT:-none}" >> "$ROOT/loop.log"

    if "${CMD[@]}" >> "$ITER_DIR/stdout.log" 2>> "$ITER_DIR/stderr.log"; then
        CONSECUTIVE_FAILS=0
        # Pass the completed checkpoint into the next iteration.
        if [[ -f "$ITER_DIR/checkpoint.json" ]]; then
            CHECKPOINT="$ITER_DIR/checkpoint.json"
        fi
        bash scripts/alert.sh info "continuous-training iter ${ITER} done; elapsed ${ELAPSED}s; checkpoint=${CHECKPOINT}"
    else
        CONSECUTIVE_FAILS=$((CONSECUTIVE_FAILS + 1))
        bash scripts/alert.sh warn "continuous-training iter ${ITER} FAILED (consecutive=${CONSECUTIVE_FAILS}); see ${ITER_DIR}/stderr.log"
        if [[ $CONSECUTIVE_FAILS -ge 3 ]]; then
            bash scripts/alert.sh critical "continuous-training stopping after ${CONSECUTIVE_FAILS} consecutive failures; needs manual debug"
            exit 1
        fi
        # Backoff before retry.
        sleep 60
    fi
done

bash scripts/alert.sh info "continuous-training finished cleanly after ${ITER} iterations; final checkpoint=${CHECKPOINT}"
