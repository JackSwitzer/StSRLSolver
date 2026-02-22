# Full-Game Java Parity + RL Readiness TODO

Last updated: 2026-02-22
Canonical repo path: `/Users/jackswitzer/Desktop/SlayTheSpireRL-worktrees/parity-core-loop`

## Current baseline (verified)
- [x] Full test suite green: `4610 passed, 5 skipped, 0 failed`.
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
- [ ] `REL-008` Dolly's Mirror acquisition must expose explicit selection action.
- [ ] `EVT-001` Event choices that require card pick/remove/transform/upgrade need explicit follow-up actions.
- [ ] `EVT-002` `event_choice` execution must pass selected card index (currently forced `card_idx=None`).
- [ ] `RWD-001/RWD-002` reward/shop relic acquisition paths should route through one selection-aware execution surface.

### P1: Java inventory parity and correctness
- [ ] `REL-006` relic ID normalization + missing Java IDs (`Toolbox` confirmed gap).
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
- [ ] `REL-008` Dolly's Mirror explicit selection.
- [ ] `REL-005` deterministic selection IDs + validation hardening.
- [ ] `REL-006` relic alias normalization + `Toolbox` inventory closure.
- [ ] `REL-007` boss/chest/reward ordering edge regressions.

### R2: Event selection surface
- [ ] `EVT-001` emit pending-selection actions for event card-required choices.
- [ ] `EVT-002` wire selected card index through `take_action_dict -> EventHandler.execute_choice`.
- [ ] `EVT-003` deterministic multi-phase event transition coverage.
- [ ] `EVT-004` alias/inventory normalization and audit lock.

### R3: Reward/shop/rest/map normalization
- [ ] `RWD-001` canonical reward action emission path.
- [ ] `RWD-002` canonical reward action execution path.
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
1. `REL-008` docs rows + Dolly selection tests -> implement selection plumbing.
2. `EVT-001` docs rows + event follow-up action tests -> implement pending event selection surface.
3. `EVT-002` docs rows + selected card passthrough tests -> wire `card_idx` execution path.
4. `REL-005` deterministic selection IDs + validation hardening.

## Working loop (must follow)
1. Pick next `feature_id` from queue.
2. Update audit docs (`domain + manifest + baseline links`).
3. Add/adjust tests first.
4. Implement smallest parity-correct code change.
5. Run targeted tests, then full suite.
6. Commit one feature ID.
7. Update TODO + audit trackers.
