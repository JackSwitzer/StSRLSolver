# Core TODO: Full-Game Java Parity + RL Readiness

Last updated: 2026-02-22
Execution queue: [`EXECUTION_QUEUE.md`](./EXECUTION_QUEUE.md)

## Baseline
- Full suite baseline: `4654 passed, 5 skipped, 0 failed`
- Command: `uv run pytest tests/ -q`
- Skips are artifact-dependent (`tests/test_parity.py` replay file missing)

## Global gates
- [ ] Canonical traceability manifest complete for all parity-critical gaps.
- [ ] Every choice interaction uses explicit action dict flow.
- [ ] Normal CI is `0 skipped, 0 failed`.
- [ ] RL readiness checklist is fully green.

## Region order (locked)
1. R1 relic selection surface
2. R2 event selection surface
3. R3 reward/shop/rest/map normalization
4. R4 powers + orbs closure
5. R5 cards long-tail closure
6. R6 final re-audit + RL gate

## Region status

### R0 docs + scaffolding (completed)
- [x] `DOC-001` canonical suite + legacy archive pointer
- [x] `DOC-002` skill pack and core-loop process docs
- [x] `DOC-003` evidence refresh with inventory counts and gap queue

### R1 relic selection surface (active)
- [x] `REL-003` Orrery explicit selection actions
- [x] `REL-004` bottled relic assignment explicit actions
- [x] `REL-008` Dolly's Mirror explicit duplicate selection
- [x] `REL-005` deterministic selection IDs + validation
- [x] `REL-006` alias normalization + Java ID coverage (`Toolbox`)
- [x] `REL-007` boss/chest/reward ordering regressions

### R2 event selection surface
- [x] `EVT-001` event selection follow-up actions
- [x] `EVT-002` pass selected card indices through action handling
- [x] `EVT-003` deterministic multi-phase transition coverage
- [x] `EVT-004` alias/inventory normalization

### R3 reward/shop/rest/map
- [x] `RWD-001` canonical reward action emission path
- [x] `RWD-002` canonical reward action execution path
- [ ] `RWD-003` proceed gating parity
- [ ] `RWD-004` modifier interaction parity

### R4 powers + orbs
- [ ] `POW-001` Java power inventory closure
- [ ] `POW-002` residual hook/timing closure
- [ ] `ORB-001` orb infrastructure for parity-critical behaviors
- [ ] `POW-003` integration tests

### R5 cards
- [ ] `CRD-IC-*` Ironclad closure
- [ ] `CRD-SI-*` Silent closure
- [ ] `CRD-DE-*` Defect closure
- [ ] `CRD-WA-*` Watcher closure

### R6 final audit + RL gate
- [ ] `AUD-001` final diff pass
- [ ] `AUD-002` zero-skip normal CI confirmation
- [ ] `AUD-003` RL readiness sign-off

## Confirmed high-impact open gaps
- [ ] Power inventory has large class-level residuals.
- [ ] Orb-linked relic behavior still has placeholder TODO paths.

## Policy reminders
- [ ] Per feature loop: `docs -> tests -> code -> commit -> todo update`.
- [ ] One feature ID per commit.
- [ ] Domain PRs only (one region per PR).
- [ ] Every commit includes Java refs + RNG notes + test delta + skip delta.
