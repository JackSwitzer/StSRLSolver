#!/usr/bin/env bash
# record_play.sh — launch the game with TraceLab record-mode
# (SPEC-tracelab-record-mode.md). Human-attended only: play normally; every
# decision + post-action state is recorded to data/traces/recordings/.
#
# Usage: scripts/record_play.sh
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
MOD_DIR="${REPO_ROOT}/packages/harness-java"
STS_DIR="/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
RECORD_DIR="${REPO_ROOT}/data/traces/recordings"

mkdir -p "${RECORD_DIR}"

# Build if stale (same pattern as trace_java.sh).
jar="${MOD_DIR}/target/TraceLab.jar"
if [[ ! -f "${jar}" ]] || find "${MOD_DIR}/src" -name '*.java' -newer "${jar}" | grep -q .; then
  echo "[record_play] building TraceLab..."
  (cd "${MOD_DIR}" && mvn -q package)
fi
cp "${jar}" "${STS_DIR}/mods/TraceLab.jar"

echo "[record_play] recordings -> ${RECORD_DIR}"
echo "[record_play] launching game (record-mode on; no script). Play normally."
cd "${STS_DIR}"
exec java -Dtracelab.recorddir="${RECORD_DIR}" \
  -jar ModTheSpire.jar --mods basemod,tracelab --skip-launcher
