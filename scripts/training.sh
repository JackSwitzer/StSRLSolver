#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPO_ROOT="${SCRIPT_DIR:h}"
cd "$REPO_ROOT"

archive_active() {
  local active_dir="logs/active"
  if [[ ! -d "$active_dir" ]]; then
    return 0
  fi
  if [[ -z "$(ls -A "$active_dir" 2>/dev/null)" ]]; then
    return 0
  fi
  local pid_file="$active_dir/training-launcher.pid"
  if [[ -f "$pid_file" ]]; then
    local prev_pid
    prev_pid="$(<"$pid_file")"
    # Validate pid_file content is numeric before signalling — malformed content
    # (whitespace, stale hex, empty) would either silently fall through or, if
    # the file is writable by another user, could cause `kill -0` to probe an
    # unrelated pid.
    if [[ "$prev_pid" =~ ^[0-9]+$ ]] && kill -0 "$prev_pid" 2>/dev/null; then
      printf 'refusing to archive: pid %s still running (pid_file=%s)\n' "$prev_pid" "$pid_file" >&2
      return 1
    fi
  fi
  local stamp
  stamp="$(date -u +%Y%m%dT%H%M%SZ)"
  local dest="logs/runs/$stamp"
  mkdir -p "logs/runs"
  if [[ -e "$dest" ]]; then
    dest="logs/runs/${stamp}-$$"
  fi
  mv "$active_dir" "$dest"
  mkdir -p "$active_dir"
  printf 'archived %s -> %s\n' "$active_dir" "$dest"
}

case "${1:-}" in
  archive)
    shift
    archive_active
    exit $?
    ;;
  smoke)
    shift
    stamp="$(date -u +%Y%m%dT%H%M%SZ)"
    output_dir="logs/smoke/$stamp"
    mkdir -p "$output_dir"
    printf 'smoke preflight output=%s\n' "$output_dir"
    uv run python -m packages.training run-phase1-puct-overnight \
      --output-dir "$output_dir" \
      --target-cases 24 \
      --collection-passes 1 \
      --epochs 1 \
      "$@"
    rc=$?
    if [[ $rc -eq 0 ]]; then
      printf 'smoke ok: %s\n' "$output_dir"
    else
      printf 'smoke failed rc=%s: %s\n' "$rc" "$output_dir" >&2
    fi
    exit $rc
    ;;
  launch)
    shift
    log_file="logs/active/training-launcher.log"
    pid_file="logs/active/training-launcher.pid"
    run_smoke=0

    while [[ $# -gt 0 ]]; do
      case "$1" in
        --log-file)
          log_file="$2"
          shift 2
          ;;
        --pid-file)
          pid_file="$2"
          shift 2
          ;;
        --with-smoke)
          run_smoke=1
          shift
          ;;
        --)
          shift
          break
          ;;
        *)
          break
          ;;
      esac
    done

    archive_active

    if [[ $run_smoke -eq 1 ]]; then
      printf 'running smoke preflight before launch...\n'
      if ! "$SCRIPT_DIR/training.sh" smoke; then
        printf 'smoke failed; aborting launch\n' >&2
        exit 1
      fi
    fi

    mkdir -p "$(dirname "$log_file")" "$(dirname "$pid_file")"
    nohup caffeinate -dimsu uv run python -m packages.training "$@" </dev/null >>"$log_file" 2>&1 &
    pid=$!
    printf '%s\n' "$pid" > "$pid_file"
    printf 'launched training pid=%s\nlog=%s\npid_file=%s\n' "$pid" "$log_file" "$pid_file"
    exit 0
    ;;
esac

exec uv run python -m packages.training "$@"
