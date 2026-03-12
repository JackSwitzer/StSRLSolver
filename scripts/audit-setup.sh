#!/bin/bash
# Manage the nightly audit launchd job.
#
# Usage:
#   ./scripts/audit-setup.sh install    # Load the launchd plist
#   ./scripts/audit-setup.sh uninstall  # Unload the launchd plist
#   ./scripts/audit-setup.sh status     # Check if loaded + next fire date
#   ./scripts/audit-setup.sh run-now    # Run the audit immediately (for testing)

set -e
cd "$(dirname "$0")/.."

PLIST_SRC="$HOME/Library/LaunchAgents/com.sts-rl.nightly-audit.plist"
LABEL="com.sts-rl.nightly-audit"
AUDIT_SCRIPT="$(pwd)/scripts/nightly-audit.sh"
LOG_DIR="logs/weekend-run/audits"

mkdir -p "$LOG_DIR"

# ── Helpers ─────────────────────────────────────────────

is_loaded() {
    launchctl list "$LABEL" &>/dev/null
}

# ── Commands ────────────────────────────────────────────

cmd_install() {
    if is_loaded; then
        echo "Already loaded. Unloading first..."
        launchctl unload "$PLIST_SRC" 2>/dev/null || true
    fi

    if [ ! -f "$PLIST_SRC" ]; then
        echo "ERROR: Plist not found at $PLIST_SRC"
        exit 1
    fi

    if [ ! -f "$AUDIT_SCRIPT" ]; then
        echo "WARNING: nightly-audit.sh not found at $AUDIT_SCRIPT"
        echo "  Another agent may still be creating it."
    fi

    launchctl load "$PLIST_SRC"
    echo "Loaded $LABEL"
    echo "  Schedule: daily at 20:00"
    echo "  Log: $LOG_DIR/launchd.log"
    echo ""
    cmd_status
}

cmd_uninstall() {
    if ! is_loaded; then
        echo "Not currently loaded."
        return 0
    fi

    launchctl unload "$PLIST_SRC"
    echo "Unloaded $LABEL"
}

cmd_status() {
    echo "=== Nightly Audit Status ==="

    if is_loaded; then
        echo "State: LOADED"
        # Show launchctl info
        launchctl list "$LABEL" 2>/dev/null | while IFS= read -r line; do
            echo "  $line"
        done
    else
        echo "State: NOT LOADED"
        echo "  Run: ./scripts/audit-setup.sh install"
    fi

    echo ""

    # Show last audit results if any
    local latest_audit
    latest_audit=$(ls -t "$LOG_DIR"/audit_*.md 2>/dev/null | head -1)
    if [ -n "$latest_audit" ]; then
        echo "Last audit: $latest_audit"
        echo "  $(head -5 "$latest_audit" | tail -1)"
    else
        echo "No audit reports yet."
    fi

    echo ""

    # Show log tail
    if [ -f "$LOG_DIR/launchd.log" ]; then
        echo "--- Last 5 log lines ---"
        tail -5 "$LOG_DIR/launchd.log" 2>/dev/null
    fi
}

cmd_run_now() {
    if [ ! -f "$AUDIT_SCRIPT" ]; then
        echo "ERROR: nightly-audit.sh not found at $AUDIT_SCRIPT"
        echo "  Create it first or wait for the other agent."
        exit 1
    fi

    if [ ! -x "$AUDIT_SCRIPT" ]; then
        echo "WARNING: nightly-audit.sh is not executable. Fixing..."
        chmod +x "$AUDIT_SCRIPT"
    fi

    echo "Running audit now ($(date))..."
    echo "  Log: $LOG_DIR/launchd.log"
    echo ""

    # Run with same env as launchd would
    "$AUDIT_SCRIPT" 2>&1 | tee -a "$LOG_DIR/launchd.log"
}

# ── Main ────────────────────────────────────────────────

case "${1:-status}" in
    install)   cmd_install ;;
    uninstall) cmd_uninstall ;;
    status)    cmd_status ;;
    run-now)   cmd_run_now ;;
    *)
        echo "Usage: $0 {install|uninstall|status|run-now}"
        echo ""
        echo "Commands:"
        echo "  install    Load the launchd plist (runs daily at 20:00)"
        echo "  uninstall  Unload the launchd plist"
        echo "  status     Check if loaded and show last audit"
        echo "  run-now    Run the audit immediately (for testing)"
        exit 1
        ;;
esac
