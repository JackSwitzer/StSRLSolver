# SlayTheSpireRL Architecture Documentation

This document provides a comprehensive technical overview of the SlayTheSpireRL project, a reinforcement learning system for achieving high win rates in Slay the Spire.

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Core Module Structure](#core-module-structure)
3. [RNG System Deep Dive](#rng-system-deep-dive)
4. [Prediction APIs](#prediction-apis)
5. [GameRunner API](#gamerunner-api)
6. [Web Dashboard](#web-dashboard)
7. [Data Flow Diagrams](#data-flow-diagrams)
8. [Key Algorithms](#key-algorithms)

---

## Project Overview

SlayTheSpireRL is a Python-based system that:
- Implements exact game RNG mechanics for seed-deterministic prediction
- Provides APIs for predicting encounters, card rewards, relics, and more
- Includes a real-time web dashboard for game state visualization
- Supports parallel simulation for RL training

### Directory Structure

```
SlayTheSpireRL/
├── packages/engine/          # Core game engine
│   ├── state/              # RNG, run state, combat state
│   ├── generation/         # Content generators (map, encounters, rewards)
│   ├── content/            # Game data (cards, enemies, events, relics, powers, stances)
│   ├── calc/               # Damage calculation and combat simulation
│   ├── effects/            # Card effect registry and executor
│   ├── handlers/           # Combat, event, shop, reward handlers
│   ├── utils/              # Utility modules
│   ├── api.py              # Curated public API exports
│   ├── game.py             # GameRunner orchestrator
│   └── combat_engine.py    # Turn-based combat engine
├── packages/parity/         # Seed parity verification tools
│   └── comparison/         # State comparators, RNG trackers, game simulator
├── tests/                   # 2480+ pytest tests
├── mod/                     # Java EVTracker mod
├── docs/                    # Documentation
│   └── vault/              # Verified game mechanics ground truth
└── decompiled/             # Decompiled Java source (reference)
```

---

## Core Module Structure

### `packages/engine/state/` - State Management

#### `rng.py` - XorShift128 Random Number Generator

The heart of the prediction system. Implements the exact RNG algorithm used by Slay the Spire (Java's SplittableRandom variant).

```python
from packages.engine.state.rng import Random, seed_to_long, long_to_seed

# Convert alphanumeric seed to numeric
seed_long = seed_to_long("ABC123XYZ")  # -> 64-bit integer

# Create RNG instance
rng = Random(seed_long)

# Generate random numbers (Java-compatible)
value = rng.random(99)           # 0 to 99 inclusive
value = rng.random_boolean(0.5)  # 50% chance true
value = rng.random_long()        # 64-bit random long

# Create RNG at specific counter position
rng = Random(seed_long, counter=250)  # Skip ahead
```

**XorShift128 Algorithm:**
```
seed0, seed1 = initial state (derived from seed)

function next():
    s1 = seed0
    s0 = seed1
    seed0 = s0
    s1 ^= (s1 << 23)
    seed1 = s1 ^ s0 ^ (s1 >> 17) ^ (s0 >> 26)
    return seed1 + s0
```

#### `game_rng.py` - GameRNGState Manager

Manages all 13 RNG streams and their counters throughout a run.

```python
from packages.engine.state.game_rng import GameRNGState, RNGStream

# Initialize RNG state for a seed
state = GameRNGState(seed_long)

# Access specific streams
card_rng = state.get_stream(RNGStream.CARD)
monster_rng = state.get_stream(RNGStream.MONSTER)

# Get counter values (for save/restore)
counters = state.get_all_counters()
```

**RNG Streams:**
| Stream | Purpose | Persistence |
|--------|---------|-------------|
| `cardRng` | Card rewards, shop cards | Entire run |
| `monsterRng` | Encounter selection | Entire run |
| `eventRng` | Event selection | Entire run |
| `relicRng` | Relic pool shuffles, tier rolls | Entire run |
| `treasureRng` | Chest type, gold variance | Entire run |
| `potionRng` | Potion drops | Entire run |
| `merchantRng` | Shop prices, relic tiers | Entire run |
| `monsterHpRng` | Enemy HP variance | Per-floor |
| `aiRng` | Enemy AI decisions | Per-floor |
| `shuffleRng` | Deck shuffling | Per-floor |
| `cardRandomRng` | Card random effects | Per-floor |
| `miscRng` | Miscellaneous random effects | Per-floor |
| `mapRng` | Map generation | Per-act |

#### `run.py` - RunState Tracker

Complete state of a run in progress, tracking deck, relics, HP, gold, map position, and more.

```python
from packages.engine.state.run import RunState, create_watcher_run

# Create a new Watcher run
run = create_watcher_run("ABC123", ascension=20)

# Access state
print(run.current_hp, run.max_hp)
print(run.deck)  # List[CardInstance]
print(run.relics)  # List[RelicInstance]

# Modify state
run.add_card("Tantrum", upgraded=True)
run.add_relic("Kunai")
run.heal(15)
run.add_gold(100)

# Map navigation
paths = run.get_available_paths()
run.move_to(paths[0].x, paths[0].y)
run.advance_floor()
```

---

### `packages/engine/generation/` - Content Generators

#### `encounters.py` - Encounter Prediction

Predicts monster encounters, elites, and bosses for all acts.

```python
from packages.engine.generation.encounters import (
    predict_act_encounters,
    predict_all_acts,
    predict_all_bosses_extended,
)

# Predict all encounters for a seed
all_acts = predict_all_acts("ABC123XYZ")
# Returns: {
#   "act1": {"monsters": [...], "elites": [...], "boss": "..."},
#   "act2": {...},
#   "act3": {...},
# }

# Predict bosses (including A20 double boss in Act 3)
bosses = predict_all_bosses_extended(seed_long, ascension=20)
# Returns: {1: ["Hexaghost"], 2: ["Collector"], 3: ["Awakened One", "Time Eater"]}
```

**Encounter Pool Algorithm:**
1. Monster pools are pre-shuffled at dungeon init using `monsterRng.randomLong()`
2. Encounters are popped from front of shuffled pool
3. Pool is reseeded each act with `seed + act_offset`

#### `map.py` - Map Generation

Generates the complete dungeon map for each act.

```python
from packages.engine.generation.map import (
    MapGenerator,
    MapGeneratorConfig,
    RoomType,
    get_map_seed_offset,
)

# Generate Act 1 map
config = MapGeneratorConfig(ascension_level=20)
map_seed = seed_long + get_map_seed_offset(act=1)
map_rng = Random(map_seed)
generator = MapGenerator(map_rng, config)
dungeon = generator.generate()  # 2D list of MapRoomNode

# Map seed offsets by act
# Act 1: seed + 1
# Act 2: seed + 200
# Act 3: seed + 600
```

**Room Types:**
- `M` - Monster (normal combat)
- `E` - Elite (elite combat)
- `R` - Rest site
- `$` - Shop
- `T` - Treasure
- `?` - Event (or monster at A15+)
- `B` - Boss

#### `relics.py` - Relic Pool Prediction

Predicts shuffled relic pools and boss relic offerings.

```python
from packages.engine.generation.relics import (
    predict_all_relic_pools,
    predict_neow_boss_swap,
    predict_calling_bell_relics,
    create_relic_pool_state,
    get_boss_relic_choices,
)

# Get all shuffled relic pools
pools = predict_all_relic_pools(seed_long, "WATCHER")
print(pools.common)    # Shuffled common relic order
print(pools.boss)      # Shuffled boss relic order

# Predict Neow boss swap result
boss_relic = predict_neow_boss_swap(seed_long, "WATCHER")

# Create mutable pool state for tracking consumption
pool_state = create_relic_pool_state(seed_long, "WATCHER")
choices = get_boss_relic_choices(pool_state, count=3)
```

**Relic Pool Shuffle Algorithm:**
1. `relicRng` initialized with seed
2. 5 `randomLong()` calls generate shuffle seeds for each pool
3. Each pool shuffled using Java's `Collections.shuffle()` algorithm

#### `potions.py` - Potion Drop Prediction

Predicts potion drops and which specific potion will be obtained.

```python
from packages.engine.generation.potions import (
    predict_potion_drop,
    predict_potion_from_seed,
    PotionPrediction,
)

# Predict potion from seed + counter
pred = predict_potion_from_seed(
    seed="ABC123",
    potion_counter=5,
    blizzard_mod=10,
    player_class="WATCHER",
    room_type="monster",
)
print(pred.will_drop)     # True/False
print(pred.potion_id)     # "Fire Potion"
print(pred.new_blizzard_mod)  # Updated blizzard modifier
```

**Potion Drop Algorithm:**
1. Roll `potionRng.random(0, 99)` for drop check
2. Base 40% chance + blizzard modifier
3. If drop: roll rarity (65% common, 25% uncommon, 10% rare)
4. Loop `potionRng.random(pool_size-1)` until matching rarity

#### `rewards.py` - Card Reward Generation

Generates card rewards after combat.

```python
from packages.engine.generation.rewards import (
    generate_card_rewards,
    RewardState,
    CardBlizzardState,
)

# Create reward state for tracking pity timers
state = RewardState()

# Generate card rewards
cards = generate_card_rewards(
    rng=card_rng,
    reward_state=state,
    act=1,
    player_class="WATCHER",
    ascension=20,
    room_type="elite",  # or "normal"
)
```

**Card Reward Algorithm:**
1. Roll rarity with blizzard offset (pity timer)
   - Rare threshold: 3% (elite: 10%)
   - Uncommon: 37% (elite: 40%)
   - Common: 60% (elite: 50%)
2. Select card from pool using `cardRng.random(pool_size-1)`
3. Reroll on duplicates within same reward
4. Upgrade check for non-rare cards (Act 2+)

#### `shop.py` - Shop Inventory Prediction

Predicts complete shop inventory including cards, relics, and potions.

```python
from packages.engine.generation.shop import predict_shop_inventory, format_shop_inventory

result = predict_shop_inventory(
    seed="ABC123",
    card_counter=50,
    merchant_counter=3,
    potion_counter=8,
    act=1,
    player_class="WATCHER",
    owned_relics={"PureWater"},
)

inv = result.inventory
print([c.card.name for c in inv.colored_cards])  # 5 colored cards
print([r.relic.name for r in inv.relics])        # 3 relics
```

#### `treasure.py` - Treasure Chest Prediction

Predicts chest contents including type, relic tier, and gold.

```python
from packages.engine.generation.treasure import predict_full_chest, ChestType

pred = predict_full_chest(
    seed=seed_long,
    treasure_counter=2,
    player_class="WATCHER",
)
print(pred.chest_type)   # ChestType.MEDIUM
print(pred.relic_name)   # "Shuriken"
print(pred.gold_amount)  # 55 (if has_gold)
```

---

### `packages/engine/content/` - Game Data

Static definitions for all game content.

- **`cards.py`** - Card definitions (name, cost, damage, effects, rarity)
- **`enemies.py`** - Enemy definitions (HP, moves, AI patterns)
- **`events.py`** - Event definitions (choices, outcomes)
- **`relics.py`** - Relic definitions (tier, effects, restrictions)
- **`potions.py`** - Potion definitions (rarity, effects)
- **`stances.py`** - Stance mechanics (Wrath, Calm, Divinity)

---

### `packages/engine/game.py` - GameRunner Orchestrator

Main game loop manager that handles all phases of a run.

```python
from packages.engine.game import GameRunner, GamePhase

# Initialize a run
runner = GameRunner(seed="ABC123", ascension=20)

# Manual control loop
while not runner.game_over:
    actions = runner.get_available_actions()
    # Bot/RL agent selects action
    runner.take_action(actions[0])

# Or automatic random play
stats = runner.run()
```

**Game Phases:**
| Phase | Description |
|-------|-------------|
| `NEOW` | Neow's blessing selection (start of run) |
| `MAP_NAVIGATION` | Choosing next room on map |
| `COMBAT` | Active combat encounter |
| `COMBAT_REWARDS` | Selecting rewards after victory |
| `EVENT` | Making event choices |
| `SHOP` | Browsing/buying in shop |
| `REST` | Rest site actions (heal/upgrade/dig) |
| `TREASURE` | Treasure room (take relic/key) |
| `BOSS_REWARDS` | Selecting boss relic |
| `RUN_COMPLETE` | Game ended (win/loss) |

---

### `packages/engine/simulation/` - Parallel Simulation Engine

High-throughput parallel simulation for RL training.

#### `engine.py` - ParallelSimulator

```python
from packages.engine.simulation.engine import ParallelSimulator, SimulationConfig

# Configure simulation
config = SimulationConfig(
    n_workers=8,
    batch_size=100,
    default_ascension=20,
    max_turns_per_combat=100,
)

# Run parallel simulations
with ParallelSimulator(config=config) as sim:
    # Batch simulation
    results = sim.simulate_batch(
        seeds=["SEED1", "SEED2", "SEED3", ...],
        agent=my_rl_agent,
        track_decisions=True,
    )

    # Combat simulation for MCTS
    outcomes = sim.simulate_combats(
        combat_states=[state1, state2],
        actions=[[action1], [action2]],
    )

    # Best play search
    best = sim.find_best_play(
        combat_state=current_combat,
        search_budget=1000,
    )
```

**Performance Optimizations:**
- ProcessPoolExecutor for true parallelism (bypasses GIL)
- Pre-forked worker pool to avoid spawn overhead
- Batch processing to minimize IPC overhead
- Pickle protocol 5 for efficient state serialization

---

## RNG System Deep Dive

### Seed Conversion

Slay the Spire uses alphanumeric seeds (base-35: 0-9 + A-Y, no Z) that convert to 64-bit signed integers.

```python
from packages.engine.state.rng import seed_to_long, long_to_seed

# Alphanumeric to numeric
seed_long = seed_to_long("ABC123XYZ")

# Numeric back to alphanumeric
seed_str = long_to_seed(seed_long)
```

**Conversion Algorithm:**
```
BASE35_CHARS = "0123456789ABCDEFGHIJKLMNOPQRSTUVWXY"

def seed_to_long(seed_str):
    result = 0
    for char in seed_str:
        result = result * 35 + BASE35_CHARS.index(char)
    return result
```

### Act Transition Counter Snapping

When transitioning between acts, the `cardRng` counter is "snapped" to fixed values to maintain seed consistency regardless of path taken.

```
Counter Range    | Snapped To
-----------------|------------
1 - 249         | 250
251 - 499       | 500
501 - 749       | 750
```

This ensures that card rewards in Act 2 are consistent regardless of how many shops/events were visited in Act 1.

### 13 RNG Streams

| Stream | Seed Offset | Persistence | Used For |
|--------|-------------|-------------|----------|
| cardRng | 0 | Run | Card rewards, shop cards |
| monsterRng | varies | Run | Monster encounter selection |
| eventRng | varies | Run | Event selection |
| relicRng | 0 | Run | Relic pools, tier rolls |
| treasureRng | varies | Run | Chest type, gold |
| potionRng | varies | Run | Potion drops |
| merchantRng | varies | Run | Shop prices |
| monsterHpRng | floor | Floor | Enemy HP variance |
| aiRng | floor | Floor | Enemy AI decisions |
| shuffleRng | floor | Floor | Deck shuffling |
| cardRandomRng | floor | Floor | Random card effects |
| miscRng | floor | Floor | Misc random |
| mapRng | act*offset | Act | Map generation |

---

## Prediction APIs

### `predict_all_acts(seed)` - Full Run Encounter Prediction

```python
from packages.engine.generation.encounters import predict_all_acts

predictions = predict_all_acts("YOUR_SEED")
# {
#   "act1": {
#     "monsters": ["Jaw Worm", "2 Louse", "Gremlin Gang", ...],
#     "elites": ["Lagavulin", "3 Sentries", "Gremlin Nob"],
#     "boss": "Hexaghost"
#   },
#   "act2": {...},
#   "act3": {...}
# }
```

### `predict_all_bosses_extended(seed, ascension)` - Boss Prediction

Includes A20 double boss in Act 3:

```python
from packages.engine.generation.encounters import predict_all_bosses_extended

bosses = predict_all_bosses_extended(seed_long, ascension=20)
# {
#   1: ["Hexaghost"],
#   2: ["The Collector"],
#   3: ["Awakened One", "Time Eater"]  # Two bosses at A20
# }
```

### Card Reward Prediction

```python
from packages.engine.comparison.full_rng_tracker import predict_card_reward

cards, new_counter = predict_card_reward(
    seed="ABC123",
    card_counter=50,
    act=1,
    room_type="elite",
    card_blizzard=3,
    relics=["PureWater", "Question Card"],
)
# Returns: (["Tantrum", "Cut Through Fate", "Ragnarok"], 53)
```

### Map Generation

```python
from packages.engine.generation.map import MapGenerator, MapGeneratorConfig, get_map_seed_offset
from packages.engine.state.rng import Random

# Generate Act 2 map
act = 2
map_seed = seed_long + get_map_seed_offset(act)
map_rng = Random(map_seed)
config = MapGeneratorConfig(ascension_level=20)
generator = MapGenerator(map_rng, config)
dungeon = generator.generate()

# dungeon is List[List[MapRoomNode]]
# dungeon[0] = first row (floor 1)
# dungeon[y][x].room_type = RoomType enum
# dungeon[y][x].edges = connections to next floor
```

---

## GameRunner API

### Initialization

```python
from packages.engine.game import GameRunner

runner = GameRunner(
    seed="ABC123",      # Seed string or numeric
    ascension=20,       # Ascension level 0-20
    character="Watcher", # Only Watcher supported
    skip_neow=True,     # Skip Neow blessing phase
    verbose=True,       # Print game events
)
```

### get_available_actions()

Returns all valid actions for the current game state:

```python
actions = runner.get_available_actions()
# Returns list of GameAction subclasses:
# - PathAction(node_index)      # Map navigation
# - NeowAction(choice_index)    # Neow blessing
# - CombatAction(type, ...)     # Combat moves
# - RewardAction(type, index)   # Reward selection
# - EventAction(choice_index)   # Event choices
# - ShopAction(type, index)     # Shop actions
# - RestAction(type, card_idx)  # Rest site
# - TreasureAction(type)        # Treasure room
# - BossRewardAction(relic_idx) # Boss relic
```

### take_action(action)

Execute an action and advance game state:

```python
success = runner.take_action(actions[0])
# Returns True if action was valid and executed
# Game state is updated, phase may change
```

### Running Full Simulations

```python
# Automatic random play to completion
stats = runner.run()
# Returns dict with win/loss, final floor, HP, etc.

# Run to specific floor
stats = runner.run_to_floor(target_floor=17)

# Manual control with agent
while not runner.game_over:
    state = runner.run_state
    actions = runner.get_available_actions()

    # Your agent logic here
    action = agent.select_action(state, actions)

    runner.take_action(action)
```

### Decision Logging

All decisions are logged for training/analysis:

```python
for entry in runner.decision_log:
    print(f"Floor {entry.floor}: {entry.phase.name}")
    print(f"  Action: {entry.action_taken}")
    print(f"  Options: {len(entry.available_actions)}")
    print(f"  Result: {entry.result}")
```

---

## Web Dashboard

### `web/server.py` - FastAPI Server

Real-time dashboard for monitoring game state during live play.

```bash
# Start the server
uv run python web/server.py
# Open http://localhost:8080
```

### Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/` | GET | Main dashboard HTML |
| `/api/state` | GET | Current game state JSON |
| `/api/stream` | GET | SSE stream for live updates |
| `/api/predict/cards` | GET | Predict card rewards |
| `/api/predict/boss-relics` | GET | Predict boss relics |
| `/api/predict/path` | POST | Predict rewards for path |

### SSE Streaming

The dashboard uses Server-Sent Events for real-time updates:

```javascript
const eventSource = new EventSource('/api/stream');
eventSource.onmessage = (event) => {
    const state = JSON.parse(event.data);
    updateDashboard(state);
};
```

### Save File Watching

The server monitors the game's autosave file for changes:

```python
SAVE_PATH = "~/.../SlayTheSpire/saves/IRONCLAD.autosave"

# Polls every 2 seconds for changes
while True:
    if file_modified():
        save_data = read_save_file()
        state = process_save_data(save_data)
        yield f"data: {json.dumps(state)}\n\n"
    await asyncio.sleep(2)
```

### State Processing

Converts raw save data to dashboard format with predictions:

```python
def process_save_data(save_data):
    return {
        "seed": {"string": seed_str, "long": seed_long},
        "run": {"floor": floor, "act": act, ...},
        "player": {"hp": hp, "max_hp": max_hp, "gold": gold},
        "rng": {"card_counter": n, "relic_counter": n, ...},
        "map": map_data,
        "accessible_nodes": [...],
        "node_predictions": [...],  # Card rewards, enemies, etc.
        "path_tree": {...},         # Next 3 floors of paths
        "predicted_bosses": {...},
        "rng_accuracy": {...},      # Prediction vs actual comparison
    }
```

---

## Data Flow Diagrams

### Seed to Encounters

```
                                    +------------------+
                                    | Encounter Pools  |
                                    | (pre-shuffled)   |
                                    +------------------+
                                           ^
                                           |
+--------+     +-------------+     +-------+-------+     +------------+
|  Seed  | --> | seed_to_long| --> |  monsterRng   | --> | Shuffle    |
| "ABC"  |     | (base-35)   |     | .randomLong() |     | Pools      |
+--------+     +-------------+     +---------------+     +------------+
                                           |
                                           v
                              +------------------------+
                              | Act 1 Monsters:        |
                              | [Jaw Worm, 2 Louse,..] |
                              | Act 1 Elites:          |
                              | [Lagavulin, Nob,..]    |
                              +------------------------+
```

### Save File to Dashboard

```
+------------------+     +-----------------+     +------------------+
| SlayTheSpire     |     | save.autosave   |     | read_save_file() |
| (Game)           | --> | (JSON+base64)   | --> | (Decode)         |
+------------------+     +-----------------+     +------------------+
                                                         |
                                                         v
+------------------+     +-----------------+     +------------------+
| Dashboard        | <-- | SSE Stream      | <-- | process_save_data|
| (Browser)        |     | (JSON events)   |     | (Add predictions)|
+------------------+     +-----------------+     +------------------+
                                                         |
                                                         v
                                              +------------------+
                                              | - Map generation |
                                              | - Card rewards   |
                                              | - Boss relics    |
                                              | - Path tree      |
                                              +------------------+
```

### Card Reward Generation

```
+----------+     +----------+     +----------+     +----------+
| cardRng  | --> | Roll     | --> | Get Pool | --> | Select   |
| .random  |     | Rarity   |     | (rarity) |     | Card     |
+----------+     +----------+     +----------+     +----------+
     |                                                   |
     |           +----------+                            v
     +---------> | Blizzard | --(offset)--> Rarity Thresholds
                 | State    |
                 +----------+
                      |
                      v (reset on rare, decrement on common)
               +-------------+
               | Card Reward |
               | [3 cards]   |
               +-------------+
```

### Combat Flow

```
+-------------+     +-------------+     +-------------+
| Start Turn  | --> | Player      | --> | Play Cards  |
| - Reset     |     | Actions     |     | - Use energy|
| - Draw 5    |     | - get_legal |     | - Effects   |
+-------------+     +-------------+     +-------------+
      ^                                       |
      |                                       v
+-------------+     +-------------+     +-------------+
| End of      | <-- | Monster     | <-- | End Turn    |
| Round       |     | Turn        |     | - Discard   |
| - Triggers  |     | - AI moves  |     | - Triggers  |
+-------------+     +-------------+     +-------------+
      |
      v
+-------------+
| Check Win/  |
| Lose        |
+-------------+
```

---

## Key Algorithms

### Java Collections.shuffle() Replication

Critical for relic pool prediction:

```python
def java_collections_shuffle(lst, java_random_seed):
    # Initialize Java Random state
    MULTIPLIER = 0x5DEECE66D
    ADDEND = 0xB
    MASK = (1 << 48) - 1

    seed = (java_random_seed ^ MULTIPLIER) & MASK

    def next_int(n):
        # Java's Random.nextInt(n) algorithm
        ...

    # Fisher-Yates shuffle
    for i in range(len(lst), 1, -1):
        j = next_int(i)
        lst[i-1], lst[j] = lst[j], lst[i-1]

    return lst
```

### HashMap Iteration Order

Java HashMap iteration order is deterministic but depends on hash values:

```python
class JavaHashMap:
    def __init__(self, initial_capacity=16):
        self.capacity = initial_capacity
        self.buckets = [[] for _ in range(initial_capacity)]

    def put(self, key, value):
        bucket_idx = self._hash(key) % self.capacity
        self.buckets[bucket_idx].append((key, value))

    def items_in_iteration_order(self):
        # Iterate buckets in order, items within bucket in insertion order
        for bucket in self.buckets:
            for key, value in bucket:
                yield key, value
```

### Card Blizzard (Pity Timer)

Increases rare card chance when getting commons:

```python
class CardBlizzardState:
    offset = 5  # Starting offset (+5 to roll)

    def on_common(self):
        self.offset -= 1  # Decreases offset (more likely rare)
        self.offset = max(self.offset, -40)

    def on_rare(self):
        self.offset = 5  # Reset to starting

    def on_uncommon(self):
        pass  # No change

# Roll with blizzard
roll = rng.random(99) + blizzard.offset
if roll < rare_threshold:
    return RARE
```

---

## Usage Examples

### Predict Full Run

```python
from packages.engine.state.rng import seed_to_long
from packages.engine.generation.encounters import predict_all_acts
from packages.engine.generation.relics import predict_all_relic_pools

seed = "MYSEED123"
seed_long = seed_to_long(seed)

# Get all encounters
encounters = predict_all_acts(seed)
print(f"Act 1 Boss: {encounters['act1']['boss']}")
print(f"Act 1 Elites: {encounters['act1']['elites']}")

# Get relic pools
pools = predict_all_relic_pools(seed_long, "WATCHER")
print(f"First boss relic: {pools.boss[0]}")
```

### Simulate Combat

```python
from packages.engine.simulation.engine import ParallelSimulator

with ParallelSimulator(n_workers=4) as sim:
    result = sim.simulate_combat_single(
        combat_state=my_combat_state,
        actions=[play_eruption, play_defend],
        max_turns=50,
    )
    print(f"Victory: {result.victory}")
    print(f"HP remaining: {result.hp_remaining}")
```

### Run Dashboard

```bash
# Terminal 1: Start dashboard
uv run python web/server.py

# Terminal 2: Play game
# Dashboard auto-updates as you play
```

---

## References

- **Decompiled Source**: `decompiled/java-src/com/megacrit/cardcrawl/`
- **Vault Documentation**: `docs/vault/` (verified game mechanics)
- **CLAUDE.md**: Project decisions and learnings
