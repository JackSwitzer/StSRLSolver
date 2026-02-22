# Full-Game Java Parity + RL Readiness TODO

Last updated: 2026-02-22
Canonical repo path: `/Users/jackswitzer/Desktop/SlayTheSpireRL-worktrees/parity-core-loop`

## Current baseline (verified)
- [x] Full test suite green: `4654 passed, 5 skipped, 0 failed`.
- [x] Skip source is isolated to artifact-dependent replay checks in `tests/test_parity.py`.
- [x] Canonical parity audit suite exists under `docs/audits/2026-02-22-full-game-parity/`.
- [x] Core-loop skill pack exists under `docs/skills/parity-core-loop/`.

## Locked execution policy
- [x] Scope is full game now (all systems, no character staging).
- [x] Feature loop is always `docs -> tests -> code -> commit -> todo update`.
- [x] One feature ID per commit, one region per PR.
- [x] Java behavior wins when Python behavior conflicts.

## Completed foundation work
- [x] `DOC-001` canonical audit suite + legacy pointer wiring.
- [x] `DOC-002` parity-core-loop skill pack for repeatable swarm/integrator loop.
- [x] `DOC-003` evidence refresh: baseline, inventory snapshots, prioritized gap queue.

## Evidence-based remaining gaps

### P0: Action-surface completeness (agent traversal)
- [x] `REL-003` Orrery purchase/reward flow now exposes explicit `select_cards` follow-up actions.
- [x] `REL-004` Bottled relic acquisition now exposes explicit selection actions in reward/shop action flow.
- [x] `REL-008` Dolly's Mirror acquisition now exposes explicit selection action in reward/shop action flow.
- [x] `EVT-001` Event choices that require card pick/remove/transform/upgrade now expose explicit follow-up actions.
- [x] `EVT-002` `event_choice` execution now passes selected card index to handler execution.
- [x] `RWD-001/RWD-002` runner reward action emission/execution now route through a single RewardHandler-backed surface.

### P1: Java inventory parity and correctness
- [x] `REL-006` relic ID normalization + missing Java IDs (`Toolbox` closed).
- [ ] `POW-001` Java power inventory closure (149 Java classes vs 94 Python entries; 69 normalized missing candidates).
- [ ] `ORB-001` remove placeholder orb-linked relic behavior in `packages/engine/registry/relics.py`.
- [ ] Convert audit tests that currently "document known bug" into parity assertions after fixes.

### P2: CI/readiness cleanup
- [ ] Split artifact replay checks from default CI (`tests/test_parity.py` skips).
- [ ] Add parity-campaign matrix tests for action-surface + traceability closure.
- [ ] Freeze action/observation contract for RL training.

## Region plan (PR boundaries)

### R1: Relic selection surface
- [x] `REL-003` Orrery explicit selection.
- [x] `REL-004` Bottled relic explicit assignment.
- [x] `REL-008` Dolly's Mirror explicit selection.
- [x] `REL-005` deterministic selection IDs + validation hardening.
- [x] `REL-006` relic alias normalization + `Toolbox` inventory closure.
- [x] `REL-007` boss/chest/reward ordering edge regressions.

### R2: Event selection surface
- [x] `EVT-001` emit pending-selection actions for event card-required choices.
- [x] `EVT-002` wire selected card index through `take_action_dict -> EventHandler.execute_choice`.
- [x] `EVT-003` deterministic multi-phase event transition coverage.
- [x] `EVT-004` alias/inventory normalization and audit lock.

### R3: Reward/shop/rest/map normalization
- [x] `RWD-001` canonical reward action emission path.
- [x] `RWD-002` canonical reward action execution path.
- [ ] `RWD-003` proceed gating parity.
- [ ] `RWD-004` reward modifier interaction parity.

### R4: Powers + orbs closure
- [ ] `POW-001` power inventory closure with Java references.
- [ ] `POW-002` remaining hook/timing parity fixes.
- [ ] `ORB-001` orb infrastructure required for relic/power parity.
- [ ] `POW-003` power/orb/relic integration tests.

### R5: Cards long-tail
- [ ] `CRD-IC-*`, `CRD-SI-*`, `CRD-DE-*`, `CRD-WA-*` closure.

### R6: Final audit + RL gate
- [ ] `AUD-001` clean Java-vs-Python diff manifests.
- [ ] `AUD-002` normal CI to `0 skipped, 0 failed`.
- [ ] `AUD-003` RL readiness sign-off.

## Immediate next commit queue
1. `RWD-003` proceed gating parity.
2. `RWD-004` reward modifier interaction parity.
3. `POW-001` power inventory closure with Java refs + behavior locks.

## Working loop (must follow)
1. Pick next `feature_id` from queue.
2. Update audit docs (`domain + manifest + baseline links`).
3. Add/adjust tests first.
4. Implement smallest parity-correct code change.
5. Run targeted tests, then full suite.
6. Commit one feature ID.
7. Update TODO + audit trackers.
