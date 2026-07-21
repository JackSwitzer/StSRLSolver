#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
ENGINE_MANIFEST="${REPO_ROOT}/packages/engine-rs/Cargo.toml"

if [[ ! -f "${ENGINE_MANIFEST}" ]]; then
  echo "engine-rs manifest not found: ${ENGINE_MANIFEST}" >&2
  exit 1
fi

export PATH="${HOME}/.cargo/bin:${PATH}"

subcommand="test"
if [[ $# -gt 0 && ( "$1" == "test" || "$1" == "check" || "$1" == "build" ) ]]; then
  subcommand="$1"
  shift
fi

exec cargo "${subcommand}" --manifest-path "${ENGINE_MANIFEST}" "$@"
