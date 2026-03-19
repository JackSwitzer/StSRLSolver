# TODO

Last updated: 2026-03-19
Baseline: 6085+ tests passing. Training v2 bug fixes merged.

## P0: Training — Weekend Experiments (2026-03-19)
- [x] `BUG-001` Kill per-cycle distillation poisoning (stale data every 100 games)
- [x] `BUG-002` Dynamic solver budgets (delete hardcoded _BUDGETS, scale by enemy HP)
- [x] `BUG-003` Fix ranked allocation (was hardcoded best_idx=0)
- [x] `BUG-004` Clean dead code (STRATEGIC_REWARDS, unused imports/handlers)
- [ ] `TRAIN-001` Learned reward signals from 255k episode analysis (data-driven card/path values)
- [ ] `TRAIN-002` Behavioral cloning warmup from Merl/F16 trajectories before PPO
- [ ] `TRAIN-003` Value head card pick evaluation (spend compute on card picks, not just combat)
- [ ] `TRAIN-004` Experiment runner for automated reward schema comparison (3-4 day unattended)
- [ ] `TRAIN-005` Fix reward signal: PBRS too small (0.001) vs death penalty (-0.9), model learns randomness

## P1: Rust Engine (separate PR)
- [x] `RUST-001` 75/75 Watcher cards defined (base + upgraded)
- [ ] `RUST-002` Fix 4 missing power triggers (Rushdown, MentalFortress, Nirvana, mantra)
- [ ] `RUST-003` Implement scry mechanic
- [ ] `RUST-004` Implement all 66 enemies with AI patterns
- [ ] `RUST-005` Combat-relevant relics
- [ ] `RUST-006` Combat potions
- [ ] `RUST-007` Fix PyO3 linking (PYO3_PYTHON env var)
- [ ] `RUST-008` Triple review: Java decompiled + Python engine + test convergence

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

## P2: Performance Optimization
- [ ] `PERF-GPU-001` Combat value network (replace CPU heuristic with GPU neural eval)
- [ ] `PERF-MEM-001` Engine state pooling (reuse copies instead of deep-copy per node)
- [ ] `PERF-MEM-002` Progressive engine view (undo stack instead of copies)
- [ ] `PERF-ASYNC-001` Async solver (continue game sim while solver computes)

## P2: App Dashboard
- [ ] `APP-001` Per-turn combat display (cards played, stance, enemy HP)
- [ ] `APP-002` Card pick offered vs chosen view
- [ ] `APP-003` Event choice tracking and display
- [ ] `APP-004` Per-floor HP history chart
- [ ] `APP-005` Experiment comparison dashboard (for unattended runs)

## See Also
- [Work units index](work_units/ACTIVE.md)
- [Remaining gaps](remaining-work-scoped.md)
