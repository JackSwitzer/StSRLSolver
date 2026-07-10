---
name: pretrain
description: Launch BC pretrain with safety checks — verify config, disk, dataset, smoke test
user-invocable: true
---

# Pretrain Launch

Safe pretrain launch with pre-flight checks and proper logging.

## Process

### Phase 1: Pre-flight Checks
Run these sequentially — abort on any failure:

1. **Disk space**: `df -h .` — need 10GB+ free. Abort if less.
2. **Stale PIDs**: `bash scripts/training.sh status` — abort if training already running.
3. **Config verify**: Read `packages/training/training_config.py` — confirm ALGORITHM, LR_BASE, MODEL_HIDDEN_DIM are expected values. Check no stale sweep overrides.
4. **Dataset check**: Count trajectory files. Report dimensions, quality estimate.

### Phase 2: Dataset Selection
Ask user which dataset tier to use:
- **all**: Everything in logs/ (96k+ trajectories)
- **filtered**: Floor 10+ trajectories only
- **curated**: Manually selected high-quality runs
- **custom**: User specifies path

### Phase 3: Smoke Test
Run 10 games with current config:
```bash
uv run python scripts/v3_1h_test.py --games 10 --timeout 300
```
Check: games complete, avg floor > 5, no crashes.

### Phase 4: Launch
```bash
caffeinate -i uv run python scripts/v3_pretrain_gpu.py --dataset <selected> 2>&1 | tee logs/pretrain_$(date +%Y%m%d_%H%M).log &
```
Report PID and log location.
