# Full-Game Java Parity + RL Readiness TODO

Last updated: 2026-02-23
Canonical repo path: `/Users/jackswitzer/Desktop/SlayTheSpireRL`

## Current baseline (verified)
- [x] Full test suite green: `4708 passed, 5 skipped, 0 failed`.
- [x] Skip source is isolated to artifact-dependent replay checks in `tests/test_parity.py`.
- [x] Canonical parity audit suite exists under `docs/audits/2026-02-22-full-game-parity/`.
- [x] Ground truth snapshot + PR ledger exist: `GROUND_TRUTH.md`, `PR_HISTORY.md`.
- [x] Core-loop skill pack exists under `docs/skills/parity-core-loop/`.
- [x] Merged chain verified through PR `#25`; stale PR `#8` closed as superseded.

## Locked execution policy
- [x] Scope is full game now (all systems, no character staging).
- [x] Feature loop is always `docs -> tests -> code -> commit -> todo update`.
- [x] One feature ID per commit, one region per PR.
- [x] Java behavior wins when Python behavior conflicts.

## Completed foundation work
- [x] `DOC-001` canonical audit suite + legacy pointer wiring.
- [x] `DOC-002` parity-core-loop skill pack for repeatable swarm/integrator loop.
- [x] `DOC-003` evidence refresh: baseline, inventory snapshots, prioritized gap queue.
- [x] `DOC-004` merged-ground-truth docs pack (`GROUND_TRUTH`, PR ledger, consolidation review).
- [x] `CONS-001A` canonical repo lock + wrapper migration manifest + curated training utility migration to `packages/training/`.
- [x] `CONS-DESKTOP-001` one-folder Desktop realignment with verified archive snapshots (`docs/audits/2026-02-22-full-game-parity/traceability/desktop-realignment-2026-02-23.md`).
- [x] `AUD-001A` deterministic Java-vs-Python inventory/hook manifest generation via `scripts/generate_parity_manifests.py`.

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
- [x] `POW-001` Java power inventory closure (149 Java classes mapped; `exact=134`, `alias=15`, `missing=0` via manifest audit).
- [x] `ORB-001` orb runtime/relic closure (`Cables`, `Frozen Core`, `Emotion Chip`, `Inserter`, `Nuclear Battery`, `Symbiotic Virus`) with deterministic start-turn wiring and RNG ownership.
- [ ] `CONS-001B` finish deterministic RNG normalization in remaining parity-critical runtime paths (relic/potion/orb).
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
- [x] `RWD-003` proceed gating parity.
- [x] `RWD-004` reward modifier interaction parity.

### R4: Cards long-tail (non-Defect first)
- [x] `CRD-INV-001` non-Defect card manifest (`exact|approximate|missing`) with Java refs.
- [x] `CRD-INV-002` card inventory closure slice (`Discipline`, `Impulse`, `Gash` alias coverage).
- [x] `CRD-SH-001` shared curse/status end-of-turn runtime closure + Void draw lock.
- [ ] `CRD-IC-*`, `CRD-SI-*`, `CRD-WA-*`, `CRD-SH-*` closure.

### R5: Powers + orbs closure
- [x] `POW-001` power inventory closure with Java references.
- [ ] `POW-002` remaining hook/timing parity fixes.
- [x] `POW-003B` long-tail hook/runtime closure (`Flight`, `Malleable`, `Invincible`, `Pen Nib`, `Equilibrium`, `Echo Form` marker path).
- [x] `ORB-001` orb infrastructure required for relic/power parity.
- [ ] `POW-003` power/orb/relic integration tests.
- [ ] `CONS-001B` remaining deterministic RNG normalization in relic/potion/orb paths.

### R6: Defect cards
- [ ] `CRD-DE-*` closure.

### R7: Final audit + RL gate
- [x] `AUD-001A` generated parity manifests and canonical audit snapshot (`java-inventory.json`, `python-inventory.json`, `parity-diff.json`, `power-hook-coverage.json`).
- [ ] `AUD-001` clean Java-vs-Python diff manifests (no unresolved parity rows).
- [ ] `AUD-002` normal CI to `0 skipped, 0 failed`.
- [ ] `AUD-003` RL readiness sign-off.

## Immediate next commit queue
1. `CONS-002` unify combat runtime ownership (`CombatEngine` canonical, `CombatRunner` compatibility shim only).
2. `POW-002` close runtime dispatch for remaining registered hooks (`11` undispatched in generated hook report).
3. `CRD-INV-003` close the `21` Java card rows flagged missing by generated parity diff.

## Working loop (must follow)
1. Pick next `feature_id` from queue.
2. Update audit docs (`domain + manifest + baseline links`).
3. Add/adjust tests first.
4. Implement smallest parity-correct code change.
5. Run targeted tests, then full suite.
6. Commit one feature ID.
7. Update TODO + audit trackers.
