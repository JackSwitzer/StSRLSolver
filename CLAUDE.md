# Slay the Spire RL Project

## Project Goal
Build a mod/bot that wins Slay the Spire (Watcher only, A20, >96% winrate) using reinforcement learning with EV-based decision tracking.

## Project Structure (Monorepo)
```
packages/engine/     # Pure Python game engine (source of truth)
packages/engine-rs/  # Rust CombatEngine + PyO3 bindings
packages/training/   # RL training pipeline
  overnight.py       # Training orchestrator (COLLECT -> TRAIN -> SYNC)
  worker.py          # Game worker loop (_play_one_game)
  reward_config.py   # REWARD_WEIGHTS, PBRS, hot-reload
  strategic_net.py   # StrategicNet (PyTorch, 3M params)
  strategic_trainer.py # PPO + GAE trainer
  inference_server.py # Centralized MLX/Torch batch inference
  state_encoders.py  # RunStateEncoder + CombatStateEncoder
  turn_solver.py     # TurnSolver + MultiTurnSolver
  replay_buffer.py   # Trajectory replay
  sweep_config.py    # Sweep templates, ascension breakpoints
  seed_pool.py       # Seed management
  mlx_inference.py   # MLX model port
  episode_log.py     # JSONL logging
  combat_calculator.py # Damage/block prediction (407 lines)
  conquerer.py       # SeedConquerer multi-path evaluation (443 lines)
  gym_env.py         # Gymnasium wrapper (future use)
packages/server/     # WebSocket server for dashboard
packages/viz/        # React 19 + Vite dashboard
packages/viz/macos/  # Native Swift/WKWebView macOS app
packages/parity/     # Seed catalog + parity verification
packages/tauri/      # Tauri desktop app (Rust, wraps viz)
tests/               # 6100+ tests (pytest)
scripts/             # Shell scripts for training, services, app
  training.sh        # Main training manager
  services.sh        # WS + Vite process manager
  hotfix.sh          # Live parameter tuning (SIGUSR1 + reload.json)
  app.sh             # Native macOS app builder
  play.sh            # Manual game player
  utils/             # Python utilities (prune, generators, enrichment)
docs/                # TODO, vault (ground truth), work units, research
```

## Testing
```bash
uv run pytest tests/ -q              # Run all tests
uv run pytest tests/test_parity.py   # Parity verification
uv run pytest tests/ --cov=packages/engine  # Coverage (~76%)
export PATH="$HOME/.cargo/bin:$PATH" PYO3_PYTHON=.venv/bin/python3
cargo test --lib --manifest-path packages/engine-rs/Cargo.toml  # 63 Rust tests
```

## Java Parity Status (Last Updated 2026-03-11)

### Core Mechanics (100% Parity)
| Domain | Status | Notes |
|--------|--------|-------|
| **RNG System** | 100% | All 13 streams verified, production ready |
| **Damage Calculation** | 100% | Vuln 1.5x, Weak 0.75x, floor operations exact |
| **Block Calculation** | 100% | Dex before Frail, floor operations exact |
| **Stances** | 100% | Wrath/Calm/Divinity/Neutral verified |
| **Enemies** | 100% | All 66 enemies verified |
| **Map Generation** | 100% | Java quirks included |
| **Shop** | 100% | Prices and pools match |
| **Card Rewards** | 100% | HashMap order + pity timer |
| **Card Data** | 100% | All characters verified |
| **Potions (Data)** | 100% | All 42 potions correct |

### Implementation Coverage (Updated 2026-03-11)
- **Power triggers**: 168 registered (136 unique power IDs). ~10 non-Watcher powers still missing (ExplosivePower, StasisPower, ConservePower, etc.).
- **Relic triggers**: 172 registered across registry files. Boss relic energy wired. Most pickup/onEquip effects implemented.
- **Events**: 51/51 handlers implemented; 48/51 choice generators (3 use aliases). All critical events verified.
- **Potions (effects)**: Core effects implemented; discovery/selection partial.
- **Rewards**: JSON action layer fully implemented.

