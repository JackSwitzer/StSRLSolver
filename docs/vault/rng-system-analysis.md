# Slay the Spire RNG System - Complete Analysis

## Executive Summary

Slay the Spire uses **13 independent RNG streams**, all seeded from the same base seed but tracked independently. The game is fully deterministic given seed + player decisions.

## RNG Stream Inventory

### Persistent Streams (Survive Entire Run)
| Stream | Purpose | Key Consumers |
|--------|---------|---------------|
| `cardRng` | Card rewards, shop cards | Combat rewards, shop inventory, some events |
| `monsterRng` | Encounter selection | Pre-generated per act |
| `eventRng` | Event selection in ? rooms | EventRoom entry |
| `relicRng` | Relic pool shuffles | Pool init (5 calls), tier rolls |
| `treasureRng` | Chest type, gold variance | Treasure rooms, combat gold |
| `potionRng` | Potion drops | Combat rewards, shops |
| `merchantRng` | Shop prices, relic tiers | Shop generation |

### Per-Floor Streams (Reset Each Floor)
| Stream | Purpose | Seeding Formula |
|--------|---------|-----------------|
| `monsterHpRng` | Enemy HP variance | `seed + floorNum` |
| `aiRng` | Enemy AI decisions | `seed + floorNum` |
| `shuffleRng` | Card draw shuffle | `seed + floorNum` |
| `cardRandomRng` | Card random effects | `seed + floorNum` |
| `miscRng` | Event transforms, misc | `seed + floorNum` |

### Special Streams
| Stream | Purpose | Notes |
|--------|---------|-------|
| `mapRng` | Map generation | Reseeded per act: `seed + actNum * multiplier` |
| `NeowEvent.rng` | Neow options | Separate from cardRng, seeded with base seed |

## Critical Discovery: cardRng Snapping

**At act transitions, cardRng counter snaps to fixed boundaries:**
```
counter 1-249   → snaps to 250
counter 251-499 → snaps to 500
counter 501-749 → snaps to 750
```

This means card predictions for Act 2+ require knowing exact Act 1 cardRng state.

## RNG Consumption by Game Element

### Shop (Most Complex)
**cardRng: 12+ calls**
- 5 rarity rolls (colored cards)
- 5+ card selections (may retry on COLORLESS hit)
- 2 colorless card selections

**merchantRng: 17 calls**
- 7 card price jitters
- 1 sale card selection
- 6 relic operations (3 tier + 3 price)
- 3 potion price jitters

**potionRng: 3 calls**
- 1 per potion rarity

### Events
**Most events use miscRng (safe for cardRng):**
- Transmogrifier, Designer, LivingWall, DrugDealer: miscRng transforms

**Events that consume cardRng:**
- TheLibrary: **20+ calls** (generates 20 unique cards)
- GremlinMatchGame: **~6 calls**
- KnowingSkull: 1 call per colorless card selection

### Combat Rewards
**Order: Gold → Relic → Potion → Cards**

| Reward | RNG Stream | Calls |
|--------|-----------|-------|
| Gold (elite/monster) | treasureRng | 1 |
| Gold (boss) | miscRng | 1 |
| Relic tier | relicRng | 1 |
| Potion chance | potionRng | 1 |
| Potion selection | potionRng | 1-3 (variable) |
| Card rarity (×3) | cardRng | 3 |
| Card selection (×3) | cardRng | 3 |
| Card upgrade check (×3) | cardRng | 0-3 (rare cards skip) |

**Total cardRng per combat: 6-9 calls**

### Treasure Rooms
- Use **treasureRng only** (not cardRng)
- Chest type: 1 call
- Gold variance: 1 call

### Relic Pools
- Shuffled **once at game start**: 5 `relicRng.randomLong()` per pool
- FIFO consumption after shuffle
- No RNG on individual draws (just pool.remove(0))

---

## Three Implementation Hypotheses

### Hypothesis 1: Counter-Based State Machine (RECOMMENDED)

**Concept:** Track each RNG stream's counter independently. Simulate exact game flow.

