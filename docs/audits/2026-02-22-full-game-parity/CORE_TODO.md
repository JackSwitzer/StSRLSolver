# Core TODO: Full-Game Java Parity + RL Readiness

Last updated: 2026-03-02 (AUD-001 audit)
Execution queue: [`EXECUTION_QUEUE.md`](./EXECUTION_QUEUE.md)
Ground truth snapshot: [`GROUND_TRUTH.md`](./GROUND_TRUTH.md)
Unit chunks: [`traceability/UNIT_CHUNKS.md`](./traceability/UNIT_CHUNKS.md)
Remaining work index: [`specs/REMAINING_WORK_INDEX.md`](./specs/REMAINING_WORK_INDEX.md)
Final audit report: [`FINAL_AUDIT_REPORT.md`](./FINAL_AUDIT_REPORT.md)

## Baseline (2026-03-02)
- Full suite baseline: `5186 passed, 0 skipped, 0 failed`
- Command: `uv run pytest tests/ -q`
- Coverage: `72.71%` of `packages/engine` (line + branch)
- No executed skips in current baseline run
- 77 test files across all domains

## Test counts by domain

| Domain | Tests | Key test files |
|--------|------:|----------------|
| Enemies + AI | 418 | encounter_combat_integration (284), enemy_ai_parity (118), enemies (16) |
| Cards (all) | 804 | ironclad_card_verification (192), silent_card_verification (130), cards (115), ironclad_cards (97), silent_cards (96), watcher_card_effects (83), defect_cards (81), status_curse (107) |
| Powers | 387 | powers (118), power_handlers_new (70), power_edge_cases (65), power_registry_integration (57), audit_power_* (145) |
| Relics | 466 | relics (169), relic_registry_integration (92), relic_triggers_combat (77), relic_* others (128) |
| Potions | 348 | audit_potions (216), potions (101), potion_effects_full (56), potion_* others (75) |
| Damage/Block | 304 | damage (133), damage_edge_cases (57), audit_damage (88), audit_block (26) |
| Combat | 304 | combat (117), effects_and_combat (187) |
| Events | 215 | events (111), audit_events (104) |
| RNG | 186 | rng (90), rng_parity (45), rng_audit (31), rng_migration_determinism (20) |
| RL/Agent | 120 | agent_api (106), agent_readiness (14) |
| Map/Generation | 115 | map (68), generation (47) |
| Game Runner | 87 | game_runner (42), audit_gameloop (45) |
| Stances | 54 | audit_stances (54) |
| Other | 378 | coverage_boost (157), ascension (154), integration (43), training (17), combat_runner_compat (2), card_id_aliases (2), audit_inventory (5) |

