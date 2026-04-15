#!/usr/bin/env zsh
set -euo pipefail

SCRIPT_DIR="${0:A:h}"
REPO_ROOT="${SCRIPT_DIR:h}"
cd "$REPO_ROOT"

if [[ "${1:-}" == "launch" ]]; then
  shift
  log_file="logs/active/training-launcher.log"
  pid_file="logs/active/training-launcher.pid"

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
      --)
        shift
        break
        ;;
      *)
        break
        ;;
    esac
  done

  mkdir -p "$(dirname "$log_file")" "$(dirname "$pid_file")"
  nohup caffeinate -dimsu uv run python -m packages.training "$@" >>"$log_file" 2>&1 &
  pid=$!
  printf '%s\n' "$pid" > "$pid_file"
  printf 'launched training pid=%s\nlog=%s\npid_file=%s\n' "$pid" "$log_file" "$pid_file"
  exit 0
fi

exec uv run python -m packages.training "$@"
