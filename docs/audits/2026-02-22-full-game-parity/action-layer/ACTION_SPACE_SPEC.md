# Action Space Spec (Canonical)

Last updated: 2026-02-24
Feature ID: `DOC-ACTION-001`

## Goals
- Preserve the current environment API (no macro action types added).
- Keep action emission deterministic and replay-safe.
- Represent all required selections as explicit follow-up actions.

## Public API (unchanged)
- `GameRunner.get_available_action_dicts()`
- `GameRunner.take_action_dict()`
- `GameRunner.get_observation()`

## Action schema versions
- Observation root includes:
  - `observation_schema_version`
  - `action_schema_version`
- Version fields are compatibility markers, not mode switches.

## Canonical legal-action contract
- Legal actions are emitted as a deterministic ordered list.
- Each action has a stable `id` for equivalent state snapshots.
- Training mask is built over this ordered list of legal actions.

## Phase action surface
- `neow`: `neow_choice`
- `map`: `path_choice`
- `combat`: `play_card`, `use_potion`, `end_turn`
- `combat_rewards`: `pick_card`, `skip_card`, `singing_bowl`, `claim_gold`, `claim_potion`, `skip_potion`, `claim_relic`, `claim_emerald_key`, `skip_emerald_key`, `proceed_from_rewards`
- `boss_rewards`: `pick_boss_relic`, `skip_boss_relic`
- `event`: `event_choice`
- `shop`: `buy_card`, `buy_relic`, `buy_potion`, `remove_card`, `leave_shop`
- `rest`: `rest`, `smith`, `dig`, `lift`, `toke`, `recall`
- `treasure`: `take_relic`, `sapphire_key`

## Selection contract (canonical)
- Selection-required mechanics are two-step:
  1. Primary action is attempted (for example `use_potion` / `pick_boss_relic` / `event_choice`).
  2. If selection args are missing, engine returns:
     - `requires_selection: true`
     - `candidate_actions: [...]` containing `select_cards` or `select_stance`.
- Follow-up action types:
  - `select_cards`
  - `select_stance`

## Invalid action policy
- Invalid actions are expected to be masked/pruned by caller.
- If invalid action is still submitted:
  - return structured error,
  - do not mutate state,
  - do not apply fallback implicit behavior.

## Macro policy
- Macro planning is external to environment API.
- Planner/trainer may compose macro decisions into primitive legal actions.
- Engine remains primitive-action ground truth for parity and replay.

## Determinism guarantees
- Same seed + same action ID sequence must yield same observations/rewards.
- Action IDs must remain stable across equivalent snapshots and selection contexts.
