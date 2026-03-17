# Full-Game Java Parity + RL Readiness TODO

Last updated: 2026-02-24
Canonical repo path: `/Users/jackswitzer/Desktop/SlayTheSpireRL`

## Current baseline (verified)
- [x] Full test suite green: `4722 passed, 0 skipped, 0 failed`.
- [x] No skips in the current baseline run (`uv run pytest tests/ -q`).
- [x] Canonical parity audit suite exists under `docs/archive/audits-2026-02-22-full-game-parity/`.
- [x] Ground truth snapshot + PR ledger exist: `GROUND_TRUTH.md`, `PR_HISTORY.md`.

## Canonical process docs
- [x] `DOC-TODO-001` unit-sized chunk tracker: `docs/archive/audits-2026-02-22-full-game-parity/traceability/UNIT_CHUNKS.md`
- [x] `DOC-ACTION-001` action contract lock: `docs/archive/audits-2026-02-22-full-game-parity/action-layer/ACTION_SPACE_SPEC.md`
- [x] `DOC-WFLOW-001` subagent loop lock: `docs/archive/audits-2026-02-22-full-game-parity/process/SUBAGENT_EXECUTION_LOOP.md`
- [x] `RNG-SPEC-001` RNG stream ownership spec: `docs/archive/audits-2026-02-22-full-game-parity/rng/JAVA_RNG_STREAM_SPEC.md`
- [x] `DOC-SPEC-CRD-001` card behavior parity spec: `docs/archive/audits-2026-02-22-full-game-parity/specs/CARD_BEHAVIOR_PARITY_SPEC.md`
- [x] `DOC-SPEC-POW-001` power behavior/order spec: `docs/archive/audits-2026-02-22-full-game-parity/specs/POWER_BEHAVIOR_ORDER_SPEC.md`
- [x] `DOC-SPEC-RNG-001` RNG runtime migration spec: `docs/archive/audits-2026-02-22-full-game-parity/specs/RNG_RUNTIME_MIGRATION_SPEC.md`
- [x] `DOC-SPEC-RL-001` RL runboard/search spec: `docs/archive/audits-2026-02-22-full-game-parity/specs/RL_RUNBOARD_AND_SEARCH_SPEC.md`
- [x] `DOC-SPEC-INDEX-001` remaining-work index + spec cleanup: `docs/archive/audits-2026-02-22-full-game-parity/specs/REMAINING_WORK_INDEX.md`

## Locked execution policy
- [x] Scope is full game now (all systems, no character staging).
- [x] Feature loop is always `docs -> tests -> code -> tracker update -> commit`.
- [x] One feature ID per commit, one region per PR.
- [x] Java behavior wins when Python behavior conflicts.
- [x] Env API remains explicit primitive actions; macros stay in planner/training layer.

## Completed foundation work
- [x] `DOC-001` canonical audit suite + legacy pointer wiring.
- [x] `DOC-002` parity-core-loop skill pack for repeatable swarm/integrator loop.
- [x] `DOC-003` evidence refresh: baseline, inventory snapshots, prioritized gap queue.
- [x] `DOC-004` merged-ground-truth docs pack (`GROUND_TRUTH`, PR ledger, consolidation review).
- [x] `CONS-001A` canonical repo lock + wrapper migration manifest + curated training utility migration to `packages/training/`.
- [x] `CONS-DESKTOP-001` one-folder Desktop realignment with verified archive snapshots.
- [x] `CONS-002A` CombatRunner compatibility facade delegates runtime execution to `CombatEngine`.
- [x] `CONS-002B` duplicated legacy CombatRunner implementation removed; shim remains.
- [x] `AUD-001A` deterministic Java-vs-Python inventory/hook manifest generation via `scripts/generate_parity_manifests.py`.
- [x] `AUD-GEN-001` Java potion inventory source fallback enabled in generator.
- [x] `AUD-GEN-002` parity manifests regenerated and synced.
- [x] `AUD-GEN-003` manifest anomalies closed (`SpireHeart` policy + starter/class-name card IDs).
- [x] `CRD-INV-003A` class-name card rows classified/covered.
- [x] `CRD-INV-003B` class-name alias mapping closure (`cards missing = 0`).

## Remaining gaps

Specification baseline is now locked in:
- `docs/archive/audits-2026-02-22-full-game-parity/specs/REMAINING_WORK_INDEX.md`
- `docs/archive/audits-2026-02-22-full-game-parity/specs/CARD_BEHAVIOR_PARITY_SPEC.md`
- `docs/archive/audits-2026-02-22-full-game-parity/specs/POWER_BEHAVIOR_ORDER_SPEC.md`
- `docs/archive/audits-2026-02-22-full-game-parity/specs/RNG_RUNTIME_MIGRATION_SPEC.md`
- `docs/archive/audits-2026-02-22-full-game-parity/specs/RL_RUNBOARD_AND_SEARCH_SPEC.md`

### P1: Card behavior parity
- [ ] `CRD-IC-001` Ironclad behavior closure.
- [ ] `CRD-SI-001` Silent behavior closure.
- [ ] `CRD-WA-001` Watcher behavior closure.
- [ ] `CRD-SH-002` shared colorless/curse/status closure.
- [ ] `CRD-DE-001` Defect behavior closure.

### P1: Powers behavior/order parity
- [ ] `POW-002B` hook ordering and trigger count lock.
- [ ] `POW-003A` behavior closure by hook family.
- [ ] `POW-003B` cross-system power integration tests.

### P1: RNG runtime parity migration
- [ ] `RNG-MOD-001` central RNG module/stream authority lock.
- [ ] `RNG-MOD-002` remove direct `random.*` from parity-critical runtime paths.
- [ ] `RNG-TEST-001` seed+action determinism regression locks.

### P2: RL readiness
- [ ] `RL-ACT-001` action mask contract lock (ordered legal list + stable IDs).
- [ ] `RL-OBS-001` human/debug observation profile lock.
- [ ] `RL-DASH-001` local runboard + combat deep-dive dashboard.
- [ ] `RL-SEARCH-001` macro planner architecture (external to env API).
- [ ] `AUD-003` RL readiness sign-off.

## Region plan (PR boundaries)
- `R4`: cards closure (non-Defect first)
- `R5`: powers + RNG closure
- `R6`: Defect closure
- `R7`: final re-audit + RL gate

## Immediate next commit queue
1. `POW-002B`
2. `POW-003A`
3. `CRD-IC-001`
4. `CRD-SI-001`
5. `RNG-MOD-001`
6. `RNG-MOD-002`

## Working loop (must follow)
1. Pick next `feature_id` from queue.
2. Update audit docs (`domain + manifest + Java refs + RNG notes`).
3. Add/adjust tests first.
4. Implement minimal parity-correct code change.
5. Run targeted tests, then full suite.
6. Commit one feature ID.
7. Update TODO + audit trackers.
