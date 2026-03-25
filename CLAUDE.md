# Slay the Spire RL Project

Watcher A20 bot via RL + tree search. >96% WR target.

## Structure
packages/engine/     # Python game engine (100% Java parity)
packages/training/   # RL pipeline (training_config.py = source of truth)
packages/app/        # SwiftUI macOS dashboard
packages/server/     # WebSocket server
tests/               # 6227+ tests
scripts/             # training.sh, v3_*.py runners
docs/                # vault/, research/, work_units/, reference files

## Commands
uv run pytest tests/ -q                          # all tests
uv run pytest tests/training/ -q                  # training tests
uv run pytest tests/ -k "test_name" -q            # specific test
bash scripts/training.sh start --games N          # start training
bash scripts/training.sh stop                     # always stop gracefully
bash scripts/training.sh status                   # live metrics
bash scripts/pause.sh                             # pause mid-run
bash scripts/app.sh                               # macOS dashboard

## Rules
- Config-driven: all params from training_config.py, never hardcode
- No silent excepts: log at WARNING+ or document why
- Stacked PRs: scope/description branches, merge in dependency order
- Strict scope: define goal at session start, new ideas go to TODO
- Pre-run checklist: disk (10GB+), config verify, smoke test, no stale PIDs
- Archive, never rm -rf: logs/archive/YYYY-MM-DD/
- Session end: run /reflect

## References
docs/CLAUDE-training.md  # Training architecture, nets, experiments
docs/CLAUDE-data.md      # Data curation progress + formats
docs/CLAUDE-reference.md # Engine API, card data, RNG details
docs/TODO.md             # Active work units
docs/COMPLETED.md        # Closed work units + history
