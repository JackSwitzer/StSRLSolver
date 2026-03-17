# Code Review Guidelines

## Focus Areas (all PRs in this cleanup series)

### 1. Broken Imports (Critical)
- Verify every deleted file is not imported by any remaining code
- Check both direct imports and dynamic imports (importlib, __import__)
- Verify test files don't reference deleted modules

### 2. Dead Code Discovery
- Flag any additional unused functions, classes, or modules not in AUDIT.md
- Check for unreachable code paths after deletions
- Identify any orphaned test files testing deleted code

### 3. Architecture
- Validate module boundaries after overnight.py decomposition
- Check circular dependencies in new module structure
- Verify extracted modules have clean interfaces

### 4. Training Correctness
- Review PBRS reward computation for mathematical correctness
- Check PPO implementation (negative loss root cause)
- Verify MultiTurnSolver correctness (enemy turn simulation, depth recursion)
- Check that removing epsilon-greedy doesn't break exploration

## Skip
- Auto-generated files (cards_gen.rs)
- Lock files, node_modules, .venv
- Test data files (*.jsonl, *.npz)
- Engine content files (cards.py, relics.py, enemies.py) — verified via 6206 tests
