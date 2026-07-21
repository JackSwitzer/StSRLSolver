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

# State-delta cross-check (P4 gap detection): an unhooked commit produces no
# record of its own, so its effect silently rides along on the snapshot of
# the NEXT action that does get recorded — the recorder can't emit UNKNOWN
# for a decision it never observed at all. Catch that class of gap by
# diffing post.relics length and deck length between consecutive action
# records; if either changes, the record's own action.type must be one that
# can legitimately explain a relic/deck-count change. This is exactly how
# the Empty Cage gap (BOSS_RELIC never fired, HOOKS.md) was found in pilot
# sitting 2: the relic shows up in post.relics with no BOSS_RELIC action, so
# the delta lands on whatever the next commit happened to be (PATH).
#
# Allowlist derivation — every action type that can change relic count
# and/or deck size in play (a broad OR across both fields, not per-field,
# since a single click can touch either or both — e.g. Empty Cage removes 2
# cards on pickup, one BOSS_RELIC commit moves both counters):
#   REWARD_TAKE, CARD_REWARD  - combat/elite reward claims (relic/potion, card)
#   BOSS_RELIC                - boss relic pick (relic; may also alter deck,
#                                e.g. Empty Cage's -2 cards)
#   SHOP_BUY_CARD/RELIC/POTION, SHOP_REMOVE - merchant transactions
#   CHEST_OPEN                - chest relic grant
#   EVENT_CHOICE               - event outcomes (either field, either sign)
#   NEOW                       - Neow bonus/penalty (either field)
#   CAMPFIRE                   - SMITH/upgrade doesn't change deck count, but
#                                the choice set is small and any run-metric
#                                surprises here are cheap to allow explicitly
#   PROCEED                    - covers reward-popup auto-resolution timing
#                                (a claimed relic/card can settle on the
#                                stable-state snapshot paired with the
#                                following PROCEED rather than the reward
#                                action itself, depending on settle timing)
delta_findings=$(zcat -f "${trace}" | jq -sc '
  [.[] | select(.kind == null)] | sort_by(.idx) as $recs
  | ($recs | length) as $n
  | [range(1; $n)] | map(
      ($recs[.-1]) as $prev | ($recs[.]) as $cur
      | ($cur.post.relics | length) as $rc | ($prev.post.relics | length) as $rp
      | ($cur.deck | length) as $dc | ($prev.deck | length) as $dp
      | (["REWARD_TAKE","CARD_REWARD","BOSS_RELIC","SHOP_BUY_CARD",
          "SHOP_BUY_RELIC","SHOP_BUY_POTION","SHOP_REMOVE","CHEST_OPEN",
          "EVENT_CHOICE","NEOW","CAMPFIRE","PROCEED"]) as $allowed
      | select(($rc != $rp or $dc != $dp) and (($allowed | index($cur.action.type)) | not))
      | {prev_idx: $prev.idx, idx: $cur.idx, action: $cur.action.type,
         relics: [$rp, $rc], deck: [$dp, $dc]}
    )
')
delta_count=$(echo "${delta_findings}" | jq 'length')
if [[ "${delta_count}" != "0" ]]; then
  while IFS= read -r finding; do
    p=$(echo "${finding}" | jq -r '.prev_idx')
    i=$(echo "${finding}" | jq -r '.idx')
    a=$(echo "${finding}" | jq -r '.action')
    r=$(echo "${finding}" | jq -c '.relics')
    dk=$(echo "${finding}" | jq -c '.deck')
    err "unexplained state delta at idx ${p}->${i} (action=${a} relics=${r} deck=${dk}) — add the missing hook or extend the allowlist in validate_recording.sh"
  done < <(echo "${delta_findings}" | jq -c '.[]')
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
