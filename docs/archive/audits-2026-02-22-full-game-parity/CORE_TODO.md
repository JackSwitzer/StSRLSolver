# Core TODO: Full-Game Java Parity + RL Readiness

Last updated: 2026-02-24
Execution queue: [`EXECUTION_QUEUE.md`](./EXECUTION_QUEUE.md)
Ground truth snapshot: [`GROUND_TRUTH.md`](./GROUND_TRUTH.md)
Unit chunks: [`traceability/UNIT_CHUNKS.md`](./traceability/UNIT_CHUNKS.md)
Remaining work index: [`specs/REMAINING_WORK_INDEX.md`](./specs/REMAINING_WORK_INDEX.md)

## Baseline
- Full suite baseline: `4722 passed, 0 skipped, 0 failed`
- Command: `uv run pytest tests/ -q`
- No executed skips in current baseline run

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

### R0 docs + scaffolding
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

### R4 cards (non-Defect first)
- [x] `CRD-INV-002` Java-ID inventory closure slice (`Discipline`, `Impulse`, `Gash` alias coverage)
- [x] `CRD-SH-001` shared curse/status end-of-turn runtime closure
- [x] `CRD-INV-003A` classify remaining class-name card rows (`alias-only` vs behavior)
- [x] `CRD-INV-003B` close alias-only card ID mapping gaps
- [ ] `CRD-IC-001` Ironclad closure
- [ ] `CRD-SI-001` Silent closure
- [ ] `CRD-WA-001` Watcher closure
- [ ] `CRD-SH-002` shared colorless/curse/status closure

### R5 powers + RNG
- [x] `POW-001` Java power inventory closure
- [x] `POW-002A` dispatch coverage artifact closure (runtime hook scan now complete)
- [x] `POW-003B` long-tail hook/runtime closure (`Flight`, `Malleable`, `Invincible`, `Pen Nib`, `Equilibrium`, `Echo Form` marker path)
- [x] `ORB-001` orb infrastructure for parity-critical behaviors
- [ ] `POW-002B` hook ordering and trigger-count closure
- [ ] `POW-003A` behavior closure by hook family
- [ ] `POW-003B` integration tests lock
- [ ] `RNG-MOD-001` central RNG authority module closure
- [ ] `RNG-MOD-002` direct `random.*` removal in parity-critical runtime paths
- [ ] `RNG-TEST-001` deterministic replay checks for migrated paths

### R6 cards (Defect)
- [ ] `CRD-DE-001` Defect closure

### R7 final audit + RL gate
- [x] `AUD-001A` deterministic inventory/hook manifest generation pass complete
- [ ] `AUD-001` final Java-vs-Python diff pass (behavior-level unresolved rows closed)
- [ ] `AUD-003` RL readiness sign-off

## Confirmed high-impact open gaps

Spec references for remaining work:
- `docs/archive/audits-2026-02-22-full-game-parity/specs/REMAINING_WORK_INDEX.md`
- `docs/archive/audits-2026-02-22-full-game-parity/specs/CARD_BEHAVIOR_PARITY_SPEC.md`
- `docs/archive/audits-2026-02-22-full-game-parity/specs/POWER_BEHAVIOR_ORDER_SPEC.md`
- `docs/archive/audits-2026-02-22-full-game-parity/specs/RNG_RUNTIME_MIGRATION_SPEC.md`
- `docs/archive/audits-2026-02-22-full-game-parity/specs/RL_RUNBOARD_AND_SEARCH_SPEC.md`

- [ ] Card inventory mapping is closed; behavior/timing parity remains open across class domains.
- [ ] Power dispatch inventory is closed; hook ordering and behavior parity remains open.
- [ ] Engine logic still contains direct Python `random` usage in parity-critical runtime modules.

## Policy reminders
- [ ] Per feature loop: `docs -> tests -> code -> tracker update -> commit`.
- [ ] One feature ID per commit.
- [ ] Domain PRs only (one region per PR).
- [ ] Every commit includes Java refs + RNG notes + test delta + skip delta.
