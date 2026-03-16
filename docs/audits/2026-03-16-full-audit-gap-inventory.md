# Full Audit Gap Inventory (2026-03-16)

This audit is intentionally findings-first. It does not claim that the engine or
training stack is unusable; it records the highest-value places where current
docs, tests, and runtime behavior disagree.

## Evidence Sources

- Current repo code paths under `packages/engine`, `packages/training`,
  `packages/server`, `packages/parity`, `scripts`, and `tests`
- Archived local run artifacts under `~/Desktop/sts-archive/logs/runs`
- Decompiled Java reference under `~/Desktop/sts-archive/decompiled/java-src`

## Ranked Findings

| ID | Priority | Area | Summary |
|---|---|---|---|
| `RUNTIME-001` | P0 | Runtime parity | Event-triggered fights enter generic hallway combat instead of the requested event encounter. |
| `RUNTIME-002` | P1 | Runtime parity | `?` rooms are flattened to `EVENT`, so Java question-room resolution and `Tiny Chest` behavior are missing in the real loop. |
| `RUNTIME-003` | P1 | Runtime parity | Burning-elite map wiring never reaches runtime reward generation, so emerald-key elites behave like normal elites. |
| `RUNTIME-004` | P1 | Runtime parity | `GameRunner` campfire actions diverge from Java; blocked smith actions still appear under `Fusion Hammer`. |
| `LOG-001` | P1 | Replay | `ReplayRunner` does not replay beyond floor 0 and overstates what it verified. |
| `LOG-002` | P1 | Observability | Episode/run artifacts are non-canonical and too lossy to reconstruct rewards, event flow, potion use, or path context. |
| `LOG-003` | P2 | Tooling | `scripts/training-status.sh` sends `get_status`, but that message is not part of the WebSocket protocol. |
| `RL-001` | P0 | RL contract | Strategic inference is blind to candidate semantics and only sees `obs + n_actions`, not the engine action surface. |
| `RL-002` | P0 | RL contract | Strategic action heads are fixed at `256`, but the engine can emit legal follow-up action surfaces larger than that. |
| `RL-003` | P1 | RL objective | Reward shaping and replay mixing are not aligned with the stated EV-focused objective. |
| `DOC-001` | P2 | Docs/tests | Several parity/readiness docs still read as handler-complete or RL-ready even though runtime and artifact gaps remain. |

## Findings Detail

### `RUNTIME-001` Event Combat Handoff Is Broken

- `GameRunner._handle_event_action()` logs `combat_encounter`, then calls
  `_enter_combat(is_elite=False, is_boss=False)` without routing the requested
  event encounter into runtime combat setup.
- Confirmed against `MaskedBandits`, `Mushrooms`, `Colosseum`, `MindBloom`, and
  `Dead Adventurer`.
- Fix direction:
  - preserve the event-requested encounter name
  - enter combat through an event-aware path
  - resolve post-combat event state instead of generic hallway rewards

### `RUNTIME-002` Runtime `?` Room Resolution Is Missing

- Map nodes store literal `RoomType.EVENT`, and runtime room entry calls
  `_enter_event()` directly.
- Java question-room resolution depends on a separate unknown-room roll.
- Side effects:
  - `Tiny Chest` counter never advances through the real loop
  - room-entry relic behavior is evaluated against `"EVENT"` instead of the
    rolled room type

### `RUNTIME-003` Burning Elite Wiring Is Missing

- Map generation marks `has_emerald_key`, but room entry never transfers that
  flag into `GameRunner.is_burning_elite`.
- Reward generation depends on `is_burning_elite`, so the emerald-key reward
  surface is absent at runtime.

### `RUNTIME-004` Campfire Action Surface Is Too Loose

- `RestHandler.get_options()` correctly blocks smithing under `Fusion Hammer`,
  but `GameRunner._get_rest_actions()` still emits per-card upgrade actions.
- Forced invalid campfire actions also consume the room in ways Java would not.

### `LOG-001` ReplayRunner Is Floor-0-Only

- `ReplayRunner.run()` creates a `GameRunner`, compares the initial state, and
  stops.
- `max_floors` is unused.
- `floors_replayed` reports the number of expected states, not the number of
  actual replay comparisons.

### `LOG-002` Artifacts Are Non-Canonical And Lossy

- `EpisodeLog`, `overnight.py`, `training_runner.py`, and dashboard-oriented
  summaries all serialize different subsets of state.
- Missing or inconsistent fields include:
  - `run_id`
  - candidate action IDs
  - `phase_trace`
  - `decision_trace`
  - `combat_trace`
  - full reward detail
  - potion identity/slot/target/outcome
  - path option sets
  - non-empty `event_id`

### `LOG-003` Training Status Tooling Does Not Match Protocol

- `scripts/training-status.sh` sends `get_status`.
- `packages/server/protocol.py` does not define `get_status`.
- `packages/server/ws_server.py` does not advertise a handler for it.

### `RL-001` Strategic Policy Is Blind To Candidate Semantics

- The engine already exposes semantic action dicts through:
  - `GameRunner.get_available_action_dicts()`
  - `GameRunner.take_action_dict()`
  - `GameRunner.get_observation(profile="human")`
- The main training path still sends strategic inference requests as
  `obs + n_actions`, which trains a slot-selection policy rather than a policy
  over semantic actions.

### `RL-002` Strategic Action Heads Are Too Small

- `StrategicNet` and `MLXStrategicNet` both default to `action_dim=256`.
- The engine can emit larger legal surfaces for exhaustive `select_cards`
  follow-up actions.
- Current overflow behavior truncates the mask rather than representing the full
  legal surface.

### `RL-003` Reward / Replay Do Not Match The EV Goal

- Reward shaping mixes PBRS with hard-coded stance, card-pick, milestone, shop,
  and potion heuristics.
- Replay mixing samples stale transitions and reconstructs PPO episode structure
  from replayed fragments.
- This is not equivalent to the repo’s stated
  `EV(decision) = P(win | decision) - P(win | baseline)` target.

### `DOC-001` Existing Status Docs Overstate Closure

- Handler-level event coverage and inventory parity are stronger than runtime
  parity.
- Some docs describe state availability that is true inside the engine but not
  true of persisted training or replay artifacts.

## Required Audit Artifact Schema

Future implementation work should treat the following as required, not optional:

- `run_id`
- config snapshot
- git SHA
- resume lineage
- `phase_trace`
- `decision_trace`
- `combat_trace`
- full reward source and payload
- `event_id`
- potion identity, slot, target, and outcome
- path options and chosen node
- timestamps for run, floor, decision, and combat events

## Gap-Test Mapping

- `RUNTIME-001` -> `test_gap_runtime_001_*`, `test_gap_runtime_002_*`
- `RUNTIME-002` -> `test_gap_runtime_003_*`
- `RUNTIME-003` -> `test_gap_runtime_004_*`
- `RUNTIME-004` -> `test_gap_runtime_005_*`
- `LOG-001` -> `test_gap_log_001_*`
- `LOG-002` -> `test_gap_log_002_*`
- `LOG-003` -> `test_gap_log_003_*`
- `RL-001` -> `test_gap_rl_001_*`
- `RL-002` -> `test_gap_rl_002_*`
