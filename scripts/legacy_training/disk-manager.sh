#!/bin/bash
# Disk retention policy manager for the Slay the Spire RL project.
#
# Reads retention settings from packages/training/training_config.py.
# Manages runs, checkpoints, worktrees, and caches.
#
# Usage:
#   ./scripts/disk-manager.sh status              Show disk usage breakdown
#   ./scripts/disk-manager.sh clean               Apply retention policies
#   ./scripts/disk-manager.sh clean --dry-run      Show what would be cleaned
#   ./scripts/disk-manager.sh archive              Compress old runs for export
#   ./scripts/disk-manager.sh policy               Show current retention settings

set -eo pipefail
cd "$(dirname "$0")/.."

PROJECT_ROOT="$(pwd)"
RUNS_DIR="logs/runs"
ARCHIVE_DIR="logs/archive"
PID_FILE=".run/training.pid"
CONFIG_FILE="packages/training/training_config.py"
DRY_RUN=false

# -- Config loading ------------------------------------------------

load_retention() {
    # Extract RETENTION dict values from training_config.py using Python
    eval "$(uv run python -c "
from packages.training.training_config import RETENTION
for k, v in RETENTION.items():
    print(f'RETENTION_{k.upper()}={v}')
" 2>/dev/null)" || {
        # Fallback defaults if Python import fails
        RETENTION_RUNS_KEEP_TOP_N=10
        RETENTION_RUNS_KEEP_LATEST_N=10
        RETENTION_CHECKPOINTS_KEEP_LATEST_N=10
        RETENTION_CHECKPOINTS_KEEP_BEST_N=3
        RETENTION_ARCHIVE_AFTER_DAYS=7
        RETENTION_DELETE_AFTER_DAYS=30
        RETENTION_DISK_WARN_GB=10
        RETENTION_DISK_PAUSE_GB=5
        RETENTION_DISK_EMERGENCY_GB=3
    }
}

# -- Helpers -------------------------------------------------------

get_free_disk_gb() {
    if [[ "$(uname)" == "Darwin" ]]; then
        df -g . | tail -1 | awk '{print $4}'
    else
        df --output=avail -BG . | tail -1 | tr -d 'G '
    fi
}

get_total_disk_gb() {
    if [[ "$(uname)" == "Darwin" ]]; then
        df -g . | tail -1 | awk '{print $2}'
    else
        df --output=size -BG . | tail -1 | tr -d 'G '
    fi
}

training_alive() {
    [ -f "$PID_FILE" ] && kill -0 "$(cat "$PID_FILE")" 2>/dev/null
}

dir_size_mb() {
    du -sm "$1" 2>/dev/null | awk '{print $1}'
}

dir_size_human() {
    du -sh "$1" 2>/dev/null | awk '{print $1}'
}

# Get avg_floor from a run's episodes.jsonl (returns 0 if unavailable)
run_avg_floor() {
    local run_dir="$1"
    local episodes="$run_dir/episodes.jsonl"
    if [ -f "$episodes" ]; then
        uv run python -c "
import json, sys
floors = []
for line in open('$episodes'):
    try:
        d = json.loads(line)
        f = d.get('final_floor', d.get('floor', 0))
        floors.append(f)
    except: pass
print(f'{sum(floors)/len(floors):.1f}' if floors else '0.0')
" 2>/dev/null || echo "0.0"
    else
        echo "0.0"
    fi
}

# Get game count from episodes.jsonl
run_game_count() {
    local episodes="$1/episodes.jsonl"
    if [ -f "$episodes" ]; then
        wc -l < "$episodes" | tr -d ' '
    else
        echo "0"
    fi
}

# List all run directories sorted by date (newest first)
list_runs() {
    ls -dt "$RUNS_DIR"/run_* 2>/dev/null || true
}

# Count .pt checkpoint files in a run
count_checkpoints() {
    find "$1" -maxdepth 1 -name "*.pt" 2>/dev/null | wc -l | tr -d ' '
}

