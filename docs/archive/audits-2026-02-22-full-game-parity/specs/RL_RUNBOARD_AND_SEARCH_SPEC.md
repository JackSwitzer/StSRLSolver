# RL Runboard + Search Spec (RL-ACT / RL-OBS / RL-DASH / RL-SEARCH)

Last updated: 2026-02-24
Status: spec-lock complete
Parent index: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/audits/2026-02-22-full-game-parity/specs/REMAINING_WORK_INDEX.md`

## Objective
Define RL-facing closure work after parity-critical cards, powers, and RNG are stable.

## Locked policy decisions
1. Environment API stays primitive action only.
2. Selection stays two-step (`requires_selection` then `select_*`).
3. Invalid actions are pruned from legal candidates and hard-rejected if submitted.
4. Training uses the human-visible observation profile.
5. Macro planning lives outside environment API in trainer/search code.

## Dependencies for RL feature execution
- Required parity closure before RL implementation focus:
  - `POW-002B`
  - `POW-003A`
  - `RNG-MOD-001`
  - `RNG-MOD-002`
  - `RNG-TEST-001`

## Unit features and acceptance

### `RL-ACT-001` action mask contract lock
Dependencies: `DOC-ACTION-001`

Scope:
1. Canonical legal action export is deterministic and stably ordered.
2. Stable action IDs are preserved for equivalent snapshots.
3. Trainer-side mask builder consumes `get_available_action_dicts()` directly.

Acceptance:
- `tests/test_agent_api.py` and `tests/test_agent_readiness.py` include mask and ID stability checks.
- Action contract doc remains aligned with runtime behavior.

### `RL-OBS-001` observation profile lock
Dependencies: `RL-ACT-001`

Scope:
1. Lock `human` profile fields for training.
2. Lock `debug` profile for diagnostics/eval only.
3. Preserve schema version fields as non-breaking contract markers.

Acceptance:
- Profile tests validate field-level inclusion/exclusion.
- `observation_schema_version` and `action_schema_version` remain stable.

### `RL-DASH-001` local runboard and deep-dive dashboard
Dependencies: `RL-ACT-001`, `RL-OBS-001`

Scope:
1. Runboard view: seed/run metrics, progression, win rate, HP loss, resource usage.
2. Deep-dive view: per-turn candidates, selected action IDs, transition traces.
3. Local-first ingestion from episode artifacts.

Acceptance:
- Dashboard loads local artifacts with no manual pre-processing.
- Ingestion and rendering smoke tests pass.

### `RL-SEARCH-001` external planner and fight search
Dependencies: `RL-ACT-001`

Scope:
1. Planner emits only legal primitive actions.
2. Default budget: 10 workers and up to 100 fight-node simulations.
3. Default objective: win probability with optional resource-shaping metrics.

Acceptance:
- Planner tests show legal primitive output only.
- Fixed seed + fixed planner config gives deterministic outputs.

## Data contracts

### Episode artifact minimum
- `run_id`, `seed`, `ascension`, `timestamp`
- `phase_trace[]` (candidates, selected action, result)
- `combat_trace[]`
- `reward_trace[]`, `event_trace[]`, `shop_trace[]`
- `summary` metrics

### Search artifact minimum
- `node_id`, `depth`, `state_hash`
- `candidate_action_ids[]`
- `rollout_count`, `value_estimate`, `policy_prior`
- resource delta metrics

## Required evidence per feature commit
1. Contract updates in action/observation docs.
2. Test evidence for mask validity, profile safety, and determinism.
3. Tracker updates in `TODO.md`, `CORE_TODO.md`, and `UNIT_CHUNKS.md`.

## Done definition
1. `RL-ACT-001`, `RL-OBS-001`, `RL-DASH-001`, and `RL-SEARCH-001` are `completed`.
2. RL readiness checklist in `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/audits/2026-02-22-full-game-parity/rl/rl-readiness.md` is green.
3. Runtime API remains backward-compatible.
