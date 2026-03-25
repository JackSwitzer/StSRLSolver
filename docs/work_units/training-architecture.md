---
status: active
priority: P0
pr: null
title: Training Architecture — Algorithms, Pretrain, Value Head
scope: training
layer: training-architecture
created: 2026-03-25
completed: null
depends_on: [data-pipeline, runtime-hardening]
assignee: claude
tags: [rl, architecture, algorithms, pretrain]
---

# Training Architecture

Algorithm selection, wiring, and the 5-day pretrain strategy.

## Algorithm Status

- **PPO**: Active, currently wired in training_runner.py. Experiment C (BC+PPO) achieved best results (avg floor 8.11)
- **IQL**: Needs wiring — offline-only, no collection phase. Good for leveraging existing 96k trajectories
- **GRPO**: Needs wiring — full rollout + group comparison. Promising for strategic decisions

## IQL Dispatch

- Wire IQL dispatch in `training_runner.py`
- IQL runs offline only: TRAIN phase consumes existing data, no COLLECT phase
- Requires: dataset loading from curated tier, Q-function network, advantage-weighted regression
- Config flag in `training_config.py` to select algorithm

## GRPO Dispatch

- Wire GRPO dispatch in `training_runner.py`
- GRPO: collect K rollouts from same state, rank by return, train on top group
- Requires: parallel rollout collection, group ranking, filtered policy gradient
- Config flag in `training_config.py` to select algorithm

## Solver Budget Fix (Floor 16 Wall)

- All games die at Act 1 boss (floor 16)
- Boss fights need 30s solver budget but were getting 20ms (hardcoded in scripts)
- Fix verified in PR #68 but needs runtime validation (see runtime-hardening)

## Value Head Investigation

- Unnormalized returns causing 47k pretrain loss
- Value head predictions wildly miscalibrated
- Need: return normalization, value clipping, separate value learning rate
- Investigate whether value head is helping or hurting PPO updates

## Pretrain Strategy (5-Day Run)

- Target: Wednesday night through Monday
- Phase 1 (12h): BC pretrain on curated dataset
- Phase 2 (4d): PPO fine-tuning with corrected solver budgets
- Checkpointing every 50 cycles, keep last 10 + best by val_acc
- Architecture discussion is a separate scope-locked session
- See `docs/CLAUDE-training.md` for full architecture reference
