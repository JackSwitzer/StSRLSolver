# Execution Queue: Full-Game Parity Campaign

Last updated: 2026-02-22

## Baseline
- Branch: `codex/parity-core-loop`
- Suite baseline: `4610 passed, 5 skipped, 0 failed`
- Policy: feature-sized commits, domain-sized PRs

## Core loop (mandatory)
1. docs: update domain + traceability rows + expected Java behavior
2. tests: add/adjust assertions first
3. code: minimal parity-correct implementation
4. commit: one feature ID only
5. todo update: core todo + test baseline + domain status

## Swarm lane model
- Lane A (audit/intel, extra-high): cross-domain inventory, Java refs, gap manifest review.
- Lane B (targeted edits, medium): localized code fixes for one feature ID.
- Lane C (tests, medium): focused tests per feature and regression tightening.
- Lane D (integrator, extra-high): merge gating, full-suite validation, tracker synchronization.

Note: model names are execution policy only; integrator enforces output quality and determinism regardless of lane model.

## Region queue

### R0 docs + tests scaffolding
- `DOC-001` canonical suite + legacy archive pointer
- `DOC-002` Java/Python inventory + gap manifest
- `TST-001` traceability matrix tests
- `TST-002` action-surface completeness tests

### R1 relic selection surface
- `REL-003` Orrery explicit selection
- `REL-004` bottled relic assignment explicit selection
- `REL-008` Dolly's Mirror explicit selection
- `REL-005` deterministic selection IDs/validation
- `REL-006` alias normalization + missing Java IDs
- `REL-007` boss/chest/reward ordering regressions

### R2 event selection surface
- `EVT-001` event selection follow-up actions
- `EVT-002` wire event selection params through `GameRunner -> EventHandler`
- `EVT-003` deterministic multi-phase transitions
- `EVT-004` event alias normalization/inventory tests

### R3 reward flow normalization
- `RWD-001` reward action emission through `RewardHandler`
- `RWD-002` reward action execution through `RewardHandler`
- `RWD-003` proceed gating parity
- `RWD-004` reward modifier interaction parity

### R4 powers + orbs closure
- `POW-001` full Java class mapping and gap tagging
- `POW-002` hook/timing closure
- `ORB-001` parity-critical orb infrastructure
- `POW-003` power-orb-relic integration tests

### R5 cards long-tail
- `CRD-IC-*` ironclad closure
- `CRD-SI-*` silent closure
- `CRD-DE-*` defect closure
- `CRD-WA-*` watcher closure

### R6 final audit + RL gate
- `AUD-001` final diff pass
- `AUD-002` zero-skip normal CI confirmation
- `AUD-003` RL readiness sign-off

## Merge gates per feature
- targeted tests green
- full suite green before merge
- docs updated with Java reference + RNG notes
- skip delta recorded
