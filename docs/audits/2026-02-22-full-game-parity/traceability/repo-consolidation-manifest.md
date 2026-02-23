# Repository Consolidation Manifest (CONS-001)

This manifest captures explicit migration/archive decisions for content from:
- `/Users/jackswitzer/Desktop/StSRLSolver`

Canonical destination:
- `/Users/jackswitzer/Desktop/SlayTheSpireRL-worktrees/parity-core-loop`

## Migrated to canonical repo

| source | action | destination | rationale |
|---|---|---|---|
| `models/enemy_database.py` | migrated | `packages/training/enemy_database.py` | reusable deterministic encounter metadata |
| `models/line_evaluator.py` | migrated | `packages/training/line_evaluator.py` | reusable deterministic line simulation utility |
| `models/kill_calculator.py` | migrated | `packages/training/kill_calculator.py` | reusable deterministic kill-line utility |
| `models/combat_calculator.py` | migrated | `packages/training/combat_calculator.py` | reusable feature engineering utility |
| `models/strategic_features.py` | migrated | `packages/training/strategic_features.py` | reusable strategic feature extractor |
| `models/mcts.py` | migrated | `packages/training/mcts.py` | reusable MCTS utility scaffold |
| `tests/test_kill_calculator.py` | migrated | `tests/training/test_kill_calculator.py` | regression coverage for migrated kill calculator |

## Archived (not migrated)

| source | disposition | reason |
|---|---|---|
| `agent.py` | archive | CommunicationMod launch wrapper; not canonical engine/runtime |
| `watcher_agent.py` | archive | legacy wrapper agent logic tied to spirecomm runtime |
| `agent_runner.py` | archive | wrapper orchestration, superseded by canonical APIs |
| `bc_agent.py` | archive | wrapper-only BC agent surface |
| `main.py` | archive | wrapper entrypoint |
| `train_bc.py` | archive | wrapper training script coupled to legacy structure |
| `collect_self_play_data.py` | archive | wrapper data collection workflow |
| `self_play_trainer.py` | archive | wrapper-specific training loop |
| `watcher_priorities.py` | archive | wrapper strategy constants |
| `scripts/*.sh` (wrapper) | archive | environment bootstrap/launch scripts not used in canonical repo |
| `docs/PORT_SPEC.md`, `docs/STATUS.md`, `docs/TRIAL-RESULTS.md` (wrapper) | archive | stale against canonical parity docs |
| `spirecomm/` subtree | archive | third-party communication layer outside canonical engine |
| `mods/*.jar` | archive | binary runtime artifacts, not source-of-truth engine code |

## Cleanup actions (aggressive archive mode)

1. Tag/freeze non-canonical repos before deletion from Desktop workspace.
2. Keep archived histories accessible via git remotes/tags only.
3. Maintain a single active development root:
   - `/Users/jackswitzer/Desktop/SlayTheSpireRL-worktrees/parity-core-loop`

## Follow-up requirements

- Add/adjust package-level tests if migrated training modules become production dependencies.
- Keep migrated training code isolated from parity-critical engine runtime until explicitly integrated.