### Remaining Gaps (~10 items, all LOW/MEDIUM, see `docs/remaining-work-scoped.md`)
| Category | Count | Priority | Description |
|----------|-------|----------|-------------|
| DrawPower passive | 1 | MEDIUM | Verify draw modification works end-to-end |
| applyStartOfTurnCards | 1 | LOW | Per-turn card cost reset (partial impl exists) |
| Non-Watcher powers | ~7 | LOW | Defect/Silent-only powers (ExplosivePower, StasisPower, etc.) |
| Relic edge cases | 2-3 | LOW | N'loth's Mask, Prismatic Shard card rewards |

See test files and `docs/remaining-work-scoped.md` for details.

## Engine API (for RL integration)
```python
from packages.engine import GameRunner, GamePhase

runner = GameRunner(seed="SEED", ascension=20)
while not runner.game_over:
    actions = runner.get_available_action_dicts()  # JSON actions
    runner.take_action_dict(actions[0])            # Execute action
    # runner.run_state = full observable state
    # runner.phase = current GamePhase
```

## Git Branches
- **main**: Clean monorepo (engine + tests + parity + training + viz)
- **archive/pre-cleanup** (tag): Full project snapshot pre-cleanup
- **archive/sts-oracle**: Web dashboard
- **archive/vod-extraction**: VOD/training pipeline
- **archive/comparison-tools**: Verification scripts
- **archive/old-master**: Original StSRLSolver repo
- External archives: `~/Desktop/sts-archive/` (decompiled/, logs/, java deps, server-legacy/)

## Workflow

### Subagent Policy
- **Opus**: ALL tasks with main context that report back to main loop
- **Haiku**: ONLY for quick sub-sub-agent searches within Opus agents
- Never use Haiku directly from main loop

### Session End Workflow
- Run `/update-claude` at end of every session
- Updates this CLAUDE.md as ground truth alongside vault docs
- Vault (docs/vault/) = game mechanics ground truth
- CLAUDE.md = project decisions + learnings ground truth

### Subagent Templates

**Decompile Search Agent** (decompiled/ archived to ~/Desktop/sts-archive/):
```
Investigate [TOPIC] in the archived Slay the Spire source.

Location: ~/Desktop/sts-archive/decompiled/java-src/com/megacrit/cardcrawl/
```

## Resource Model

All resources affect probability of winning. Track EV impact of each.

### Primary Resources
| Resource | Type | Notes |
|----------|------|-------|
| **HP** | Fungible | Most important currency. Replenishes at rest, some events, Reaper |
| **Energy** | Per-turn | 3 base, Calm exit +2, Divinity +3, relics modify |
| **Potions** | Consumable | Limited slots, drop from elites/events, powerful effects |
| **Gold** | Persistent | Shops, some events, relic removal |

### Card Resources
| Resource | Behavior | Notes |
|----------|----------|-------|
| **Exhaustable cards** | One-use/combat | Track when worth exhausting |
| **Draw pile** | Blind during combat | But known composition - sim possible |
| **Discard pile** | Visible | Returns on shuffle |
| **Exhaust pile** | Visible | Gone unless specific retrieval |

### Strategic Resources
| Resource | Impact |
|----------|--------|
| **Relics** | Affect probability distributions, permanent effects |
| **Deck composition** | Defines strategy space |
| **Upgrades** | Permanent improvements |
| **Card removes** | Deck thinning |

## Seed Visibility Model

Agent is NOT blind to seed:
- Can see upcoming events (with RNG knowledge)
- Can see how relics impact randomness
- Can simulate forward (but only so far - complexity explodes)
- Draw pile blind during combat but can sim given deck state

For training:
- Sim bare games for more accurate Watcher models
- Distillation from full-info to partial-info agent

## Core Features (Priority Order)

1. **EV Tracking** - Track all decisions and outcomes for win/Act 1 boss correlation
2. **Resource Tracking** - Full resource model above
3. **Damage Calculation** - See docs/vault/damage-mechanics.md
4. **Infinite Detection** - Fading death animation + console kill for infinite combos
5. **Stats Logging** - Win/loss, all decisions, resource states per floor

## Reference Projects

