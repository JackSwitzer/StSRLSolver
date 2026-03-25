# Codex Task: Bug Audit

See PR description for full instructions. Key files to investigate:

- `packages/training/worker.py` — solver budget wiring, reward signals, action masking
- `packages/training/turn_solver.py` — TurnSolverAdapter budget override, neural eval
- `packages/training/training_config.py` — SOLVER_BUDGETS, MCTS_COMBAT_ENABLED, reward weights
- `packages/training/strategic_trainer.py` — PPO correctness, per-head LR, BC pretrain
- `scripts/*.py` — check for hardcoded values overriding config
- `docs/analysis/floor16_report.md` — root cause analysis of the floor 16 wall
- `docs/research/training-infra-audit-2026-03-24.md` — 9 infrastructure findings

Use subagents: one for solver/reward bugs, one for config/data quality bugs.
