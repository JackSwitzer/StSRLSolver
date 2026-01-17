# StSRLSolver Project Analysis

Cloned to: `/Users/jackswitzer/Desktop/SlayTheSpireRL/reference/StSRLSolver`

## Architecture

```
StSRLSolver/
├── agent.py              # Main entry, Coordinator integration
├── watcher_agent.py      # Watcher logic (439 lines)
├── watcher_priorities.py # Card/relic priorities (389 lines)
├── bc_agent.py           # Behavioral cloning agent
├── train_bc.py           # BC trainer
├── self_play_trainer.py  # Self-play loop (532 lines)
├── models/
│   ├── encoding.py       # State/action encoding (290 lines)
│   ├── bc_model.py       # MLP architecture (289 lines)
│   ├── strategic_features.py # 44 features (285 lines)
│   ├── line_evaluator.py # Combat simulation (572 lines)
│   ├── combat_calculator.py # Damage/block calc (408 lines)
│   ├── enemy_database.py # Enemy stats (339 lines)
│   └── mcts.py           # AlphaZero-style MCTS (345 lines)
├── spirecomm/            # Empty - needs ForgottenArbiter/spirecomm
└── pyproject.toml        # uv, torch>=2.0, numpy>=1.24
```

## Implementation Status

**Done:**
- CommunicationMod via spirecomm (Coordinator pattern)
- Full heuristic Watcher agent with stance management
- Card/relic priority lists
- BC model (3-layer MLP, 512->256->128, LayerNorm, GELU, Dropout)
- 44 strategic features for NN input
- Self-play data collection
- Line evaluator for card sequences
- Enemy database with threat levels
- MCTS infrastructure (PUCT)
- Data processing for Nov 2020 dataset

**TODO:**
- Validate BC baseline performance
- Self-play optimization
- Feature importance analysis
- Win rate benchmarking
- Full MCTS integration

## Interface

Uses CommunicationMod:
```python
from spirecomm.communication.coordinator import Coordinator
coordinator = Coordinator()
coordinator.signal_ready()
coordinator.register_state_change_callback(agent.get_next_action_in_game)
coordinator.play_one_game(PlayerClass.WATCHER, ascension_level=0)
```

## State Encoding

**Card-Level (265 dims):**
- Deck: multi-hot counts for 140+ cards
- Relics: multi-hot for 120+ relics
- HP ratio, gold, ascension, floor/act

**Strategic (44 features):**
1. Lethal (6): can_lethal_all, enemy_can_lethal, turns_to_kill
2. Resources (6): energy, effective_energy, cards_playable, hand_quality
3. Damage (6): max_damage_this_turn, damage_efficiency, total_enemy_hp
4. Defense (6): incoming_damage, max_block, expected_damage_taken
5. Stance (6): current_stance, can_enter_wrath/calm, energy_pending
6. Deck Cycle (5): deck_size, turns_until_reshuffle, key_cards_in_draw
7. Potions (4): has_damage_potion, should_potion
8. Context (5): is_elite, is_boss, fight_turn, race_condition

## RL Approach

**Stage 1: Behavioral Cloning**
- 77M official runs filtered to Watcher A20 wins
- CrossEntropyLoss for card pick
- AdamW + cosine LR

**Stage 2: Self-Play**
- Line evaluation picks best combat actions
- Records (state, action, outcome)
- 10% exploration rate

**Stage 3: MCTS (planned)**
- AlphaZero PUCT: UCB = Q + c * P * sqrt(N) / (1+N)
- Dirichlet noise at root

## Watcher Logic

Priority: Exit Wrath if attacked -> Powers first -> Block if needed -> Zero-cost non-attacks -> Non-zero -> Zero-cost attacks -> AOE for multi

Card tiers: Rushdown, Tantrum, Ragnarok, MentalFortress > InnerPeace, CutThroughFate, WheelKick > Skip line

## Experience Recording

```python
@dataclass
class Experience:
    state_features: List[float]
    action_taken: str
    line_score: float
    actual_outcome: str  # win/loss/continue
    damage_taken: int
    damage_dealt: int
    floor: int
    turn: int
```

Line scoring: +1000 lethal, +100 safe, +50/kill, +2/damage, -5/damage_taken, -50 end in Wrath, -10000 die

## Launch

```bash
uv sync
./launch_game.sh      # STS with mods
./run_agent.sh        # Heuristic agent
./run_selfplay.sh     # 10000 games, 15% explore
python train_bc.py    # Train BC
```