# Size of all .pt files in a run (MB)
checkpoints_size_mb() {
    find "$1" -maxdepth 1 -name "*.pt" -exec du -sm {} + 2>/dev/null | awk '{s+=$1} END {print s+0}'
}

# -- status --------------------------------------------------------

cmd_status() {
    load_retention

    local free_gb total_gb
    free_gb=$(get_free_disk_gb)
    total_gb=$(get_total_disk_gb)
    local used_pct=$(( (total_gb - free_gb) * 100 / total_gb ))
    local free_pct=$(( 100 - used_pct ))

    echo "=== Disk Manager ==="
    echo "Free: ${free_gb}GB / ${total_gb}GB (${free_pct}%)"

    # Training status
    if training_alive; then
        local pid
        pid=$(cat "$PID_FILE")
        echo "Training: RUNNING (PID $pid)"
    else
        echo "Training: STOPPED"
    fi
    echo ""

    # Runs breakdown
    local all_runs=()
    while IFS= read -r d; do
        [ -n "$d" ] && all_runs+=("$d")
    done < <(list_runs)
    local total_runs=${#all_runs[@]}

    local active_count=0
    local archivable_count=0
    if [ "$total_runs" -gt 0 ]; then
        # Active = symlinked or running
        local active_link=""
        [ -L "logs/active" ] && active_link=$(cd "$(dirname "logs/active")" && readlink "active" 2>/dev/null) || true

        for run in "${all_runs[@]}"; do
            local run_base
            run_base=$(basename "$run")
            if [[ "$active_link" == *"$run_base"* ]]; then
                active_count=$((active_count + 1))
            fi
        done
        archivable_count=$((total_runs - active_count))
        # Cap at what would actually be archived
        local keep_count=$((RETENTION_RUNS_KEEP_TOP_N + RETENTION_RUNS_KEEP_LATEST_N))
        if [ "$archivable_count" -gt "$((total_runs - keep_count))" ] && [ "$total_runs" -gt "$keep_count" ]; then
            archivable_count=$((total_runs - keep_count))
        fi
        [ "$archivable_count" -lt 0 ] && archivable_count=0
    fi
    echo "Runs: $total_runs total, $active_count active, $archivable_count archivable"

    # Checkpoints breakdown
    local total_ckpt=0
    local prunable_ckpt=0
    local prunable_ckpt_mb=0
    local keep_per_run=$((RETENTION_CHECKPOINTS_KEEP_LATEST_N + RETENTION_CHECKPOINTS_KEEP_BEST_N))
    for run in "${all_runs[@]}"; do
        local n
        n=$(count_checkpoints "$run")
        total_ckpt=$((total_ckpt + n))
        if [ "$n" -gt "$keep_per_run" ]; then
            local excess=$((n - keep_per_run))
            prunable_ckpt=$((prunable_ckpt + excess))
            # Estimate prunable size proportionally
            local run_ckpt_mb
            run_ckpt_mb=$(checkpoints_size_mb "$run")
            local prunable_mb=$(( run_ckpt_mb * excess / n ))
            prunable_ckpt_mb=$((prunable_ckpt_mb + prunable_mb))
        fi
    done
    local kept_ckpt=$((total_ckpt - prunable_ckpt))
    if [ "$prunable_ckpt_mb" -ge 1024 ]; then
        local prunable_gb=$(echo "scale=1; $prunable_ckpt_mb / 1024" | bc)
        echo "Checkpoints: $total_ckpt total, $kept_ckpt kept, $prunable_ckpt prunable (${prunable_gb}GB)"
    else
        echo "Checkpoints: $total_ckpt total, $kept_ckpt kept, $prunable_ckpt prunable (${prunable_ckpt_mb}MB)"
    fi

    # Worktrees
    local worktree_total=0
    local worktree_stale=0
    if [ -d ".claude/worktrees" ]; then
        local merged_branches
        merged_branches=$(git branch --merged main 2>/dev/null | sed 's/^[* ]*//' | grep -v '^main$' || true)
        while IFS= read -r wt; do
            [ -z "$wt" ] && continue
            worktree_total=$((worktree_total + 1))
            local wt_branch
            wt_branch=$(git -C "$wt" rev-parse --abbrev-ref HEAD 2>/dev/null || true)
            if echo "$merged_branches" | grep -qx "$wt_branch" 2>/dev/null; then
                worktree_stale=$((worktree_stale + 1))
            fi
        done < <(find .claude/worktrees -mindepth 1 -maxdepth 1 -type d 2>/dev/null)
    fi
    local worktree_active=$((worktree_total - worktree_stale))
    echo "Worktrees: $worktree_active active, $worktree_stale stale"

    # Caches
    local cache_mb=0
    local pycache_mb
    pycache_mb=$(find . -type d -name "__pycache__" -exec du -sm {} + 2>/dev/null | awk '{s+=$1} END {print s+0}')
    cache_mb=$((cache_mb + pycache_mb))
    local pytest_mb
    pytest_mb=$(du -sm .pytest_cache 2>/dev/null | awk '{print $1}' || echo "0")
    cache_mb=$((cache_mb + pytest_mb))
    echo "Caches: ${cache_mb}MB clearable"

    echo ""

    # Retention compliance
    if [ "$free_gb" -lt "$RETENTION_DISK_EMERGENCY_GB" ]; then
        echo "Retention compliance: EMERGENCY (${free_gb}GB < ${RETENTION_DISK_EMERGENCY_GB}GB)"
        echo "  Run: disk-manager.sh clean --emergency"
    elif [ "$free_gb" -lt "$RETENTION_DISK_PAUSE_GB" ]; then
        echo "Retention compliance: CRITICAL (${free_gb}GB < ${RETENTION_DISK_PAUSE_GB}GB)"
        echo "  Training will be paused. Run: disk-manager.sh clean"
    elif [ "$free_gb" -lt "$RETENTION_DISK_WARN_GB" ]; then
        echo "Retention compliance: WARNING (${free_gb}GB < ${RETENTION_DISK_WARN_GB}GB)"
        echo "  Run: disk-manager.sh clean"
    else
        echo "Retention compliance: OK"
    fi
}

# -- clean ---------------------------------------------------------

cmd_clean() {
    load_retention

    local free_gb
    free_gb=$(get_free_disk_gb)
    local emergency=false

    echo "=== Disk Manager: Clean ==="
    $DRY_RUN && echo "[DRY RUN - no changes will be made]"
    echo ""

    # 1. Prune checkpoints per run
    echo "--- Checkpoints ---"
    local ckpt_pruned=0
    local ckpt_freed_mb=0
    while IFS= read -r run; do
        [ -z "$run" ] && continue
        local pts=()
        while IFS= read -r pt; do
            [ -n "$pt" ] && pts+=("$pt")
        done < <(find "$run" -maxdepth 1 -name "*.pt" -not -name "shutdown_checkpoint.pt" 2>/dev/null | sort)

        local n=${#pts[@]}
        local keep_n=$((RETENTION_CHECKPOINTS_KEEP_LATEST_N + RETENTION_CHECKPOINTS_KEEP_BEST_N))
        if [ "$n" -le "$keep_n" ]; then
            continue
        fi

        # Keep latest N (sorted by name = by date for checkpoint_NNNNN.pt)
        local keep_set=()
        local sorted_by_date=()
        while IFS= read -r pt; do
            sorted_by_date+=("$pt")
        done < <(ls -t "${pts[@]}" 2>/dev/null)

        for i in $(seq 0 $((RETENTION_CHECKPOINTS_KEEP_LATEST_N - 1))); do
            [ "$i" -lt "${#sorted_by_date[@]}" ] && keep_set+=("${sorted_by_date[$i]}")
        done

        # Always keep shutdown_checkpoint.pt
        local shutdown_ckpt="$run/shutdown_checkpoint.pt"
        [ -f "$shutdown_ckpt" ] && keep_set+=("$shutdown_ckpt")

        # For "best by floor" -- we'd need metadata. Fall back to keeping largest files
        # (larger checkpoints often correspond to more training = better performance)
        local sorted_by_size=()
        while IFS= read -r pt; do
            sorted_by_size+=("$pt")
        done < <(ls -S "${pts[@]}" 2>/dev/null)

        for i in $(seq 0 $((RETENTION_CHECKPOINTS_KEEP_BEST_N - 1))); do
            [ "$i" -lt "${#sorted_by_size[@]}" ] && keep_set+=("${sorted_by_size[$i]}")
        done

        # Deduplicate keep_set
        local keep_unique
        keep_unique=$(printf '%s\n' "${keep_set[@]}" | sort -u)

        for pt in "${pts[@]}"; do
            if ! echo "$keep_unique" | grep -qxF "$pt"; then
                local pt_mb
                pt_mb=$(du -sm "$pt" 2>/dev/null | awk '{print $1}')
                if $DRY_RUN; then
                    echo "  [would delete] $pt (${pt_mb}MB)"
                else
                    rm -f "$pt"
                    echo "  Deleted: $pt (${pt_mb}MB)"
                fi
                ckpt_pruned=$((ckpt_pruned + 1))
                ckpt_freed_mb=$((ckpt_freed_mb + pt_mb))
            fi
        done
    done < <(list_runs)
    echo "Checkpoints: pruned $ckpt_pruned, freed ${ckpt_freed_mb}MB"
    echo ""

    # 2. Archive old runs
    echo "--- Runs ---"
    local runs_archived=0
    local all_runs=()
    while IFS= read -r d; do
        [ -n "$d" ] && all_runs+=("$d")
    done < <(list_runs)

    if [ "${#all_runs[@]}" -gt 0 ]; then
        # Build keep set: latest N + top N by avg_floor
        local keep_latest=()
        for i in $(seq 0 $((RETENTION_RUNS_KEEP_LATEST_N - 1))); do
            [ "$i" -lt "${#all_runs[@]}" ] && keep_latest+=("${all_runs[$i]}")
        done

        # Rank by avg_floor
        local scored_runs=""
        for run in "${all_runs[@]}"; do
            local avg
            avg=$(run_avg_floor "$run")
            scored_runs+="$avg $run"$'\n'
        done
        local top_runs=()
        while IFS= read -r line; do
            [ -z "$line" ] && continue
            local run_path
            run_path=$(echo "$line" | awk '{print $2}')
            [ -n "$run_path" ] && top_runs+=("$run_path")
        done < <(echo "$scored_runs" | sort -t' ' -k1 -rn | head -n "$RETENTION_RUNS_KEEP_TOP_N")

        # Combine keep sets
        local keep_runs
        keep_runs=$(printf '%s\n' "${keep_latest[@]}" "${top_runs[@]}" | sort -u)

        # Also keep the active symlink target
        if [ -L "logs/active" ]; then
            local active_target
            active_target=$(cd logs && readlink active 2>/dev/null || true)
            [ -n "$active_target" ] && keep_runs+=$'\n'"$RUNS_DIR/${active_target#runs/}"
        fi

        local archive_cutoff
        archive_cutoff=$(date -v-"${RETENTION_ARCHIVE_AFTER_DAYS}"d +%Y%m%d 2>/dev/null || \
                         date -d "${RETENTION_ARCHIVE_AFTER_DAYS} days ago" +%Y%m%d 2>/dev/null || \
                         echo "00000000")

        for run in "${all_runs[@]}"; do
            if echo "$keep_runs" | grep -qxF "$run"; then
                continue
            fi
            # Check age from directory name (run_YYYYMMDD_HHMMSS)
            local run_date
            run_date=$(basename "$run" | sed 's/run_\([0-9]\{8\}\).*/\1/')
            if [[ "$run_date" =~ ^[0-9]{8}$ ]] && [ "$run_date" -lt "$archive_cutoff" ]; then
                if $DRY_RUN; then
                    echo "  [would archive] $run"
                else
                    mkdir -p "$ARCHIVE_DIR"
                    mv "$run" "$ARCHIVE_DIR/"
                    echo "  Archived: $run -> $ARCHIVE_DIR/"
                fi
                runs_archived=$((runs_archived + 1))
            fi
        done
    fi
    echo "Runs: archived $runs_archived"
    echo ""

    # 3. Remove stale worktrees (branches merged to main)
    echo "--- Worktrees ---"
    local wt_removed=0
    if [ -d ".claude/worktrees" ]; then
        local merged_branches
        merged_branches=$(git branch --merged main 2>/dev/null | sed 's/^[* ]*//' | grep -v '^main$' || true)
        while IFS= read -r wt; do
            [ -z "$wt" ] && continue
            local wt_branch
            wt_branch=$(git -C "$wt" rev-parse --abbrev-ref HEAD 2>/dev/null || true)
            if echo "$merged_branches" | grep -qx "$wt_branch" 2>/dev/null; then
                if $DRY_RUN; then
                    echo "  [would remove] $wt (branch: $wt_branch, merged)"
                else
                    git worktree remove "$wt" --force 2>/dev/null || rm -rf "$wt"
                    echo "  Removed: $wt (branch: $wt_branch)"
                fi
                wt_removed=$((wt_removed + 1))
            fi
        done < <(find .claude/worktrees -mindepth 1 -maxdepth 1 -type d 2>/dev/null)
    fi
    echo "Worktrees: removed $wt_removed stale"
    echo ""

    # 4. Clear caches
    echo "--- Caches ---"
    local cache_freed_mb=0

    # __pycache__
    local pycache_mb
    pycache_mb=$(find . -type d -name "__pycache__" -exec du -sm {} + 2>/dev/null | awk '{s+=$1} END {print s+0}')
    if [ "$pycache_mb" -gt 0 ]; then
        if $DRY_RUN; then
            echo "  [would clear] __pycache__ (${pycache_mb}MB)"
        else
            find . -type d -name "__pycache__" -exec rm -rf {} + 2>/dev/null || true
            echo "  Cleared: __pycache__ (${pycache_mb}MB)"
        fi
        cache_freed_mb=$((cache_freed_mb + pycache_mb))
    fi

    # .pytest_cache
    if [ -d ".pytest_cache" ]; then
        local pytest_mb
        pytest_mb=$(du -sm .pytest_cache 2>/dev/null | awk '{print $1}' || echo "0")
        if $DRY_RUN; then
            echo "  [would clear] .pytest_cache (${pytest_mb}MB)"
        else
            rm -rf .pytest_cache
            echo "  Cleared: .pytest_cache (${pytest_mb}MB)"
        fi
        cache_freed_mb=$((cache_freed_mb + pytest_mb))
    fi

    echo "Caches: freed ${cache_freed_mb}MB"
    echo ""

    # 5. Low disk escalation
    free_gb=$(get_free_disk_gb)
    if [ "$free_gb" -lt "$RETENTION_DISK_WARN_GB" ]; then
        echo "--- Low Disk Escalation (${free_gb}GB free) ---"

        # Clear UV cache
        local uv_cache_dir
        uv_cache_dir=$(uv cache dir 2>/dev/null || echo "")
        if [ -n "$uv_cache_dir" ] && [ -d "$uv_cache_dir" ]; then
            local uv_mb
            uv_mb=$(du -sm "$uv_cache_dir" 2>/dev/null | awk '{print $1}' || echo "0")
            if $DRY_RUN; then
                echo "  [would clear] UV cache (${uv_mb}MB)"
            else
                uv cache clean 2>/dev/null || true
                echo "  Cleared: UV cache (${uv_mb}MB)"
            fi
        fi

        # Brew cleanup
        if command -v brew &>/dev/null; then
            if $DRY_RUN; then
                echo "  [would run] brew cleanup"
            else
                brew cleanup --prune=0 2>/dev/null || true
                echo "  Ran: brew cleanup"
            fi
        fi
    fi

    # 6. Critical: pause training
    if [ "$free_gb" -lt "$RETENTION_DISK_PAUSE_GB" ]; then
        echo ""
        echo "*** CRITICAL: ${free_gb}GB free < ${RETENTION_DISK_PAUSE_GB}GB pause threshold ***"
        if training_alive; then
            local pid
            pid=$(cat "$PID_FILE")
            if $DRY_RUN; then
                echo "  [would pause] Training PID $pid"
            else
                echo "  Pausing training (PID $pid)..."
                kill -TERM "$pid" 2>/dev/null || true
                echo "  Training paused. Restart with: ./scripts/training.sh resume"
            fi
        fi

        # Emergency prune: delete archived runs older than delete_after_days
        if [ "$free_gb" -lt "$RETENTION_DISK_EMERGENCY_GB" ] && [ -d "$ARCHIVE_DIR" ]; then
            echo ""
            echo "*** EMERGENCY: ${free_gb}GB free < ${RETENTION_DISK_EMERGENCY_GB}GB ***"
            local delete_cutoff
            delete_cutoff=$(date -v-"${RETENTION_DELETE_AFTER_DAYS}"d +%Y%m%d 2>/dev/null || \
                            date -d "${RETENTION_DELETE_AFTER_DAYS} days ago" +%Y%m%d 2>/dev/null || \
                            echo "00000000")
            while IFS= read -r archived_run; do
                [ -z "$archived_run" ] && continue
                local ar_date
                ar_date=$(basename "$archived_run" | sed 's/run_\([0-9]\{8\}\).*/\1/')
                if [[ "$ar_date" =~ ^[0-9]{8}$ ]] && [ "$ar_date" -lt "$delete_cutoff" ]; then
                    if $DRY_RUN; then
                        echo "  [would delete] $archived_run"
                    else
                        rm -rf "$archived_run"
                        echo "  Deleted archived: $archived_run"
                    fi
                fi
            done < <(find "$ARCHIVE_DIR" -mindepth 1 -maxdepth 1 -type d -name "run_*" 2>/dev/null)
        fi
    fi

    echo ""
    free_gb=$(get_free_disk_gb)
    echo "Done. Free disk: ${free_gb}GB"
}

# -- archive -------------------------------------------------------

cmd_archive() {
    load_retention

    echo "=== Disk Manager: Archive ==="
    $DRY_RUN && echo "[DRY RUN - no changes will be made]"
    echo ""

    local archive_cutoff
    archive_cutoff=$(date -v-"${RETENTION_ARCHIVE_AFTER_DAYS}"d +%Y%m%d 2>/dev/null || \
                     date -d "${RETENTION_ARCHIVE_AFTER_DAYS} days ago" +%Y%m%d 2>/dev/null || \
                     echo "00000000")

    local export_dir="logs/export"
    mkdir -p "$export_dir"

    local archived=0
    while IFS= read -r run; do
        [ -z "$run" ] && continue
        local run_date
        run_date=$(basename "$run" | sed 's/run_\([0-9]\{8\}\).*/\1/')
        if ! [[ "$run_date" =~ ^[0-9]{8}$ ]]; then
            continue
        fi
        if [ "$run_date" -ge "$archive_cutoff" ]; then
            continue
        fi

        local run_base
        run_base=$(basename "$run")
        local tar_file="$export_dir/${run_base}.tar.gz"

        if [ -f "$tar_file" ]; then
            echo "  Already archived: $tar_file"
            continue
        fi

        local games
        games=$(run_game_count "$run")
        local avg_floor
        avg_floor=$(run_avg_floor "$run")

        if $DRY_RUN; then
            echo "  [would compress] $run -> $tar_file (${games} games, avg F${avg_floor})"
        else
            echo "  Compressing: $run -> $tar_file (${games} games, avg F${avg_floor})"
            tar -czf "$tar_file" -C "$(dirname "$run")" "$(basename "$run")" 2>/dev/null
            echo "    Size: $(du -sh "$tar_file" | awk '{print $1}')"
        fi
        archived=$((archived + 1))
    done < <(list_runs)

    # Also compress anything already in logs/archive/
    if [ -d "$ARCHIVE_DIR" ]; then
        while IFS= read -r run; do
            [ -z "$run" ] && continue
            local run_base
            run_base=$(basename "$run")
            local tar_file="$export_dir/${run_base}.tar.gz"
            if [ -f "$tar_file" ]; then
                continue
            fi
            if $DRY_RUN; then
                echo "  [would compress] $run -> $tar_file"
            else
                echo "  Compressing archived: $run -> $tar_file"
                tar -czf "$tar_file" -C "$(dirname "$run")" "$(basename "$run")" 2>/dev/null
                echo "    Size: $(du -sh "$tar_file" | awk '{print $1}')"
            fi
            archived=$((archived + 1))
        done < <(find "$ARCHIVE_DIR" -mindepth 1 -maxdepth 1 -type d -name "run_*" 2>/dev/null)
    fi

    echo ""
    echo "Archived $archived runs to $export_dir/"
    echo "Ready for Google Drive export."
}

# -- policy --------------------------------------------------------

cmd_policy() {
    load_retention

    echo "=== Disk Manager: Retention Policy ==="
    echo "Source: $CONFIG_FILE"
    echo ""
    echo "Runs:"
    echo "  Keep top N by avg_floor:     $RETENTION_RUNS_KEEP_TOP_N"
    echo "  Keep latest N by date:       $RETENTION_RUNS_KEEP_LATEST_N"
    echo "  Archive after (days):        $RETENTION_ARCHIVE_AFTER_DAYS"
    echo "  Delete archived after (days): $RETENTION_DELETE_AFTER_DAYS"
    echo ""
    echo "Checkpoints (per run):"
    echo "  Keep latest N:               $RETENTION_CHECKPOINTS_KEEP_LATEST_N"
    echo "  Keep best N:                 $RETENTION_CHECKPOINTS_KEEP_BEST_N"
    echo ""
    echo "Disk thresholds:"
    echo "  Warning  (<${RETENTION_DISK_WARN_GB}GB):     clear caches, UV, brew"
    echo "  Pause    (<${RETENTION_DISK_PAUSE_GB}GB):      stop training, aggressive prune"
    echo "  Emergency (<${RETENTION_DISK_EMERGENCY_GB}GB):    delete old archives"
}

# -- Main ----------------------------------------------------------

case "${1:-status}" in
    status)
        cmd_status
        ;;
    clean)
        shift || true
        [[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true
        cmd_clean
        ;;
    archive)
        shift || true
        [[ "${1:-}" == "--dry-run" ]] && DRY_RUN=true
        cmd_archive
        ;;
    policy)
        cmd_policy
        ;;
    *)
        echo "Usage: $0 {status|clean|archive|policy}"
        echo ""
        echo "  status               Show disk usage breakdown and retention compliance"
        echo "  clean                Apply retention policies (prune checkpoints, archive runs)"
        echo "  clean --dry-run      Show what would be cleaned without doing it"
        echo "  archive              Compress old runs for Google Drive export"
        echo "  archive --dry-run    Show what would be archived"
        echo "  policy               Show current retention settings"
        exit 1
        ;;
esac
