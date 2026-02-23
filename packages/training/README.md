# Training Utilities (Migrated)

This package contains curated deterministic training-side utilities migrated from the legacy `StSRLSolver` wrapper.

Scope in this package is intentionally limited to reusable pure-Python helpers. Wrapper launch code, CommunicationMod integration, and binary/runtime artifacts were not migrated.

## Modules
- `enemy_database.py`: enemy/encounter metadata tables.
- `line_evaluator.py`: deterministic combat line simulation scaffolding.
- `kill_calculator.py`: deterministic kill-line analysis on simulated states.
- `combat_calculator.py`: feature extraction for combat-value heuristics.
- `strategic_features.py`: strategic feature extraction from game-like state objects.
- `mcts.py`: generic MCTS utility with policy/value callback surface.

## Status
- Imported modules are test-covered by `tests/training/test_kill_calculator.py`.
- These modules are not yet integrated into canonical RL training loops.
- Integration into production training should be gated by parity-safe observation/action contracts.
