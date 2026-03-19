# Weekend Training Plan: March 19-22, 2026

## Goal
Break the floor 16 ceiling. First Act 1 boss kill in training. Move from 0% win rate to measurable progress.

## Current State (as of Thursday morning)
- **255k+ total games played** across all historical runs, **0 wins**
- **Peak floor: 16** (reached ~2000 times, never cleared)
- **Root causes identified**:
  1. Per-cycle distillation was poisoning training with stale data (FIXED in PR #63)
  2. Hardcoded solver budgets ignored SOLVER_BUDGETS config (FIXED in PR #63)
  3. Ranked sweep allocation was a stub (FIXED in PR #63)
  4. Reward signal broken: PBRS ~0.001 per step vs death penalty -0.9 (model learns randomness)
  5. Zero positive terminal rewards ever seen (0% WR → no winning gradient)
- **Boss solver fix**: 3 bugs fixed (broken simulation, dead tick, scoring penalized damage). Guardian 10/10, Hexaghost 7/10 in isolation tests.
- **Rust engine**: 75/75 Watcher cards, 17 enemies, 13 relics, 14 potions, 132 tests passing

## Thursday (Today)
### Test A: Behavioral Cloning Warmup (1-2h)
- Supervised learning on F16 trajectories + Merl seed replays
- Model learns "what good players do" before PPO begins
- Metric: does floor improve faster than cold-start PPO?

### Test B: MCTS Strategic Decisions (1-2h)
- Value head evaluation at every card pick, path choice, rest, shop, event
- Model spends real compute on strategic decisions (not just 0ms inference)
- Metric: does card pick quality improve? Does floor climb?

## Weekend (Thursday night → Monday)
### Phase 1: AlphaZero Strategic MCTS (24h)
Full AlphaZero-style loop:
1. Play game with MCTS at every decision (strategic AND combat)
2. MCTS guided by value head + policy head
3. After each game: train network on (state, mcts_policy, value_target)
4. Iterate: better network → better MCTS → better games → better network

Dynamic compute:
- Simple decisions (F1 path): 100ms
- Complex decisions (card pick with rare): 10s
- Boss fights: up to 5 min per turn
- Scales by decision importance, not fixed per room type

### Phase 2: Hyperparameter Sweep (remaining time)
Based on Phase 1 results, sweep:
- Search budget (100ms → 10s → 60s per strategic decision)
- Temperature schedule (start high, anneal)
- Learning rate (1e-4 → 1e-5)
- Value/policy weight ratio
- BC warmup duration (0 → 30min → 2h)

## Key Metrics to Track
- **Avg floor** (target: 12+ by end of weekend)
- **F16+ rate** (target: 10%+)
- **F17+ rate** (target: >0 — first boss kill!)
- **Loss** (must stay positive, not negative)
- **Card pick quality** (do picks correlate with deep runs?)
- **Value head accuracy** (explained variance > 0.3)
- **Search time per decision** (are we spending compute wisely?)

## Available Resources
- M4 Mac Mini: 10 perf cores, 24GB unified memory, Metal GPU
- 12 Merl A20 winning seeds (known-solvable)
- 255k historical episodes (F16 trajectories available)
- Rust engine ready for solver speedup (separate PR pending review)
- Bug-fixed training pipeline on main (PR #63 merged)

## Success Criteria
- [ ] At least 1 game reaches floor 17 (Act 1 boss killed)
- [ ] Avg floor > 10 sustained
- [ ] Model picks Tier 1 cards (Rushdown, Tantrum, MentalFortress) >50% of offers
- [ ] Loss stays positive throughout training
- [ ] Value head explained variance > 0.3
