# Audit 1 — F1-F4 determinism closure

**Status:** REOPEN (docs-only — code is CLEAN)
**Rows audited:** F1, F2, F3, F4 (D162-D165)
**Rust test count:** 5/5 pass (`test_snapshot_determinism`) + 4/4 pass (`training_contract`) + 42/42 pass (`pytest tests/training`)

## Findings

### Code (all CLEAN)
- OK F1 / D162 — `CombatSnapshotV1` has `ai_rng_seed0/seed1/counter` at `packages/engine-rs/src/training_contract.rs:575-579`; populated from `engine.ai_rng.state_tuple()` at `:685`; restored via `set_state_tuple` at `:839-841`.
- OK F2 / D163 — `EnemySnapshotV1.move_history: Vec<i32>` at `training_contract.rs:528`; serialized at `:745`; restored BEFORE `set_move` at `:778-782` (comment explicit on ordering).
- OK F3 / D164 — `combat_state_hash` hashes both streams at `search.rs:1095` (`engine.rng.state_tuple()`) and `:1096` (`engine.ai_rng.state_tuple()`). Grep confirms exactly 2 `state_tuple()` occurrences.
- OK F4 / D165 — `test_snapshot_determinism.rs` has all three required assertions:
  - `move_id` twin match: `:104` (`snapshot_continuation_produces_same_next_move`)
  - `ai_rng.state_tuple()` twin match: `:114`
  - `move_history` twin match: `:109`
  - Plus per-item roundtrip tests for ai_rng (`:46`), history (`:73`), and dual hash-divergence tests (`:141`, `:166`).
- OK Python symmetry — `packages/training/contracts.py` carries `move_history` (`:374`), `ascension` (`:379`), `ai_rng_seed0/1/counter` (`:418-420`); `parse_combat_snapshot` round-trips all at `:516-517` and `:528-530`.
- OK Test regression — zero failures across all three test runs.

### Documentation (REOPEN)
- FAIL `docs/work_units/parity-deviations-register.md:236-239` — D162, D163, D164, D165 all still marked `**open**` with no closure note. Cycle 2 landed these fixes in commit `f30066fe` ("determinism: Cycle 2 — F1-F4 snapshot/hash RNG + move_history") but the register was never stamped. This violates the documented pattern used for D152/D154/D155/D156 (stamped in `44069a3f`), D70/D89/D111 (stamped in `9e6555c4`), and D91/D124 (stamped in `03cff674`).

## Recommendation

**Ship PR #138 conditional on a one-line docs patch.** The code closure is fully correct — all four merge-blockers are resolved, tests are green, Python contract mirrors Rust. The only gap is docs hygiene: D162-D165 rows need status flipped to `**closed 2026-04-21** — <summary>. Commit f30066fe (Cycle 2 / PR #138).` matching the established stamp-SHA pattern.

Without that stamp, a future audit will mis-triage these as still-open P0 blockers. Block merge only if docs stamps are required by repo convention; otherwise stamp in a fast-follow commit before PR #138 merges.