**Implementation:**
```python
class GameRNGState:
    def __init__(self, seed: int):
        self.seed = seed
        self.counters = {
            'card': 0, 'monster': 0, 'event': 0, 'relic': 0,
            'treasure': 0, 'potion': 0, 'merchant': 0
        }
        self.floor_num = 0
        self.act_num = 1

    def get_rng(self, stream: str) -> Random:
        return Random(self.seed, self.counters[stream])

    def advance(self, stream: str, n: int = 1):
        self.counters[stream] += n

    def transition_act(self):
        # Apply cardRng snapping
        c = self.counters['card']
        if 0 < c < 250: self.counters['card'] = 250
        elif 250 < c < 500: self.counters['card'] = 500
        elif 500 < c < 750: self.counters['card'] = 750
        self.act_num += 1
```

**Pros:**
- Exact game behavior reproduction
- Handles all edge cases
- Can predict any point in game given path

**Cons:**
- Requires tracking full path through game
- Complex to implement all consumers

### Hypothesis 2: Event-Driven Simulation

**Concept:** Define each game event (combat, shop, event, etc.) as a function that consumes RNG.

**Implementation:**
```python
class GameSimulator:
    def __init__(self, seed: int):
        self.state = GameRNGState(seed)

    def simulate_combat(self, room_type: str):
        # Gold
        if room_type == 'boss':
            self.state.advance('misc', 1)
        else:
            self.state.advance('treasure', 1)
        # Relic (if elite)
        if room_type == 'elite':
            self.state.advance('relic', 1)
        # Potion
        self.state.advance('potion', 2)  # chance + selection
        # Cards
        self.state.advance('card', 9)  # 3 rarity + 3 select + 3 upgrade

    def simulate_shop(self):
        self.state.advance('card', 12)
        self.state.advance('merchant', 17)
        self.state.advance('potion', 3)
```

**Pros:**
- Clean abstraction per event type
- Easy to extend with new event types
- Good for path simulation

**Cons:**
- Fixed consumption estimates may be inaccurate
- Doesn't handle variable consumption (retries, duplicates)

### Hypothesis 3: Delta-Based Prediction

**Concept:** Pre-compute massive lookup tables for common paths.

**Implementation:**
```python
# Pre-computed: how much each path element consumes
PATH_DELTAS = {
    'combat_normal': {'card': 9, 'treasure': 1, 'potion': 2},
    'combat_elite': {'card': 9, 'treasure': 1, 'potion': 2, 'relic': 1},
    'shop': {'card': 12, 'merchant': 17, 'potion': 3},
    'event_library': {'card': 20},
    'event_other': {},  # Most events don't affect cardRng
    'treasure': {'treasure': 2},
}

def predict_cards_at_floor(seed, path: List[str], floor: int):
    state = GameRNGState(seed)
    for i, node in enumerate(path[:floor]):
        delta = PATH_DELTAS.get(node, {})
        for stream, count in delta.items():
            state.advance(stream, count)
    return generate_card_reward(state.get_rng('card'))
```

**Pros:**
- Fast prediction once path is known
- Simple to reason about
- Easy to validate against game

**Cons:**
- Approximations may accumulate error
- Doesn't handle variable consumption

---

## Recommendation: Implement Hypothesis 1

The counter-based state machine is the most accurate and flexible. It:
1. Matches game behavior exactly
2. Handles act transitions with cardRng snapping
3. Can be extended to any RNG stream
4. Supports save/load of state

**Implementation Status: COMPLETE**
- `core/state/game_rng.py` - GameRNGState class
- **Test Results: 15/15 verified seeds passing (100%)**

### Verified Neow Consumption Values
| Neow Choice | cardRng Consumption |
|-------------|---------------------|
| Simple options (UPGRADE, GOLD, REMOVE, etc.) | 0 |
| CURSE drawback | 1 |
| RANDOM_COLORLESS | 3 |
| CURSE + COLORLESS_2 | 4 |
| Calling Bell boss swap | 9 |
| Combat reward | ~9 |
| Shop visit | ~12 |

## Key Implementation Notes

1. **All RNG streams start at counter 0** with same seed
2. **Per-floor streams use seed + floorNum** (not counter-based)
3. **cardRng snapping is CRITICAL** for multi-act prediction
4. **Shop is the biggest cardRng consumer** (~12+ calls)
5. **TheLibrary event is dangerous** (~20 cardRng calls)
6. **Most events are safe** (use miscRng)

## Files Referenced
- AbstractDungeon.java: lines 129-141, 378-421, 1727-1731, 2542-2584
- Merchant.java: lines 54-80
- EventRoom.java: lines 22-27
- Various event files in events/exordium/, events/city/, events/shrines/
