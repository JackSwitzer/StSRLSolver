---
status: reference
priority: P2
pr: null
title: "Reward Tuning"
scope: foundation
layer: engine-parity
created: 2026-02-23
completed: null
depends_on: []
assignee: claude
tags: [training, rewards, tuning]
---

# Work Unit: Reward Tuning

## Goal
Improve reward shaping to better guide learning through Act 1 and beyond. Current agent averages floor 8.5 -- needs to consistently reach and beat the Act 1 boss.

## Changes

### Act 2 Entry Bonus
- F17 milestone: 1.0 -> 3.0+ (reaching Act 2 is a major achievement)
- Consider scaling with HP: healthier arrival = better reward

### Boss HP Threshold Rewards
- +0.2 at 75% boss HP remaining
- +0.3 at 50% boss HP
- +0.5 at 25% boss HP
- Partial credit for progress in boss fights even when dying

### Death Penalty Scaling
- Reduce death penalty at boss: dying at F16 is progress, not failure
- Scale by floor: `death_penalty = -1.0 * max(0, 1 - floor/20)` (zero penalty at F20+)
- Currently: `-1.0 * (1 - floor/55)` penalizes boss deaths heavily

### Reward Audit
- Sample 1000 episodes, histogram all reward components
- Identify which rewards dominate the signal
- Ensure PBRS doesn't overwhelm event rewards or vice versa

## Dependencies
- Enhanced combat logging (for boss HP threshold data)
- `reward_config.py` is hot-reloadable -- can tune live

## Metrics
- F16+ rate (currently ~5%)
- Boss damage dealt before death
- Win rate (long-term)
