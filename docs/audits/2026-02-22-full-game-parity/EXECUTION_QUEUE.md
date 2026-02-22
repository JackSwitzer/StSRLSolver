# Execution Queue: Full-Game Parity Campaign

Last updated: 2026-02-22

## Baseline
- Branch: `main` (current merged baseline, includes PR `#24`)
- Suite baseline: `4663 passed, 5 skipped, 0 failed`
- Policy: feature-sized commits, region-sized PRs

## Mandatory core loop
1. docs: update domain + manifest + Java refs
2. tests: add/adjust assertions first
3. code: minimal parity-correct implementation
4. commit: one feature ID only
5. tracker update: `TODO.md` + `testing/test-baseline.md` + domain status

## Lane model (integrator enforced)
- Lane A: audit/intel (inventory diffs, Java refs, queue quality)
- Lane B: targeted code edit for one feature ID
- Lane C: targeted tests + regression lock
- Lane D: integrator (full-suite gate + tracker sync)

## Immediate execution batches

### Batch R1-A (completed)
- `REL-003` Orrery explicit selection surface (completed)
- `REL-004` Bottled relic assignment selection surface (completed)
- `REL-008` Dolly's Mirror explicit selection surface (completed)

### Batch R1-B
- `REL-005` deterministic selection IDs/validation consistency (completed)
- `REL-006` relic alias normalization + `Toolbox` coverage (completed)
- `REL-007` boss/chest/reward ordering edge cases (completed)

### Batch R2
- `EVT-001` event selection follow-up actions (completed)
- `EVT-002` wire selected card index to handler execution (completed)
- `EVT-003` deterministic multi-phase transitions (completed)
- `EVT-004` event alias/inventory parity (completed)

### Batch R3
- `RWD-001` canonical reward action emission path (completed)
- `RWD-002` canonical reward action execution path (completed)
- `RWD-003` proceed gating parity (completed)
- `RWD-004` reward modifiers parity (completed)

### Batch R4 (cards non-Defect, docs/tests/code loop)
- `CRD-INV-001` non-Defect card manifest and Java->Python mapping rows
- `CRD-SH-001` shared curse/status end-of-turn runtime closure (completed)
- `CRD-IC-*` Ironclad parity clusters
- `CRD-SI-*` Silent parity clusters
- `CRD-WA-*` Watcher parity clusters
- `CRD-SH-*` shared colorless/curse/status parity clusters

### Batch R5 (orbs + powers)
- `POW-001`, `POW-002`, `ORB-001`, `POW-003`
- `CONS-001B` remaining deterministic RNG normalization (relic/potion/orb runtime callsites)

### Batch R6 (Defect cards)
- `CRD-DE-*` Defect parity closure after orb semantics are stable

### Batch R7 (final audit and RL gate)
- `AUD-001`, `AUD-002`, `AUD-003`

## Merge gates per feature
- targeted tests green
- full suite green (`uv run pytest tests/ -q`)
- docs updated with Java refs + RNG notes
- skip delta recorded
