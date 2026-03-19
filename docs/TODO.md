# TODO

Last updated: 2026-03-18
Baseline: 6100+ tests passing, training active with action-encoded observations.

## P1: Card behavior parity
- [ ] `CRD-IC-001` Ironclad behavior closure -- [work unit](work_units/granular-cards-ironclad.md)
- [ ] `CRD-SI-001` Silent behavior closure -- [work unit](work_units/granular-cards-silent.md)
- [ ] `CRD-WA-001` Watcher behavior closure -- [work unit](work_units/granular-cards-watcher.md)
- [ ] `CRD-SH-002` Shared colorless/curse/status closure -- [work unit](work_units/granular-cards-shared.md)
- [ ] `CRD-DE-001` Defect behavior closure -- [work unit](work_units/granular-cards-defect.md)

## P1: Powers behavior/order parity
- [ ] `POW-002B` Hook ordering and trigger count lock -- [work unit](work_units/granular-powers.md)
- [ ] `POW-003A` Behavior closure by hook family -- [work unit](work_units/granular-powers.md)
- [ ] `POW-003B` Cross-system power integration tests -- [work unit](work_units/granular-powers.md)

## P1: RNG runtime parity
- [ ] `RNG-MOD-001` Central RNG module/stream authority lock
- [ ] `RNG-MOD-002` Remove direct `random.*` from parity-critical paths
- [ ] `RNG-TEST-001` Seed+action determinism regression locks -- [work unit](work_units/granular-determinism.md)

## P2: RL readiness
- [ ] `RL-ACT-001` Action mask contract lock -- [work unit](work_units/granular-actions.md)
- [ ] `RL-OBS-001` Human/debug observation profile lock -- [work unit](work_units/granular-observation.md)
- [ ] `RL-DASH-001` Local runboard + combat deep-dive dashboard
- [ ] `RL-SEARCH-001` Macro planner architecture (external to env API)
- [ ] `AUD-003` RL readiness sign-off

## See Also
- [Work units index](work_units/ACTIVE.md) -- granular implementation specs
- [Remaining gaps](remaining-work-scoped.md) -- verified gap list (3 items, all LOW)
