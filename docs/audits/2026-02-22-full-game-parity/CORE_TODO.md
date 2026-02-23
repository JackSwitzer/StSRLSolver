# Core TODO: Full-Game Java Parity + RL Readiness

Last updated: 2026-02-23
Execution queue: [`EXECUTION_QUEUE.md`](./EXECUTION_QUEUE.md)
Ground truth snapshot: [`GROUND_TRUTH.md`](./GROUND_TRUTH.md)

## Baseline
- Full suite baseline: `4691 passed, 5 skipped, 0 failed`
- Command: `uv run pytest tests/ -q`
- Current executed skips are replay-artifact dependent (`tests/test_parity.py`)

## Global gates
- [x] PR history synced through merged PR [#25](https://github.com/JackSwitzer/StSRLSolver/pull/25).
- [x] Regions `R1` to `R3` closed and merged to `main`.
- [ ] Canonical traceability manifest fully decomposed for remaining powers/cards/orbs gaps.
- [ ] Every parity-critical choice interaction is explicit in action dict flow.
- [ ] Normal CI path is `0 skipped, 0 failed`.
- [ ] RL readiness checklist is fully green.

## Region order (locked)
1. `R4` cards closure (non-Defect first)
2. `R5` powers + orbs closure
3. `R6` Defect cards closure
4. `R7` final re-audit + RL gate

## Region status

### R0 docs + scaffolding
- [x] `DOC-001` canonical suite + legacy archive pointer
- [x] `DOC-002` skill pack and core-loop process docs
- [x] `DOC-003` evidence refresh with inventory counts and gap queue
- [x] `DOC-004` merged-ground-truth docs pack (`GROUND_TRUTH`, `PR_HISTORY`, consolidation review)

### R1 relic selection surface
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
- [x] `RWD-003` proceed gating parity
- [x] `RWD-004` indexed secondary relic claim/gating parity

### R4 cards (non-Defect first)
- [x] `CRD-INV-002` Java-ID inventory closure slice (`Discipline`, `Impulse`, `Gash` alias coverage)
- [x] `CRD-SH-001` shared curse/status end-of-turn runtime closure (Burn/Decay/Doubt/Shame/Regret/Pride + Void draw lock)
- [ ] `CRD-IC-*` Ironclad closure
- [ ] `CRD-SI-*` Silent closure
- [ ] `CRD-WA-*` Watcher closure
- [ ] `CRD-SH-*` shared colorless/curse/status closure

### R5 powers + orbs
- [x] `CONS-001` phase-0 deterministic RNG hardening for shared effect/power/card runtime paths
- [x] `POW-001` Java power inventory closure
- [ ] `POW-002` residual hook/timing closure
- [x] `POW-003B` long-tail hook/runtime closure (`Flight`, `Malleable`, `Invincible`, `Pen Nib`, `Equilibrium`, `Echo Form` marker path)
- [x] `ORB-001` orb infrastructure for parity-critical behaviors
- [ ] `POW-003` integration tests

#### ORB-001 checklist (docs/tests/code loop)
- [x] `ORB-001A` docs+tests-first scaffolding:
  - `docs/work_units/granular-orbs.md`
  - `domains/orbs.md`
  - `traceability/gap-manifest.md`
- [x] `ORB-001B` orb runtime timing/determinism:
  - start-turn orb passives wired in combat runtime
  - owned RNG streams for random orb selection/targeting
- [x] `ORB-001C` orb-linked relic closure:
  - `Cables`, `Frozen Core`, `Emotion Chip`, `Inserter`, `Nuclear Battery`, `Symbiotic Virus`
  - remove placeholder `state.orbs`/`hasattr` branches
- [x] `ORB-001D` verification and ledger sync:
  - targeted orb/relic suites green
  - full suite green
  - baseline/docs/todo synced

### R6 cards (Defect)
- [ ] `CRD-DE-*` Defect closure

### R7 final audit + RL gate
- [ ] `AUD-001` final Java-vs-Python diff pass
- [ ] `AUD-002` zero-skip normal CI confirmation
- [ ] `AUD-003` RL readiness sign-off

## Confirmed high-impact open gaps
- [ ] Power inventory mapping is closed; hook-order/behavior parity remains open (`POW-002`, `POW-003`).
- [ ] Engine logic still contains direct Python `random` usage in parity-critical modules (relic/potion/orb long-tail after Phase-0 card/power/context hardening).

## Policy reminders
- [ ] Per feature loop: `docs -> tests -> code -> commit -> todo update`.
- [ ] One feature ID per commit.
- [ ] Domain PRs only (one region per PR).
- [ ] Every commit includes Java refs + RNG notes + test delta + skip delta.
