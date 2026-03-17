# RNG Runtime Migration Spec (RNG-MOD-001 / RNG-MOD-002 / RNG-TEST-001)

Last updated: 2026-02-24
Status: spec-lock complete
Parent index: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/specs/REMAINING_WORK_INDEX.md`

## Objective
Remove remaining direct Python `random.*` usage from parity-critical runtime paths and enforce Java-style owned RNG streams.

## Source of truth
- RNG stream ownership: `/Users/jackswitzer/Desktop/SlayTheSpireRL/docs/archive/audits-2026-02-22-full-game-parity/rng/JAVA_RNG_STREAM_SPEC.md`
- RNG implementation: `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/state/rng.py`
- Runtime modules in scope:
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/registry/relics.py`
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/registry/potions.py`
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/effects/defect_cards.py`
  - `/Users/jackswitzer/Desktop/SlayTheSpireRL/packages/engine/game.py` (helper loops only)

## Locked stream ownership map

### `packages/engine/registry/relics.py`
- `Warped Tongs` -> `card_random_rng`
- `Mark of Pain` draw-pile shuffle -> `shuffle_rng`
- `Enchiridion` -> `card_random_rng`
- `Nilry's Codex` -> `card_random_rng`
- `Mummified Hand` -> `card_random_rng`
- `Dead Branch` -> `card_random_rng`
- `Strange Spoon` -> `misc_rng`
- `Tingsha` random enemy -> `card_random_rng`
- `The Specimen` transfer target -> `card_random_rng`
- `War Paint` pickup upgrades -> `misc_rng`
- `Whetstone` pickup upgrades -> `misc_rng`

### `packages/engine/registry/potions.py`
- `SneckoOil` rerolls -> `card_random_rng`
- `EntropicBrew` fill empty slots -> `potion_rng`

### `packages/engine/effects/defect_cards.py`
- `Thunder Strike`, `Rip and Tear` random hits -> `card_random_rng`
- `Reboot` shuffle -> `shuffle_rng`
- `White Noise` random power -> `card_random_rng`

### `packages/engine/game.py`
- `run()` and `run_to_floor()` random pickers are non-parity helpers.
- Either migrate to owned streams or explicitly mark as helper-only, never parity authority.

## Unit features and acceptance

### `RNG-MOD-001` stream plumbing closure
Dependencies: `RNG-SPEC-001`

Scope:
1. Normalize stream access helpers for touched modules.
2. Remove ad-hoc fallback behavior in stream lookup.

Acceptance:
- No missing-stream runtime errors in touched modules.
- Existing RNG tests remain green.

### `RNG-MOD-002` callsite migration closure
Dependencies: `RNG-MOD-001`

Scope:
1. Replace mapped `random.*` callsites with owned streams.
2. Remove fallback direct-random branches in parity-critical code paths.

Acceptance:
- `rg "random\." packages/engine/registry/relics.py packages/engine/registry/potions.py packages/engine/effects/defect_cards.py` shows no parity-critical leftovers.
- Touched mechanic tests are green.

### `RNG-TEST-001` determinism lock
Dependencies: `RNG-MOD-002`

Scope:
1. Add seed + action replay assertions for migrated mechanics.
2. Add stream-advancement assertions where feasible.

Acceptance:
- Same seed + same actions gives identical outcomes for migrated flows.
- Full suite remains green.

## Required evidence per feature commit
1. Exact callsite list changed.
2. Stream mapping note for each callsite.
3. Determinism test evidence.
4. Tracker updates in `TODO.md`, `CORE_TODO.md`, and `UNIT_CHUNKS.md`.

## Done definition
1. `RNG-MOD-001`, `RNG-MOD-002`, and `RNG-TEST-001` are `completed` in `UNIT_CHUNKS.md`.
2. No unresolved direct-random parity paths remain in mapped modules.
3. Determinism tests for migrated flows are stable.
