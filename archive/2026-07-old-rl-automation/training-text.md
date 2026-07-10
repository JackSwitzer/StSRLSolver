---
name: training-text
description: Send dashboard-style training status via iMessage with thresholds, death analysis, and milestone detection
user-invocable: true
---

# Training Status Text

Send a dashboard-style iMessage with current training metrics. Min 10 games before sending.

## Process

### Phase 1: Gather Data

Run `bash scripts/training.sh text --force` to check if the bash version works. If it fails or you want richer analysis (milestone detection, trend comparison), gather manually:

1. Find latest run: `ls -td logs/runs/run_* | head -1`
2. Read status.json with jq for all metrics
3. Read last 50 episodes for death analysis
4. Check disk, PID, GPU

**Skip if < 10 games** (no meaningful data yet).

### Phase 2: Format Dashboard Message

Use this exact template with inline threshold annotations:

```
=== StS Training ({elapsed}h) ===

PROGRESS
  Games: {games} ({collect_progress}) sweep {n}/{total}
  Floor: {avg_floor} avg | {peak} peak
  Wins: {wins}

BOSS WALL
  {top_killer}
  Boss: {kills}/{attempts} kills | Early deaths: {early}

HEALTH
  Entropy: {entropy} ({threshold: ok >0.5, low <0.5, COLLAPSED <0.02})
  Value loss: {val_loss} ({threshold: ok <2.0, elevated <5.0, high >5.0})
  Policy: {policy_loss}
  Throughput: {gpm} g/min

SYSTEM
  GPU {gpu}% | {disk}GB free ({ok/LOW}) | {alive/DEAD}
  Config: {config_name}
```

### Phase 3: Milestone Detection

Read `/tmp/sts_last_text.json`. Prepend if triggered:
- `NEW PEAK F{n}!` — peak > last peak
- `FIRST WIN!` — wins > 0, was 0
- `SWEEP {n} STARTING` — sweep changed
- `{n}K GAMES` — crossed 1000 boundary
- `ALERT: {issue}` — PID dead, disk < 5GB, entropy < 0.02

### Phase 4: Send

```bash
osascript -e "tell application \"Messages\"
  set s to 1st account whose service type = iMessage
  send \"$MSG\" to participant \"+14166293183\" of s
end tell"
```

### Phase 5: Save State

Write `{peak, wins, sweep, games}` to `/tmp/sts_last_text.json`.

## Usage

- `/training-text` — send one status text now
- `/loop 2h /training-text` — send every 2 hours (weekend monitoring)
- `bash scripts/training.sh text` — bash-only version (no Claude needed)
- `bash scripts/training.sh text --loop 2h` — background bash loop
