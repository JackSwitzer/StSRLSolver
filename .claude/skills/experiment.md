---
name: experiment
description: Design and launch a training experiment with named config, success metrics, and auto-compare
user-invocable: true
---

# Experiment Runner

Design, configure, and launch a training experiment with proper tracking.

## Usage
`/experiment <name>` — e.g., `/experiment boss-budget-30s`

## Process

### Phase 1: Experiment Design
Ask user for:
1. **Name**: Short identifier (becomes directory name in logs/)
2. **Hypothesis**: What are we testing?
3. **Config overrides**: Which training_config params to change
4. **Success metrics**: What defines success? (avg floor, win rate, clip fraction, etc.)
5. **Duration**: How long to run (games or hours)
6. **Baseline**: What to compare against (last run? specific experiment?)

### Phase 2: Setup
1. Create experiment directory: `logs/experiments/<name>/`
2. Write experiment config to `logs/experiments/<name>/config.json`:
   - All overrides
   - Baseline reference
   - Success criteria
   - Start time
3. Pre-flight: disk check, stale PID check, config verify

### Phase 3: Launch
```bash
caffeinate -i uv run python scripts/v3_experiment_sweep.py \
  --name <name> \
  --config <overrides> \
  --games <N> \
  2>&1 | tee logs/experiments/<name>/run.log &
```

### Phase 4: Monitor
Provide commands for checking progress:
```bash
bash scripts/training.sh status
cat logs/experiments/<name>/status.json
```

### Phase 5: Compare (after completion)
Read status.json from experiment and baseline. Compare metrics.
Report: did experiment meet success criteria?
