#!/usr/bin/env bash
# trace_diff.sh — offline parity-oracle orchestrator (docs/goal/TOOLING.md T4).
#
# Usage: scripts/trace_diff.sh <script.json>
#
# Locates the frozen Java golden for <script.json> at
# data/traces/java/<script-stem>.jsonl, runs it through the trace_replay bin
# (docs/goal/TOOLING.md T3), and writes both the replayed Rust trace and the
# divergence report to logs/traces/<script-stem>/. Never launches the game —
# if the golden is missing, this fails loudly with a "needs mint" message
# (minting a new golden is a human-attended session, docs/goal/GOAL.md).
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENGINE_MANIFEST="${REPO_ROOT}/packages/engine-rs/Cargo.toml"
VENV_PYTHON="${REPO_ROOT}/.venv/bin/python3"
XCODE_FRAMEWORKS="/Applications/Xcode.app/Contents/Developer/Library/Frameworks"

if [[ $# -ne 1 ]]; then
  echo "usage: scripts/trace_diff.sh <script.json>" >&2
  exit 2
fi

script_path="$1"
if [[ ! -f "${script_path}" ]]; then
  echo "script not found: ${script_path}" >&2
  exit 2
fi

script_stem="$(basename "${script_path}")"
script_stem="${script_stem%.json}"

golden_path="${REPO_ROOT}/data/traces/java/${script_stem}.jsonl"
if [[ ! -f "${golden_path}" ]]; then
  echo "needs mint: ${golden_path}" >&2
  exit 3
fi

out_dir="${REPO_ROOT}/logs/traces/${script_stem}"
mkdir -p "${out_dir}"

masks_path="${REPO_ROOT}/docs/goal/masks.json"

# --- reuse test_engine_rs.sh's cargo/PyO3 env setup (minimal replica) ---
if [[ ! -f "${ENGINE_MANIFEST}" ]]; then
  echo "engine-rs manifest not found: ${ENGINE_MANIFEST}" >&2
  exit 1
fi
if [[ ! -x "${VENV_PYTHON}" ]]; then
  echo "expected Python runtime missing: ${VENV_PYTHON}" >&2
  exit 1
fi
export PATH="${HOME}/.cargo/bin:${PATH}"
export PYO3_PYTHON="${VENV_PYTHON}"
if [[ -d "${XCODE_FRAMEWORKS}" ]]; then
  export DYLD_FRAMEWORK_PATH="${XCODE_FRAMEWORKS}${DYLD_FRAMEWORK_PATH:+:${DYLD_FRAMEWORK_PATH}}"
fi

cargo_args=(
  run --manifest-path "${ENGINE_MANIFEST}" --bin trace_replay --quiet --
  --script "${script_path}"
  --java-trace "${golden_path}"
  --out "${out_dir}/rust_trace.jsonl"
  --diff "${out_dir}/report.json"
)
if [[ -f "${masks_path}" ]]; then
  cargo_args+=(--masks "${masks_path}")
fi

set +e
cargo "${cargo_args[@]}"
status=$?
set -e

echo "report: ${out_dir}/report.json"
echo "rust trace: ${out_dir}/rust_trace.jsonl"
exit "${status}"