- https://github.com/JackSwitzer/StSRLSolver - Existing RL solver (see vault/stsrlsolver-analysis.md)
- https://github.com/alexdriedger/SlayTheSpireFightPredictor - ML fight prediction (+/-7 HP)
- https://github.com/I3eyonder/StS-DamageCalculator - Damage calc (new Jan 2026, contact for collab)
- https://github.com/ForgottenArbiter/CommunicationMod - Bot communication protocol
- https://github.com/xaved88/bottled_ai - 52% Watcher A0 (graph traversal baseline)

## Architecture

### Mega-Mod Components
1. **Damage Calculator** - Accurate prediction with all modifiers
2. **EV Tracker** - Decision logging with outcome correlation
3. **Infinite Detector** - Fading death + console kill safety
4. **Resource Logger** - Full state serialization

### Build
- Java 8, Maven
- Deps: ModTheSpire, BaseMod, StSLib
- See docs/vault/modding-infrastructure.md

### Launch
```bash
cd "/Users/jackswitzer/Library/Application Support/Steam/steamapps/common/SlayTheSpire/SlayTheSpire.app/Contents/Resources"
./jre/bin/java -Xmx1G -jar ModTheSpire.jar --skip-launcher --skip-intro --mods basemod,CommunicationMod
```

## Vault Docs

Ground truth for game mechanics:
- `docs/vault/damage-mechanics.md` - Full damage/block formulas
- `docs/vault/modding-infrastructure.md` - Patching, hooks, build
- `docs/research/rl-methodology-survey.md` - Existing approaches
- `docs/vault/direct-launch.md` - Running without Steam
- `docs/research/stsrlsolver-analysis.md` - Existing project structure
- `docs/vault/rng-system-analysis.md` - Complete 13-stream RNG analysis

## Engine Registry Pattern

Effect handlers use decorator-based registration:
```python
# packages/engine/registry/
@relic_trigger("atBattleStart", relic="Vajra")
def vajra_start(ctx: RelicContext) -> None:
    ctx.apply_power_to_player("Strength", 1)

@power_trigger("atDamageGive", power="Strength")
def strength_damage_give(ctx: PowerContext) -> int:
    return ctx.trigger_data.get("value", 0) + ctx.amount

@potion_effect("Fire Potion", requires_target=True)
def fire_potion(ctx: PotionContext) -> None:
    ctx.deal_damage_to_target(ctx.potency)
```

Trigger hooks match Java: `atBattleStart`, `onPlayCard`, `wasHPLost`, `atDamageGive`, `atDamageReceive`, etc.

## EV Tracking Framework

### Decision Points to Track
1. **Card plays** - Which card, target, stance, resources before/after
2. **Card picks** - Which card chosen, alternatives, deck state
3. **Path choices** - Node type, HP state, deck readiness
4. **Rest decisions** - Rest vs upgrade vs lift vs dig
5. **Shop decisions** - Buy/remove/skip
6. **Event choices** - Option taken, outcome
7. **Potion use** - When, on what, alternatives

### Outcome Attribution
- HP change per decision
- Did we win floor? Act? Game?
- Cards played to kill ratio
- Damage taken vs incoming (block efficiency)

### EV Calculation
```
EV(decision) = P(win | decision) - P(win | baseline)
```

Baseline options:
- Random valid action
- Heuristic agent action
- Model's second-best action

## Watcher-Specific

### Stance Priority
1. Exit Wrath if enemies attacking and can't lethal
2. Enter Wrath if can lethal or enemies not attacking
3. Calm for energy generation on safe turns
4. Divinity (10 Mantra) for burst

### Key Cards
Tier 1: Rushdown, Tantrum, Ragnarok, MentalFortress, TalkToTheHand
Tier 2: InnerPeace, CutThroughFate, WheelKick, Conclude

### Energy Math
- Base: 3
- Calm exit: +2 (Violet Lotus: +3)
- Divinity enter: +3
- Deva Form: +1/turn stacking

### Scry Mechanics
- Scry X: Look at top X cards of draw pile, choose which to DISCARD (rest stay on top)
- Golden Eye relic: +2 to ALL scry amounts
- Melange relic: Scry 3 on shuffle
- Nirvana power: Gain block once per scry ACTION (not per card)
- Agent decides which cards to discard via `SelectScryDiscard` action

## RNG Prediction System (Verified 100% Java Parity)

