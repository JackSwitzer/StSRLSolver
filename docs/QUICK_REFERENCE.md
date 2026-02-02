# Quick Reference - SlayTheSpireRL

## Most Common Commands

### Launch Game

```bash
# Standard (with CommunicationMod)
./scripts/dev/launch_sts.sh

# Vanilla (no mods)
./scripts/dev/launch_sts.sh --no-mods

# Foreground (see output)
./scripts/dev/launch_sts.sh --fg
```

### Check Save File

```bash
# Read current WATCHER save
./scripts/dev/save.sh

# Read other characters
./scripts/dev/save.sh IRONCLAD
./scripts/dev/save.sh SILENT
./scripts/dev/save.sh DEFECT
```

### RNG Parity Testing

```bash
# Compare predictions with current save
./scripts/dev/parity.sh

# Generate predictions for specific seed
uv run scripts/dev/test_parity.py --seed "1234567890"
```

### Live Dashboards

```bash
# Web dashboard (open http://localhost:8080)
uv run python web/server.py

# GUI viewer (DearPyGui)
uv run tools/game_viewer.py
```

### Interactive Engine

```bash
# Test Python engine with seed
uv run tools/test_engine.py --seed 1234567890

# With engine.sh wrapper
./scripts/dev/engine.sh 1234567890
```

---

## Quick Comparison Table

| I want to... | Command |
|-------------|---------|
| Launch STS with mods | `./scripts/dev/launch_sts.sh` |
| Launch STS vanilla | `./scripts/dev/launch_sts.sh --no-mods` |
| Read save file | `./scripts/dev/save.sh` |
| Check RNG parity | `./scripts/dev/parity.sh` |
| Start web dashboard | `uv run python web/server.py` |
| Start GUI viewer | `uv run tools/game_viewer.py` |
| Test engine interactively | `uv run tools/test_engine.py --seed SEED` |
| Run batch simulations | `uv run tools/launcher.py` |

---

## File Locations

```
scripts/dev/
  parity.sh          # RNG parity wrapper
  test_parity.py     # Parity testing logic
  engine.sh          # Engine tester wrapper
  launch_sts.sh      # Game launcher
  read_save.py       # Save file reader
  save.sh            # Save reader wrapper

tools/
  game_viewer.py     # DearPyGui viewer
  launcher.py        # Simulation launcher
  test_engine.py     # Interactive engine

web/
  server.py          # FastAPI dashboard
  index.html         # STS Oracle UI
```

---

## Seed Formats

The game accepts two seed formats:

```bash
# Numeric (long)
--seed 1234567890

# Alphanumeric
--seed "ABCDEF"
```

Both are interchangeable and converted internally.

---

## Key URLs

| Service | URL |
|---------|-----|
| Web Dashboard | http://localhost:8080 |
| API State | http://localhost:8080/api/state |
| API Stream (SSE) | http://localhost:8080/api/stream |
