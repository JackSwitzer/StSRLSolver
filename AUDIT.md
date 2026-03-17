# Codebase Audit & Simplification Plan

> Draft PR for Claude Code Review. 4 parallel audits completed 2026-03-17.

## Executive Summary

- **Current**: ~145K lines Python, 6206 tests, 55 engine modules, 32 training modules
- **Target**: 58% reduction in training code (17K -> 7.2K lines), zero functionality loss
- **Immediate wins**: Delete 14 dead training files (-3500 lines), 3 unused deps, 20+ stale git branches
- **Engine**: Production-ready, minor refactoring only (observation builders, registry split)
- **Training**: Major consolidation needed (4 combat solvers -> 1, 2 encoders -> 1, overnight.py decomposition)

---

## Phase 1: Immediate Cleanup (Zero Risk)

### Delete Dead Training Modules (-3500 lines)

| File | Lines | Reason |
|------|-------|--------|
| `training/autoresearch.py` | 1014 | Legacy RL loop, replaced by overnight.py |
| `training/self_play.py` | 880 | Old AlphaZero trainer, replaced by overnight.py |
| `training/run_experiment.py` | 296 | Old experiment harness |
| `training/train.py` | 478 | Legacy REINFORCE trainer |
| `training/policy_net.py` | 463 | NumPy fallback net, never imported |
| `training/mcts.py` | 542 | Older MCTS, replaced by turn_solver.py |
| `training/benchmark.py` | 513 | Legacy evaluation harness |
| `training/episode_logger.py` | 200 | Defined but never called |
| `training/combat_state_encoder.py` | 300 | Duplicate of state_encoder_v2.py |
| `training/torch_policy_net.py` | 421 | Used only by dead modules |
| `training/combat_options.py` | 150 | Old action encoding |
| `training/param_optimizer.py` | 150 | Unused hyperparameter search |
| `training/meta_learner.py` | 282 | Unused scaffold |
| `training/strategic_features.py` | 200 | Unused research analysis |

### Delete Dead Core Data (-1600 lines)

| Path | Lines | Reason |
|------|-------|--------|
| `core/data/enemies.py` | 698 | Duplicate of packages/engine/content/enemies.py |
| `core/data/events.py` | 853 | Duplicate of packages/engine/content/events.py |

### Delete Empty/Dead Directories

- `agents/` -- empty
- `tools/` -- only `__pycache__`
- `web/static/` -- empty
- `vod/verify_ui/` -- only stale JSON

### Delete Legacy Scripts (-800 lines)

| Script | Reason |
|--------|--------|
| `scripts/start-training.sh` | Replaced by training.sh |
| `scripts/nightly-audit.sh` | Old launchd job, not in current workflow |
| `scripts/audit-setup.sh` | Old launchd installer |
| `scripts/codex-review.sh` | Replaced by /codex-review skill |
| `scripts/gpt54-review.sh` | Legacy review tool |

### Remove Unused Python Dependencies

```toml
# Remove from pyproject.toml:
"flask>=3.1.2"          # Not used (fastapi replaces)
"google-genai>=1.59.0"  # Never imported
"dearpygui>=2.1.1"      # Never imported
```

### Delete Stale Remote Git Branches

```
archive/* (8 branches) -- old archived work
codex/* (12 branches) -- analysis branches, all closed
training/v11-reward-mcts -- merged into main
viz/dashboard-v2 -- merged into main
feat/mcts-gpu-inference -- superseded
RNGDeconstructedv1 -- very old
```

---

## Phase 2: Training Architecture Consolidation

### Problem: 4 Combat Solvers

| Solver | Module | Status | Action |
|--------|--------|--------|--------|
| TurnSolver | turn_solver.py | Active (all fights) | **KEEP** |
| MultiTurnSolver | turn_solver.py | Active (boss/elite) | **KEEP** |
| CombatPlanner | combat_planner.py | Fallback, rarely reached | **DELETE** |
| GumbelMCTS | gumbel_mcts.py | Orphaned, never called | **DELETE** |
| CombatMCTS | mcts.py | Dead, old implementation | **DELETE** (Phase 1) |

**Target**: TurnSolver + MultiTurnSolver only (already in turn_solver.py).

### Problem: overnight.py is 2400 Lines

**Decomposition plan:**

