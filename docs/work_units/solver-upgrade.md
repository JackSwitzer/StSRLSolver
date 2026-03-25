---
status: reference
priority: P2
pr: null
title: "Solver Upgrade"
scope: foundation
layer: engine-parity
created: 2026-02-23
completed: null
depends_on: []
assignee: claude
tags: [training, solver, performance]
---

# Work Unit: Solver Upgrade

## Goal
Dynamic search budget that scales with fight importance. Currently all fights get the same 50ms/5000 node budget.

## Changes

### Dynamic Budget by Room Type
- **Boss**: 30s budget, 5-turn multi-turn lookahead (up from 3)
- **Elite**: 5s budget (up from 100ms)
- **Monster**: 1s budget (up from 10ms/500 nodes)
- Node budget scales proportionally: 100 nodes/ms

### Implementation
- `worker.py`: Pass room_type to TurnSolverAdapter, select budget from config
- `turn_solver.py`: Accept dynamic time/node budgets per call
- `reward_config.py`: Add SOLVER_BUDGETS dict (hot-reloadable)

### Worker Count
- 10 workers (one per performance core on M4 Mac Mini)
- Solver gets longer to think -> fewer games/min but higher quality decisions

### Convergence Criterion
- Expand search until top-3 actions stable for 3 consecutive expansions
- Hard cap at budget limit for safety

## Dependencies
- None (isolated to solver + worker)

## Metrics
- Win rate improvement
- Games/min (will decrease -- expected)
- Solver time distribution by room type
