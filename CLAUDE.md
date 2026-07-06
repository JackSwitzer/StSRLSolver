# Slay the Spire RL Training Rebuild

## Repo shape

Work happens on `main` plus per-unit branches (`claude/uNN-*` / `codex/uNN-*`) per
`docs/goal/UNITS.md`; see `AGENTS.md` for agent rules.

## Active System

Use the new artifact-first training stack only:

- `packages/engine-rs/`
  - canonical Rust engine and training contract
- `packages/training/`
  - snapshot corpus generation, Rust PUCT collection, MLX policy/value training, manifests, frontier logging
- `packages/app/SpireMonitor/`
  - manifest/frontier/benchmark/replay monitor

Do not reintroduce or document superseded training flows in this branch.

## Commands

```bash
./scripts/test_engine_rs.sh check --lib
./scripts/test_engine_rs.sh test --lib training_contract -- --nocapture
uv run pytest tests/training -q

./scripts/training.sh print-corpus-plan
./scripts/training.sh print-seed-suite
./scripts/training.sh launch --log-file logs/active/training-launcher.log --pid-file logs/active/training-launcher.pid run-phase1-puct-overnight --output-dir logs/active --target-cases 500 --collection-passes 3 --epochs 1
```

## Key Docs

- `AGENTS.md` + `docs/goal/GOAL.md` — conversion goal spec: end state, unit queue, oracle tooling, inventory (the /goal loop)
- `docs/CLAUDE-training.md`
- `docs/training-log-shape.md`
- `docs/work_units/combat-first-training-rebuild.md`
- `docs/work_units/watcher-seed-validation-suite.md`

## Current Phase

- Watcher A0 combat first
- artifact-first monitor output
- mixed snapshot corpus plus reconstructed Act 1 validation seeds
- MLX-only backend policy
- strategic/pathing training deferred until the combat loop is stable
