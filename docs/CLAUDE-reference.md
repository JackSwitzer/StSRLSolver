# Branch Reference

This file is the quick reference for the stacked training rebuild branch.

## Canonical Commands

```bash
./scripts/test_engine_rs.sh check --lib
./scripts/test_engine_rs.sh test --lib training_contract -- --nocapture
uv run pytest tests/training -q

./scripts/training.sh print-corpus-plan
./scripts/training.sh print-seed-suite
./scripts/training.sh run-phase1-overnight --output-dir logs/active --epochs 1 --target-requests 24 --backend linear
```

## Canonical Paths

- Rust contract:
  - `packages/engine-rs/src/training_contract.rs`
- training runtime:
  - `packages/training/`
- monitor app:
  - `packages/app/SpireMonitor/`
- active branch docs:
  - `docs/CLAUDE-training.md`
  - `docs/work_units/combat-first-training-rebuild.md`
  - `docs/work_units/watcher-seed-validation-suite.md`

## Active Artifacts

- `manifest.json`
- `events.jsonl`
- `metrics.jsonl`
- `frontier_report.json`
- `frontier_groups.json`
- `benchmark_report.json`
- `episodes.jsonl`
- `dataset.jsonl`
- `checkpoint.json`
- `summary.json`

## Current Scope

- Watcher A0 combat only
- corpus-driven search + reanalysis
- external validation seeds
- artifact-first monitoring

Strategic/pathing learning comes later.
