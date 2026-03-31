#!/bin/bash
# Send an alert via iMessage and log it.
#
# Usage: bash scripts/alert.sh [info|warn|critical] "message"
#
# Sends "[STS {severity}] {message}" via iMessage to the configured phone number.
# Logs all alerts to logs/active/alerts.log (falls back to logs/alerts.log).

set -e
cd "$(dirname "$0")/.."

PHONE="+14166293183"
SEVERITY="${1:-info}"
MESSAGE="${2:-no message}"

# Validate severity
case "$SEVERITY" in
    info|warn|critical) ;;
    *) echo "Unknown severity: $SEVERITY (use info|warn|critical)"; exit 1 ;;
esac

FORMATTED="[STS ${SEVERITY}] ${MESSAGE}"
TIMESTAMP=$(date "+%Y-%m-%d %H:%M:%S")

# Determine log file
LOG_DIR="logs/active"
[ ! -d "$LOG_DIR" ] && LOG_DIR="logs"
mkdir -p "$LOG_DIR"
LOG_FILE="$LOG_DIR/alerts.log"

# Log to file
echo "${TIMESTAMP} | ${SEVERITY} | ${MESSAGE}" >> "$LOG_FILE"

# Send via iMessage
osascript -e "tell application \"Messages\"
  set s to 1st account whose service type = iMessage
  send \"${FORMATTED}\" to participant \"${PHONE}\" of s
end tell" 2>/dev/null && echo "Alert sent: $FORMATTED" || echo "Failed to send alert (iMessage unavailable)"
