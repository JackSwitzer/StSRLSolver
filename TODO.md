# TODO

Last updated: 2026-03-18
Baseline: 6100+ tests passing, training active with action-encoded observations.

## P1: Card behavior parity
- [ ] `CRD-IC-001` Ironclad behavior closure
- [ ] `CRD-SI-001` Silent behavior closure
- [ ] `CRD-WA-001` Watcher behavior closure
- [ ] `CRD-SH-002` Shared colorless/curse/status closure
- [ ] `CRD-DE-001` Defect behavior closure

## P1: Powers behavior/order parity
- [ ] `POW-002B` Hook ordering and trigger count lock
- [ ] `POW-003A` Behavior closure by hook family
- [ ] `POW-003B` Cross-system power integration tests

## P1: RNG runtime parity
- [ ] `RNG-MOD-001` Central RNG module/stream authority lock
- [ ] `RNG-MOD-002` Remove direct `random.*` from parity-critical paths
- [ ] `RNG-TEST-001` Seed+action determinism regression locks

## P2: RL readiness
- [ ] `RL-ACT-001` Action mask contract lock (ordered legal list + stable IDs)
- [ ] `RL-OBS-001` Human/debug observation profile lock
- [ ] `RL-DASH-001` Local runboard + combat deep-dive dashboard
- [ ] `RL-SEARCH-001` Macro planner architecture (external to env API)
- [ ] `AUD-003` RL readiness sign-off
