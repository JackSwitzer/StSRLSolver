# TODO

Last updated: 2026-03-25
Status: Main clean with all v3 work merged. Floor 16 wall is primary blocker.

## P0: Floor 16 Blocker (Act 1 Boss)
- [ ] Root cause: solver budget too low for boss fights, model doesn't learn boss strategy
- [ ] Increase boss solver budget (dynamic, based on boss HP)
- [ ] Investigate value head quality at boss floor states
- [ ] Consider boss-specific training curriculum

## P1: Training Pipeline Improvements
- [ ] Normalize value targets in pretrain (currently 47k loss due to unnormalized returns)
- [ ] Wire GRPO full rollout collection (currently falls back to PPO)
- [ ] Fix EndTurn-with-playable-cards issue (solver ends turns with 3 energy + 5 playable)
- [ ] Trajectory dimension filter (older trajectories incompatible, ~1k/18k filtered)
- [ ] Scale BC pretrain to full 96k trajectory dataset (620k transitions)

## P1: Card Behavior Parity
- [ ] `CRD-IC-001` Ironclad behavior closure -- [work unit](work_units/granular-cards-ironclad.md)
- [ ] `CRD-SI-001` Silent behavior closure -- [work unit](work_units/granular-cards-silent.md)
- [ ] `CRD-WA-001` Watcher behavior closure -- [work unit](work_units/granular-cards-watcher.md)
- [ ] `CRD-SH-002` Shared colorless/curse/status closure -- [work unit](work_units/granular-cards-shared.md)
- [ ] `CRD-DE-001` Defect behavior closure -- [work unit](work_units/granular-cards-defect.md)

## P1: Powers Behavior/Order Parity
- [ ] `POW-002B` Hook ordering and trigger count lock -- [work unit](work_units/granular-powers.md)
- [ ] `POW-003A` Behavior closure by hook family -- [work unit](work_units/granular-powers.md)
- [ ] `POW-003B` Cross-system power integration tests -- [work unit](work_units/granular-powers.md)

## P2: Dashboard Upgrades
- [ ] Per-turn combat detail in app views
- [ ] Card picks: offered vs chosen visualization
- [ ] Floor-by-floor HP trajectory chart

## P2: Infrastructure
- [ ] Rust combat engine integration for faster MCTS sims
- [ ] Checkpoint curation strategy (what to keep on GitHub Releases)
- [ ] Auto-pause on low disk space (< 5GB)

## Completed
See [COMPLETED.md](COMPLETED.md) for full history of completed work with dates and PR references.
