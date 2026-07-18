#!/usr/bin/env bash
# validate_recording.sh — offline completeness check for a record-mode run dir
# (SPEC-tracelab-record-mode.md P4). Proves the recording is sufficient to
# recreate the run: schema-valid, gapless indices, no UNKNOWN actions, all 13
# RNG counters present on every record.
#
# Usage: scripts/validate_recording.sh data/traces/recordings/<run-id>
set -euo pipefail

dir="${1:?usage: validate_recording.sh <recording-dir>}"
meta="${dir}/meta.json"
script="${dir}/script.jsonl"
trace="${dir}/trace.jsonl.gz"
fail=0

err() { echo "FAIL: $*" >&2; fail=1; }

[[ -f "${meta}" ]] || { echo "FAIL: missing meta.json" >&2; exit 1; }
[[ -f "${script}" ]] || { echo "FAIL: missing script.jsonl" >&2; exit 1; }
[[ -f "${trace}" ]] || { echo "FAIL: missing trace.jsonl.gz" >&2; exit 1; }

jq -e . "${meta}" >/dev/null || err "meta.json is not valid JSON"
status="$(jq -r '.status' "${meta}")"
records="$(jq -r '.records' "${meta}")"

# Every line parses.
if ! zcat -f "${trace}" | jq -e . >/dev/null 2>&1; then
  err "trace contains an unparseable line"
fi
if ! jq -e . "${script}" >/dev/null 2>&1 && ! jq -es . "${script}" >/dev/null 2>&1; then
  err "script.jsonl contains an unparseable line"
fi

# Action records: gapless idx sequence 0..N-1 matching meta.records.
actual_idx=$(zcat -f "${trace}" | jq -s '[.[] | select(.kind == null) | .idx]')
expected_idx=$(jq -n --argjson n "${records}" '[range(0; $n)]')
if [[ "${actual_idx}" != "${expected_idx}" ]]; then
  err "trace idx sequence has gaps or disorder (expected 0..$((records-1)))"
fi

# Script/trace action counts agree.
script_actions=$(jq -s '[.[] | select(.lifecycle == null)] | length' "${script}")
if [[ "${script_actions}" != "${records}" ]]; then
  err "script has ${script_actions} actions but meta.records=${records}"
fi

# No UNKNOWN actions.
unknown=$(zcat -f "${trace}" | jq -s '[.[] | select(.action.type == "UNKNOWN")] | length')
if [[ "${unknown}" != "0" ]]; then
  err "${unknown} UNKNOWN action(s) — add the missing hook (HOOKS.md)"
fi

# All 13 RNG counters, non-null, on every action record.
bad_rng=$(zcat -f "${trace}" | jq -s '
  [.[] | select(.kind == null) | .post.rng
   | select((keys | length) != 13 or ([.[]] | any(. == null or . == -1)))] | length')
if [[ "${bad_rng}" != "0" ]]; then
  err "${bad_rng} record(s) missing complete 13-stream RNG counters"
fi

# Lifecycle sanity: starts with RUN_START or RESUME; ended runs end with RUN_END.
first_life=$(zcat -f "${trace}" | jq -rs '[.[] | select(.kind == "lifecycle")][0].type')
[[ "${first_life}" == "RUN_START" || "${first_life}" == "RESUME" ]] \
  || err "first lifecycle record is ${first_life}, expected RUN_START/RESUME"
if [[ "${status}" != "in_progress" ]]; then
  last_life=$(zcat -f "${trace}" | jq -rs '[.[] | select(.kind == "lifecycle")][-1].type')
  [[ "${last_life}" == "RUN_END" ]] || err "run status=${status} but last lifecycle is ${last_life}"
fi

if [[ "${fail}" == "0" ]]; then
  echo "OK: ${dir} (${records} actions, status=${status})"
else
  exit 1
fi
