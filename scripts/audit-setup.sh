#!/bin/bash
# Manage the nightly audit launchd job.
#
# Usage:
#   ./scripts/audit-setup.sh install    # Generate plist + load the launchd job
#   ./scripts/audit-setup.sh uninstall  # Unload the launchd plist
#   ./scripts/audit-setup.sh status     # Check if loaded + next fire date
#   ./scripts/audit-setup.sh run-now    # Run the audit immediately (for testing)

set -e
cd "$(dirname "$0")/.."

PLIST_PATH="$HOME/Library/LaunchAgents/com.sts-rl.nightly-audit.plist"
LABEL="com.sts-rl.nightly-audit"
PROJECT_DIR="$(pwd)"
AUDIT_SCRIPT="$PROJECT_DIR/scripts/nightly-audit.sh"
LOG_DIR="$PROJECT_DIR/logs/weekend-run/audits"

mkdir -p "$LOG_DIR"

# ── Helpers ─────────────────────────────────────────────

is_loaded() {
    launchctl list "$LABEL" &>/dev/null
}

generate_plist() {
    cat > "$PLIST_PATH" << PLIST_EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>${LABEL}</string>
    <key>ProgramArguments</key>
    <array>
        <string>${AUDIT_SCRIPT}</string>
    </array>
    <key>WorkingDirectory</key>
    <string>${PROJECT_DIR}</string>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>21</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>${LOG_DIR}/launchd.log</string>
    <key>StandardErrorPath</key>
    <string>${LOG_DIR}/launchd.log</string>
    <key>EnvironmentVariables</key>
    <dict>
        <key>PATH</key>
        <string>/usr/local/bin:/usr/bin:/bin:/opt/homebrew/bin:/Users/jackswitzer/.local/bin:/Users/jackswitzer/.cargo/bin</string>
        <key>HOME</key>
        <string>/Users/jackswitzer</string>
    </dict>
    <key>RunAtLoad</key>
    <false/>
    <key>Nice</key>
    <integer>10</integer>
</dict>
</plist>
PLIST_EOF
    echo "Generated plist at $PLIST_PATH"
}

# ── Commands ────────────────────────────────────────────

cmd_install() {
    if is_loaded; then
        echo "Already loaded. Unloading first..."
        launchctl unload "$PLIST_PATH" 2>/dev/null || true
    fi

    # Always regenerate the plist to pick up any path/schedule changes
    generate_plist

    if [ ! -f "$AUDIT_SCRIPT" ]; then
        echo "WARNING: nightly-audit.sh not found at $AUDIT_SCRIPT"
        echo "  Another agent may still be creating it."
    fi

    if [ ! -x "$AUDIT_SCRIPT" ]; then
        echo "Making nightly-audit.sh executable..."
        chmod +x "$AUDIT_SCRIPT"
    fi

    launchctl load "$PLIST_PATH"
    echo "Loaded $LABEL"
    echo "  Schedule: daily at 21:00 (9pm)"
    echo "  Log: $LOG_DIR/launchd.log"
    echo ""
    cmd_status
}

cmd_uninstall() {
    if ! is_loaded; then
        echo "Not currently loaded."
        return 0
    fi

    launchctl unload "$PLIST_PATH"
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

    # Show plist schedule
    if [ -f "$PLIST_PATH" ]; then
        echo "Plist: $PLIST_PATH"
        echo "Schedule: daily at 21:00 (9pm)"
    else
        echo "Plist: NOT FOUND (run install to generate)"
    fi

    echo ""

    # Show last audit results if any
    local latest_audit
    latest_audit=$(ls -t "$LOG_DIR"/*-audit.md 2>/dev/null | head -1 || true)
    if [ -n "$latest_audit" ]; then
        echo "Last audit: $latest_audit"
        echo "  $(head -5 "$latest_audit" | tail -1)"
    else
        echo "No audit reports yet."
    fi

    echo ""

    # Show log tail
    if [ -f "$LOG_DIR/launchd.log" ]; then
        echo "--- Last 10 log lines ---"
        tail -10 "$LOG_DIR/launchd.log" 2>/dev/null
    fi
}

cmd_run_now() {
    if [ ! -f "$AUDIT_SCRIPT" ]; then
        echo "ERROR: nightly-audit.sh not found at $AUDIT_SCRIPT"
        echo "  Create it first or wait for the other agent."
        exit 1
    fi

    if [ ! -x "$AUDIT_SCRIPT" ]; then
        echo "Making nightly-audit.sh executable..."
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
        echo "  install    Generate plist + load the launchd job (runs daily at 21:00)"
        echo "  uninstall  Unload the launchd plist"
        echo "  status     Check if loaded and show last audit"
        echo "  run-now    Run the audit immediately (for testing)"
        exit 1
        ;;
esac
