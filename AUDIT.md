# Codebase Audit & Simplification Plan

> Draft PR for Claude Code Review — identify priorities, dead code, architectural improvements.

## Current State (2026-03-17)

- **6206 tests** pass, training running (v11 reward overhaul + multi-turn solver)
- Training plateau: avg floor 5-9, 0% win rate, 100% death at F16 boss
- Loss going slightly negative (-0.02) after ~2000 games — PPO instability
- Native macOS app built and running (Swift/WKWebView)

## Known Issues Requiring Review

### 1. Training Loop (overnight.py — 2400+ lines)
- **Monolithic**: game loop, reward computation, logging, hot-reload, worker management all in one file
- **Negative loss**: PPO objective goes negative, indicating value/policy head mismatch or reward scale issues
- **4 combat solvers**: TurnSolver, MultiTurnSolver, GumbelMCTS, CombatPlanner — need consolidation
- **Heuristic planner** still used as epsilon-greedy fallback — is this helping or hurting?
- **Reward shaping**: PBRS + event rewards + milestones + terminal — too many interacting signals?

### 2. Engine Complexity
- **combat_engine.py**: ~2900 lines — copy() is expensive for MCTS (1-5ms per copy)
- **game.py**: ~4000 lines — could split GameRunner from action handling
- **Registry pattern**: 168 power triggers, 172 relic triggers — how many are tested?

### 3. Dead Code Candidates
- `packages/server/` — is the WebSocket server still the right approach?
- `packages/parity/` — seed catalog tools, still needed?
- `packages/training/combat_planner.py` — superseded by MultiTurnSolver?
- `packages/training/mcts.py` — old MCTS, superseded by gumbel_mcts.py?
- `packages/training/line_evaluator.py` — used only by CombatPlanner?
- `docs/` — several docs reference old architecture

### 4. Performance Bottlenecks
- **CombatEngine.copy()**: Main bottleneck for MCTS — needs COW/undo-stack
- **RunStateEncoder**: Per-decision allocation — could cache
- **InferenceServer**: Batch size and timeout tuning
- **No profiling data**: Need per-phase timing in episodes

### 5. Missing Infrastructure
- **RTX 3070**: Not integrated into training — could run deep MCTS for boss fights
- **Combat value network**: No dedicated combat NN — strategic model used as heuristic
- **Distillation pipeline**: MCTS → combat NN training loop not implemented
- **Proper logging**: No per-solver stats, no decision context, no EV tracking

## Proposed Architecture (Target)

```
packages/
  engine/           # Pure game engine (keep, optimize copy())
  engine-rs/        # Rust engine (72 tests, 166 cards generated)
  training/
    loop.py         # Clean training loop: COLLECT → TRAIN → SYNC (replace overnight.py)
    solver.py       # Unified combat solver: MultiTurnSolver + value network
    encoder.py      # Single encoder (merge state_encoder_v2 + combat_state_encoder)
    model.py        # StrategicNet + CombatValueNet
    worker.py       # Worker process (local + remote RTX)
    distill.py      # MCTS distillation pipeline
  viz/              # React dashboard (cleaned, 16 components)
  viz/macos/        # Native Swift app
scripts/
  training.sh       # Training management
  prune_data.py     # Data lifecycle
  app.sh            # Native app build/run
```

## Review Checklist

- [ ] Identify all dead code in packages/training/ (unused imports, unreachable functions)
- [ ] Identify all dead code in packages/engine/ (unused handlers, untested paths)
- [ ] Identify stale docs that need updating or removal
- [ ] Review reward computation for correctness (PBRS + event + terminal interactions)
- [ ] Review PPO implementation for negative loss root cause
- [ ] Review combat solver hierarchy (4 solvers → 1-2)
- [ ] Review encoder architecture (2 encoders → 1?)
- [ ] Review InferenceServer architecture for RTX integration
- [ ] Review CombatEngine.copy() for optimization opportunities
- [ ] Propose file consolidation plan (current 15+ training files → 6-8)
