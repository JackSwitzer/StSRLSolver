# Execution Queue: Full-Game Parity Campaign

Last updated: 2026-02-22

## Baseline
- Branch: `codex/parity-core-loop`
- Suite baseline: `4610 passed, 5 skipped, 0 failed`
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

### Batch R1-A (active)
- `REL-003` Orrery explicit selection surface
- `REL-004` Bottled relic assignment selection surface
- `REL-008` Dolly's Mirror explicit selection surface

### Batch R1-B
- `REL-005` deterministic selection IDs/validation consistency
- `REL-006` relic alias normalization + `Toolbox` coverage
- `REL-007` boss/chest/reward ordering edge cases

### Batch R2
- `EVT-001` event selection follow-up actions
- `EVT-002` wire selected card index to handler execution
- `EVT-003` deterministic multi-phase transitions
- `EVT-004` event alias/inventory parity

### Batch R3
- `RWD-001` canonical reward action emission path
- `RWD-002` canonical reward action execution path
- `RWD-003` proceed gating parity
- `RWD-004` reward modifiers parity

### Batch R4+
- `POW-001`, `POW-002`, `ORB-001`, `POW-003`
- `CRD-*`
- `AUD-*`

## Merge gates per feature
- targeted tests green
- full suite green (`uv run pytest tests/ -q`)
- docs updated with Java refs + RNG notes
- skip delta recorded