| New Module | Lines | Extracted From |
|------------|-------|----------------|
| `reward_config.py` | 200 | Reward defs, hot-reload, REWARD_WEIGHTS |
| `worker.py` | 300 | `_play_one_game()` (620 lines -> cleaner interface) |
| `overnight.py` (trimmed) | 1500 | Orchestration, PPO loop, worker pool |

### Problem: 2 State Encoders

| Module | Purpose | Action |
|--------|---------|--------|
| `state_encoder_v2.py` | RunStateEncoder (260d) + CombatStateEncoder (298d) | **KEEP, rename** |
| `combat_state_encoder.py` | Duplicate encode_combat_state() | **DELETE** (Phase 1) |

### Target Training Architecture (9 core files, 7.2K lines)

```
packages/training/
  overnight.py        (1500)  Orchestrator: COLLECT -> TRAIN -> SYNC
  strategic_net.py     (236)  PyTorch model (unchanged)
  strategic_trainer.py (378)  PPO + GAE (unchanged)
  inference_server.py  (400)  MLX/Torch inference batching (unchanged)
  state_encoders.py    (420)  Run + Combat encoders (renamed)
  turn_solver.py       (850)  TurnSolver + MultiTurnSolver (unchanged)
  planner.py           (400)  StrategicPlanner heuristic fallback
  reward_config.py     (200)  Centralized reward defs + hot-reload
  worker.py            (300)  Game worker loop
  self_play.py         (100)  SeedPool only (gutted)
  mlx_inference.py     (360)  MLX model port (unchanged)
```

---

## Phase 3: Engine Optimization

### Engine is Production-Ready (Minor Changes Only)

Audit found: no dead code, no circular deps, 6206 tests passing, 100% RNG parity.

**Minor refactoring:**

| Change | Lines Saved | Effort | Risk |
|--------|------------|--------|------|
| Extract observation builders to factory (game.py) | 150 | 2-3h | Low |
| Split registry/__init__.py into contexts + executor | 0 (split) | 3-4h | Very low |

### CombatEngine.copy() Optimization

Current: ~0.5-1ms per copy. Options:
1. **Copy-on-Write undo stack** -- 50-100x faster, 3-4 days
2. **Shallow copy for search** -- 2x faster, 2 hours, medium risk
3. **`__slots__`** -- 20-30% memory, minimal effort

---

## Phase 4: Infrastructure

### RTX 3070 Integration

1. Add `--inference-server-port` to inference_server.py (gRPC instead of mp.Queue)
2. Remote workers connect via hostname:port
3. Offload boss MCTS to RTX (30ms local -> 10ms RTX + 5-10ms network)

### Combat Value Network (Missing)

No dedicated combat NN. Strategic model used as MCTS heuristic.
Plan: Small 2-block residual NN (128 hidden, 298 input), train from MCTS distillation.

### Logging Gaps

| Gap | Impact |
|-----|--------|
| Per-turn combat telemetry (damage/block/stance) | Can't correlate decisions with outcomes |
| Decision context (alternatives, policy prob, value) | Can't compute EV |

---

## Phase 5: Documentation Cleanup

### Stale Docs to Archive

- `docs/RL_TRAINING_PLAN.md` -- describes Phase 1A/1B, current is Phase 2B
- `docs/ARCHITECTURE.md` -- references old parity structure, needs update
- `docs/audits/2026-02-21-parity/` -- superseded
- `docs/audits/2026-02-22-full-game-parity/` -- superseded

### Vault Docs (Ground Truth - Keep)

All vault docs verified accurate.

---

## Review Checklist

- [ ] Delete 14 dead training modules (-3500 lines)
- [ ] Delete core/data/ duplicates (-1600 lines)
- [ ] Delete 5 legacy scripts (-800 lines)
- [ ] Remove 3 unused pyproject.toml dependencies
- [ ] Delete 20+ stale remote git branches
- [ ] Delete combat_planner.py + line_evaluator.py (superseded)
- [ ] Extract reward_config.py from overnight.py
- [ ] Extract worker.py from overnight.py
- [ ] Gut self_play.py (keep SeedPool only)
- [ ] Update docs/ARCHITECTURE.md
- [ ] Archive stale docs
- [ ] Review PPO negative loss root cause
- [ ] Profile CombatEngine.copy()
- [ ] Design RTX 3070 gRPC inference server
