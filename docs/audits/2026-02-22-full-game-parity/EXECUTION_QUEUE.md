# Execution Queue: Full-Game Parity Campaign

Last updated: 2026-02-24

## Baseline
- Branch: `codex/parity-d0-d2-foundation`
- Suite baseline: `4722 passed, 0 skipped, 0 failed`
- Policy: feature-sized commits, region-sized PRs

## Mandatory core loop
1. docs: update domain + manifest + Java refs
2. tests: add/adjust assertions first
3. code: minimal parity-correct implementation
4. tracker: update `TODO.md` + `CORE_TODO.md` + baseline references
5. commit: one feature ID only

## Lane model (integrator enforced)
- Lane A: audit/intel/docs (`DOC-*`, `AUD-*`)
- Lane B: card inventory/behavior (`CRD-*`)
- Lane C: powers/hooks integration (`POW-*`)
- Lane D: RNG + RL contracts (`RNG-*`, `RL-*`)

## Completed batches
- `R1`: relic selection surface + alias/order closure (`REL-003/004/005/006/007/008`)
- `R2`: event selection/action-surface closure (`EVT-001/002/003/004`)
- `R3`: reward action-surface and execution closure (`RWD-001/002/003/004`)
- `R5`: `POW-001` inventory closure + `POW-003B` long-tail hook closure + `ORB-001`
- `CONS-001A`: canonical repo lock + migration manifest + curated training migration
- `CONS-002A/B`: CombatRunner runtime unification to compatibility shim
- `AUD-001A`: deterministic parity manifest generation (`scripts/generate_parity_manifests.py`)
- `D0`: docs/process backbone (`DOC-TODO-001`, `DOC-ACTION-001`, `DOC-WFLOW-001`, `RNG-SPEC-001`)
- `D0-spec`: behavior and implementation specs (`DOC-SPEC-CRD-001`, `DOC-SPEC-POW-001`, `DOC-SPEC-RNG-001`, `DOC-SPEC-RL-001`)
- `D0-spec-index`: unified remaining-work index and spec cleanup (`DOC-SPEC-INDEX-001`)
- `D1`: generated truth refresh (`AUD-GEN-001/002/003`)
- `D2-inventory`: card class-name alias closure (`CRD-INV-003A/B`)

## Active execution batches

### Batch D3: Power behavior/order closure
- `POW-002B`: enforce runtime hook ordering parity and trigger counts
- `POW-003A`: close remaining hook-family behavior gaps
- `POW-003B`: lock cross-system integration tests

### Batch D4: Card behavior closure
- `CRD-IC-001`: Ironclad behavior parity
- `CRD-SI-001`: Silent behavior parity
- `CRD-WA-001`: Watcher behavior parity
- `CRD-SH-002`: shared card/status behavior parity
- `CRD-DE-001`: Defect behavior parity

### Batch D5: RNG authority migration
- `RNG-MOD-001`: central RNG ownership wiring
- `RNG-MOD-002`: remove direct `random.*` in parity-critical runtime modules
- `RNG-TEST-001`: deterministic seed/action replay locks

### Batch D6: RL readiness
- `RL-ACT-001`: action mask contract lock (ordered legal list + stable IDs)
- `RL-OBS-001`: human/debug observation profile lock
- `RL-DASH-001`: local runboard + combat deep dive dashboard
- `RL-SEARCH-001`: external macro planner architecture (no env API break)
- `AUD-003`: final RL launch gate

## Merge gates per feature
- targeted tests green
- full suite green (`uv run pytest tests/ -q`)
- docs updated with Java refs + RNG notes
- test delta and skip delta recorded