### Documentation
- `docs/vault/rng-system-analysis.md` - Complete 13-stream analysis
- `docs/vault/verified-seeds.md` - Verified seed data and Neow cardRng consumption
- `packages/engine/state/game_rng.py` - **GameRNGState** implementation (counter-based state machine)

### Key Implementation: GameRNGState
```python
from packages.engine.state.game_rng import GameRNGState, simulate_path, predict_card_reward

# Simulate a path through the game
path = [
    ('neow', 'HUNDRED_GOLD'),
    ('floor', 1),
    ('combat', 'monster'),
    ('shop', None),  # Consumes ~12 cardRng calls
    ('floor', 2),
]
state = simulate_path('SEED', path)
cards = predict_card_reward(state)
```

### Critical Discoveries

**1. Act Transition cardRng Snapping:**
```
counter 1-249   -> snaps to 250
counter 251-499 -> snaps to 500
counter 501-749 -> snaps to 750
```

**2. cardRng Consumption by Room Type:**
| Room | cardRng Calls | Notes |
|------|---------------|-------|
| Combat | ~9 | 3 rarity + 3 select + 3 upgrade |
| Shop | ~12 | 5 colored + 2 colorless + retries |
| TheLibrary event | ~20 | Generates 20 unique cards |
| Most events | 0 | Use miscRng instead |
| Treasure | 0 | Uses treasureRng |

**3. Safe Neow Options (no cardRng):**
UPGRADE_CARD, HUNDRED_GOLD, TEN_PERCENT_HP_BONUS, RANDOM_COMMON_RELIC, THREE_ENEMY_KILL, THREE_CARDS, ONE_RANDOM_RARE_CARD, TRANSFORM_CARD, REMOVE_CARD

**4. cardRng Consumers:**
- RANDOM_COLORLESS: 3+ calls
- CURSE drawback: 1 call
- Calling Bell boss swap: ~3 calls
- Shops: ~12 calls

### RNG Stream Inventory (13 Total)

**Persistent (survive entire run):**
| Stream | Used For |
|--------|----------|
| cardRng | Card rewards, shop cards |
| monsterRng | Encounter selection |
| eventRng | Event selection |
| relicRng | Relic pool shuffles, tier rolls |
| treasureRng | Chest type, gold variance |
| potionRng | Potion drops |
| merchantRng | Shop prices, relic tiers |

**Per-Floor (reset with seed+floorNum):**
monsterHpRng, aiRng, shuffleRng, cardRandomRng, miscRng

**Special:**
- mapRng: Reseeded per act (seed + actNum * multiplier)
- NeowEvent.rng: Separate stream for Neow options

## Training Pipeline

### Architecture (COLLECT -> TRAIN -> SYNC)
- overnight.py orchestrates the loop
- worker.py plays games using inference_server.py for batched model calls
- strategic_trainer.py runs PPO + GAE updates
- reward_config.py defines all rewards (hot-reloadable via SIGUSR1)

### REWARD_WEIGHTS System (v11)
All rewards live in `packages/training/reward_config.py`:
- `damage_per_hp`: -0.005/HP (was -0.03, reduced 6x)
- `combat_win/elite_win/boss_win`: 0.05/0.30/0.80
- `floor_milestones`: F6-F55 progression bonuses (F17=1.0, F34=2.0, F51=3.0, F55=5.0)
- `f16_hp_bonus`: 0.50 + 0.02 * current_hp (arriving healthy at boss)
- `upgrade_rewards`: Eruption +0.30, Defend_P -1.50, Strike_P -0.50
- `potion_*`: elite/boss use +0.50, kill same fight +0.50, hoard -0.30
- `win_reward`: 10.0, `death_penalty_scale`: -1.0 * (1 - progress)
- PBRS potential: floor_pct, hp_pct, deck_quality, relic_bonus
- Stance/card pick rewards: REMOVED (zeroed, was dominating)
- gamma=0.99, entropy_coeff=0.05 (cap 0.10)

### Key Parameters
- StrategicNet: 3M params, hidden=768, blocks=4, input_dim=260, action_dim=512
- 12 workers on M4 Mac Mini (10 cores, 24GB RAM)
- Mixed temperature: 75% exploit (0.9), 25% explore (1.35)
- Distillation on cold start from best_trajectories/ (200 F16 .npz files)
