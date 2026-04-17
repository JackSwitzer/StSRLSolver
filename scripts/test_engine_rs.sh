#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENGINE_MANIFEST="${REPO_ROOT}/packages/engine-rs/Cargo.toml"
VENV_PYTHON="${REPO_ROOT}/.venv/bin/python3"
XCODE_FRAMEWORKS="/Applications/Xcode.app/Contents/Developer/Library/Frameworks"

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

# Some previously built local test binaries have resolved against the Xcode
# Python framework loader path on macOS. Keeping this set is harmless when the
# uv-managed Python is used, and it makes focused reruns reliable.
if [[ -d "${XCODE_FRAMEWORKS}" ]]; then
  export DYLD_FRAMEWORK_PATH="${XCODE_FRAMEWORKS}${DYLD_FRAMEWORK_PATH:+:${DYLD_FRAMEWORK_PATH}}"
fi

subcommand="test"
if [[ $# -gt 0 && ( "$1" == "test" || "$1" == "check" || "$1" == "build" ) ]]; then
  subcommand="$1"
  shift
fi

exec cargo "${subcommand}" --manifest-path "${ENGINE_MANIFEST}" "$@"
