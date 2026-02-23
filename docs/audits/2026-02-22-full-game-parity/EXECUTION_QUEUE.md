# Execution Queue: Full-Game Parity Campaign

Last updated: 2026-02-23

## Baseline
- Branch: `codex/cons-aud-001`
- Suite baseline: `4708 passed, 5 skipped, 0 failed`
- Policy: feature-sized commits, region-sized PRs

## Mandatory core loop
1. docs: update domain + manifest + Java refs
2. tests: add/adjust assertions first
3. code: minimal parity-correct implementation
4. commit: one feature ID only
5. tracker update: `TODO.md` + `testing/test-baseline.md` + domain status

## Lane model (integrator enforced)
- Lane A: audit/intel (generated inventory diffs, Java refs, queue quality)
- Lane B: targeted code edits for one feature ID
- Lane C: targeted tests + regression lock
- Lane D: integrator (full-suite gate + tracker sync)

## Completed batches
- `R1`: relic selection surface + alias/order closure (`REL-003/004/005/006/007/008`)
- `R2`: event selection/action-surface closure (`EVT-001/002/003/004`)
- `R3`: reward action-surface and execution closure (`RWD-001/002/003/004`)
- `R5`: `POW-001` inventory closure + `POW-003B` long-tail hook closure + `ORB-001`
- `CONS-001A`: canonical repo lock + migration manifest + curated training migration
- `AUD-001A`: deterministic parity manifest generation (`scripts/generate_parity_manifests.py`)

## Active execution batches

### Batch C1: Combat runtime unification
- `CONS-002A`: enumerate `CombatRunner` consumers and replace with `CombatEngine` paths
- `CONS-002B`: convert `CombatRunner` to compatibility shim
- `CONS-002C`: remove shim after zero consumers

### Batch C2: Power dispatch closure
- `POW-002A`: dispatch hooks for damage/block/card-draw/scry/hp-loss families
- `POW-002B`: add hook-order tests and trigger-count tests
- `POW-002C`: ensure both runtimes (until unification complete) share dispatch order

### Batch C3: Card inventory/behavior closure
- `CRD-INV-003A`: close starter-ID mapping gaps from generated list
- `CRD-INV-003B`: close non-starter missing card rows (`Alchemize`, `Apparition`, etc.)
- `CRD-INV-003C`: lock with manifest-backed tests

### Batch C4: Final readiness
- `CONS-001B`: finish RNG normalization in parity-critical paths
- `AUD-002`: move replay checks to dedicated profile and hit `0 skipped` in default CI
- `AUD-003`: RL launch sign-off

## Merge gates per feature
- targeted tests green
- full suite green (`uv run pytest tests/ -q`)
- docs updated with Java refs + RNG notes
- skip delta recorded
