---
status: active
priority: P0
pr: null
title: Tooling — Hooks, Skills, Automation
scope: visibility
layer: tooling
created: 2026-03-25
completed: null
depends_on: []
assignee: claude
tags: [hooks, skills, automation, session]
---

# Tooling

Claude Code hooks, skills, and automation for safer and faster development.

## Claude Code Hooks (Safety Guards)

### Pre-Push Hook
- Verify tests pass before any push
- Block push if test failures detected
- Quick smoke test subset for speed (<30s)

### Pre-Training Hook
- Disk check: require >10GB free before starting training
- No stale PIDs: kill or warn about orphaned worker/inference processes
- Config consistency: verify training_config.py matches any script overrides
- Model checkpoint exists and dimensions match config

## Skills to Create

### /pretrain
- Config verification (architecture, hyperparams, paths)
- Disk space check
- Dataset validation (dimensions, quality, size)
- Smoke test (1 game collection + 1 train step)
- Launch training with logging

### /experiment
- Name the experiment
- Configure hyperparams (diff from baseline)
- Launch with isolated logging directory
- Auto-compare results against baseline when complete

### /training-status
- Disk usage and free space
- GPU utilization (Metal)
- Current metrics: loss, val_acc, avg floor, win rate
- Anomaly detection: flag unusual patterns
- Worker health: which workers are alive, any crashes

### /data-audit
- Scan all data directories
- Check quality: dimension consistency, action mask validity, reward sanity
- Recommend curation actions (delete, archive, promote to higher tier)
- Summary statistics and report

## PR Auto-Update

- On merge, regenerate `TODO.md` and `COMPLETED.md` from work unit YAML frontmatter
- Parse `status`, `priority`, `completed` fields from all work unit files
- Keep docs in sync with actual project state automatically

## Session Discipline

- Strict scope lock: each session works on one work unit only
- /reflect at end of every meaningful session
- Update work unit status in YAML frontmatter when work completes
- No scope creep: if new work is discovered, create a new work unit, don't expand current one

## Stacked PR Workflow

- Branch naming: `scope/description` (e.g. `foundation/data-pipeline`)
- Merge in dependency order (respect `depends_on` in YAML)
- Each PR maps to one work unit
- Rebase stack when base PR merges
