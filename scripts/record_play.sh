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
STS_DIR="$HOME/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
RECORD_DIR="${REPO_ROOT}/data/traces/recordings"
MVN=/opt/homebrew/bin/mvn
BUILD_JAVA_HOME="${TRACELAB_JAVA_HOME:-/opt/homebrew/opt/openjdk@11}"
LOG="${REPO_ROOT}/logs/record_play.log"

mkdir -p "${RECORD_DIR}" "$(dirname "${LOG}")"

# Patch pre-flight BEFORE build/launch: catches MTS signature violations
# (e.g. illegal special names) without burning a game launch.
"${SCRIPT_DIR}/check_patches.sh"

# Build if stale (same pattern as trace_java.sh).
jar="${MOD_DIR}/target/TraceLab.jar"
if [[ ! -f "${jar}" ]] || find "${MOD_DIR}/src" -name '*.java' -newer "${jar}" | grep -q .; then
  echo "[record_play] building TraceLab..."
  (cd "${MOD_DIR}" && JAVA_HOME="${BUILD_JAVA_HOME}" "${MVN}" -q package)
fi
cp "${jar}" "${STS_DIR}/mods/TraceLab.jar"

echo "[record_play] recordings -> ${RECORD_DIR}"
echo "[record_play] log -> ${LOG}"
echo "[record_play] launching game (record-mode on; no script). Play normally."
cd "${STS_DIR}"
exec ./jre/bin/java -Xmx2G \
  -Djava.awt.headless=false \
  -Dtracelab.recorddir="${RECORD_DIR}" \
  -jar ModTheSpire.jar --mods basemod,tracelab --skip-intro \
  >"${LOG}" 2>&1
