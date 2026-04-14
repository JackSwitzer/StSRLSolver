# Slay the Spire RL

Reinforcement learning agent for Slay the Spire — targeting **Watcher Ascension 20 with >96% winrate**.

## Architecture

| Component | Location | Description |
|-----------|----------|-------------|
| **Engine** | `packages/engine/` | Pure Python game engine, 100% Java parity (RNG, damage, enemies, maps, shops, cards, relics, events) |
| **Engine (Rust)** | `packages/engine-rs/` | Rust CombatEngine + PyO3 bindings (scaffold, 63 tests) |
| **Training** | `packages/training/` | PPO + GAE pipeline with action-encoded observations, MLX inference, multi-turn combat solver |
| **Dashboard** | `packages/viz/` | React 19 + Vite live dashboard with floor curves, episode analysis |
| **Native App** | `packages/viz/macos/` | Swift/WKWebView macOS wrapper |
| **Server** | `packages/server/` | WebSocket bridge for dashboard |
| **Parity** | `packages/parity/` | Seed catalog + parity verification |

## Quick Start

```bash
# Install
uv sync

# Run tests (6100+ tests)
uv run pytest tests/ -q

# Rust engine tests
./scripts/test_engine_rs.sh --lib
./scripts/test_engine_rs.sh test --lib --no-run
./scripts/test_engine_rs.sh check --lib

# Start training
./scripts/training.sh start

# Dashboard (WebSocket + Vite)
./scripts/services.sh start    # localhost:5174

# Native macOS app
./scripts/app.sh build && ./scripts/app.sh run
```

## Engine API

```python
from packages.engine import GameRunner, GamePhase

runner = GameRunner(seed="SEED", ascension=20)
while not runner.game_over:
    actions = runner.get_available_action_dicts()
    runner.take_action_dict(actions[0])
```

## Training

- **Model**: StrategicNet (3M params, hidden=768, 4 transformer blocks)
- **Pipeline**: COLLECT 100 games -> TRAIN PPO epochs -> SYNC -> repeat
- **Inference**: Centralized MLX batch server (M4 Mac Mini, 10 cores)
- **Observations**: 260-dim state + 512-dim action encoding (available actions mask)
- **Combat**: TurnSolver (30ms) with multi-turn lookahead
- **Rewards**: Floor milestones, combat outcomes, HP preservation, PBRS shaping (hot-reloadable)

## Status

- **Engine parity**: 100% across all core mechanics (13 RNG streams, 66 enemies, 51 events, 168 powers, 172 relics)
- **Tests**: 6100+ passing (pytest) + 63 Rust
- **Training**: Active — PPO with action encoding, mixed exploit/explore temperature
- **Best trajectories**: Floor 16 (200 saved), iterating toward Act 2+

## References

- [bottled_ai](https://github.com/xaved88/bottled_ai) — 52% Watcher A0 baseline
- [CommunicationMod](https://github.com/ForgottenArbiter/CommunicationMod) — Bot communication protocol
- [StSRLSolver](https://github.com/JackSwitzer/StSRLSolver) — Prior RL solver attempt