## Global gates
- [x] PR history synced through merged PR [#25](https://github.com/JackSwitzer/StSRLSolver/pull/25).
- [x] Regions `R1` to `R3` closed and merged to `main`.
- [x] Canonical traceability manifest generation is script-driven (`scripts/generate_parity_manifests.py`).
- [ ] Every parity-critical choice interaction is explicit in action dict flow.
- [x] Normal CI path is `0 skipped, 0 failed` in current baseline run.
- [ ] RL readiness checklist is fully green.

## Region order (locked)
1. `R4` cards closure (non-Defect first)
2. `R5` powers + RNG closure
3. `R6` Defect cards closure
4. `R7` final re-audit + RL gate

## Region status

### R0 docs + scaffolding -- COMPLETE
- [x] `DOC-001` canonical suite + legacy archive pointer
- [x] `DOC-002` skill pack and core-loop process docs
- [x] `DOC-003` evidence refresh with inventory counts and gap queue
- [x] `DOC-004` merged-ground-truth docs pack (`GROUND_TRUTH`, `PR_HISTORY`, consolidation review)
- [x] `CONS-001A` canonical repo lock + wrapper migration manifest + curated training utility migration
- [x] `CONS-DESKTOP-001` one-folder Desktop realignment + archive verification
- [x] `CONS-002A` CombatRunner facade delegates runtime ownership to CombatEngine + compatibility tests
- [x] `CONS-002B` removed duplicated legacy CombatRunner runtime body; shim-only module retained
- [x] `DOC-TODO-001` unit chunk tracker and canonical TODO map
- [x] `DOC-ACTION-001` action-layer contract lock
- [x] `DOC-WFLOW-001` subagent execution loop lock
- [x] `RNG-SPEC-001` RNG stream ownership spec lock
- [x] `DOC-SPEC-CRD-001` card behavior parity spec lock
- [x] `DOC-SPEC-POW-001` power behavior/order spec lock
- [x] `DOC-SPEC-RNG-001` RNG runtime migration spec lock
- [x] `DOC-SPEC-RL-001` RL runboard/search spec lock
- [x] `DOC-SPEC-INDEX-001` remaining-work index and spec cleanup lock
- [x] `AUD-GEN-001` potion inventory source fallback in manifest generator
- [x] `AUD-GEN-002` regenerated manifest artifacts synced
- [x] `AUD-GEN-003` anomaly cleanup (`SpireHeart` policy + class-name card IDs)

### R4 cards (non-Defect first) -- MOSTLY COMPLETE
- [x] `CRD-INV-002` Java-ID inventory closure slice (`Discipline`, `Impulse`, `Gash` alias coverage)
- [x] `CRD-SH-001` shared curse/status end-of-turn runtime closure
- [x] `CRD-INV-003A` classify remaining class-name card rows (`alias-only` vs behavior)
- [x] `CRD-INV-003B` close alias-only card ID mapping gaps
- [x] `CRD-IC-001` Ironclad closure -- 62/62 cards verified, 192 tests, work unit fully checked
- [x] `CRD-SI-001` Silent closure -- 61/61 cards verified, 130 tests, work unit fully checked
- [x] `CRD-WA-001` Watcher closure -- InnerPeace + Discipline verified, 83 tests in watcher_card_effects
- [ ] `CRD-SH-002` shared colorless/curse/status closure -- 107 status/curse tests exist; formal sign-off pending

### R5 powers + RNG -- PARTIALLY COMPLETE
- [x] `POW-001` Java power inventory closure (149 Java -> Python, missing=0)
- [x] `POW-002A` dispatch coverage artifact closure (runtime hook scan now complete)
- [x] `POW-003B` long-tail hook/runtime closure (`Flight`, `Malleable`, `Invincible`, `Pen Nib`, `Equilibrium`, `Echo Form` marker path)
- [x] `ORB-001` orb infrastructure for parity-critical behaviors
- [ ] `POW-002B` hook ordering and trigger-count closure -- 134 @power_trigger decorators exist; 7 power behaviors remain unchecked
- [ ] `POW-003A` behavior closure by hook family
- [ ] `POW-003B` integration tests lock
- [x] `RNG-MOD-001` central RNG authority module closure -- EFFECTIVELY COMPLETE: zero `random.*` in registry/ or effects/
- [x] `RNG-MOD-002` direct `random.*` removal in parity-critical runtime paths -- EFFECTIVELY COMPLETE (see RNG section below)
- [ ] `RNG-TEST-001` deterministic replay checks for migrated paths -- 20 tests in test_rng_migration_determinism; full replay lock pending

### R6 cards (Defect) -- NOT STARTED
- [ ] `CRD-DE-001` Defect closure -- 0/68 effect implementations; 81 tests cover data/inventory only

### R7 final audit + RL gate
- [x] `AUD-001A` deterministic inventory/hook manifest generation pass complete
- [x] `AUD-001` final Java-vs-Python diff pass -- THIS AUDIT (2026-03-02)
- [ ] `AUD-003` RL readiness sign-off -- blocked on Defect cards and power behavior closure

## RNG migration status (2026-03-02)

### Parity-critical paths -- CLEAN
- `packages/engine/registry/` -- zero `import random` or `random.*` usage
- `packages/engine/effects/` -- zero `import random` or `random.*` usage

### Non-parity paths (acceptable)
- `generation/encounters.py` -- `import random` exists but only in a comment (line 412)
- `calc/combat_sim.py` -- `random.choice()` in `_random_agent()` fallback (simulation helper, not game engine)
- `game.py` -- `random.choice()` in test/simulation random-action loops only (not parity path)

All gameplay-critical randomness uses owned Java-parity RNG streams (`StsRandom`).

## Confirmed high-impact open gaps

Spec references for remaining work:
- `docs/audits/2026-02-22-full-game-parity/specs/REMAINING_WORK_INDEX.md`
- `docs/audits/2026-02-22-full-game-parity/specs/CARD_BEHAVIOR_PARITY_SPEC.md`
- `docs/audits/2026-02-22-full-game-parity/specs/POWER_BEHAVIOR_ORDER_SPEC.md`
- `docs/audits/2026-02-22-full-game-parity/specs/RNG_RUNTIME_MIGRATION_SPEC.md`
- `docs/audits/2026-02-22-full-game-parity/specs/RL_RUNBOARD_AND_SEARCH_SPEC.md`

- [x] Card inventory mapping is closed; behavior/timing parity for Ironclad and Silent is closed.
- [ ] Defect card behavior implementations are entirely missing (68 effects).
- [ ] Power dispatch inventory is closed; 7 specific power behaviors remain unchecked.
- [x] No direct Python `random` in parity-critical engine runtime paths (registry/, effects/).

## Policy reminders
- [ ] Per feature loop: `docs -> tests -> code -> tracker update -> commit`.
- [ ] One feature ID per commit.
- [ ] Domain PRs only (one region per PR).
- [ ] Every commit includes Java refs + RNG notes + test delta + skip delta.
