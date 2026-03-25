---
name: training-status
description: Deep training status check — disk, metrics, anomaly detection, recommendations
user-invocable: true
---

# Training Status

Comprehensive health check of running or recent training.

## Process

### Phase 1: System Health (parallel checks)

**Disk**: `df -h .` — warn if < 10GB, critical if < 5GB
**PIDs**: `bash scripts/training.sh status` — running processes
**GPU**: Check Metal GPU utilization if available
**Logs**: `ls -lt logs/active/` — most recent log files

### Phase 2: Training Metrics

Read `logs/active/status.json` (or latest experiment status):
- Games completed / total
- Average floor (trend: improving?)
- Win rate
- Clip fraction (> 0.30 = bad)
- Value loss (> 5.0 = bad)
- Entropy (< 0.01 = collapsed)
- Policy loss trend
- Throughput (games/min)

### Phase 3: Anomaly Detection

Flag if:
- Clip fraction > 0.30 for > 100 games
- Entropy < 0.02 (policy collapsing)
- Value loss increasing for > 50 games
- Avg floor decreasing for > 200 games
- Disk < 5GB remaining
- Any worker crashed (PID missing)
- Throughput dropped > 50% from baseline

### Phase 4: Report

Present concise status:
```
Training: RUNNING | 1,234 / 10,000 games | 12.4 games/min
Floor: 8.9 avg (↑ from 7.2) | Peak: F16
Metrics: clip=0.18 ✓ | entropy=0.04 ✓ | val_loss=2.3 ✓
Disk: 14GB free ✓
Anomalies: none
```

If anomalies detected, recommend specific actions.
