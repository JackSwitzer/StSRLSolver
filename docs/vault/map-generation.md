# Map Generation Algorithm

Extracted from decompiled source: `com/megacrit/cardcrawl/map/MapGenerator.java` and `AbstractDungeon.java`.

## Map Structure

### Dimensions
- **Height**: 15 floors per act (0-14, with row 14 leading to boss)
- **Width**: 7 columns (indices 0-6)
- **Path Density**: 6 paths generated per map
- **Boss Floor**: Floor 16 (actually y=15, outside the 15-row grid - accessed from row 14)

```
Floor 15: Boss Room (single node, not part of grid)
Floor 14: Rest Sites (row before boss)
Floor 8: Treasure Room (fixed)
Floor 0: Monster Rooms (first floor - always monsters)
```

### Act 4 (The Ending) - Special Map
Act 4 has a completely different structure:
- 5 floors total (0-4)
- Single linear path in column 3
- Floor 0: Rest Site
- Floor 1: Shop
- Floor 2: Elite (Shield and Spear)
- Floor 3: Boss (The Heart)
- Floor 4: True Victory Room

## Room Type Distribution

### Base Probabilities (from `initializeLevelSpecificChances`)

| Room Type | Probability | Notes |
|-----------|-------------|-------|
| Shop | 5% | `shopRoomChance = 0.05f` |
| Rest Site | 12% | `restRoomChance = 0.12f` |
| Treasure | 0% | Fixed at floor 8 instead |
| Event | 22% | `eventRoomChance = 0.22f` |
| Elite | 8% | `eliteRoomChance = 0.08f` |
| Monster | ~53% | Remainder after other types |

### Elite Room Modifications

**Ascension 1+**: Elite count multiplied by 1.6x
```java
if (ascensionLevel >= 1) {
    eliteCount = Math.round((float)availableRoomCount * eliteRoomChance * 1.6f);
}
```

**"Elite Swarm" Mod**: Elite count multiplied by 2.5x

### Fixed Room Assignments

1. **Floor 0 (Row 0)**: Always Monster Rooms
2. **Floor 8 (Row 8)**: Always Treasure Room (or Elite with MimicInfestation blight in Endless)
3. **Floor 14 (Row 14)**: Always Rest Sites

## Room Placement Rules

From `RoomTypeAssigner.java`:

### Row Restrictions (`ruleAssignableToRow`)
- **Rest Sites and Elites**: Cannot appear on floors 0-4 (first 5 floors)
- **Rest Sites**: Cannot appear on floors 13-14 (already forced on 14)

### Parent/Sibling Rules
- **No consecutive same room types** for: Rest, Treasure, Shop, Elite
- If a node's parent has one of these types, the child cannot have the same type
- If a sibling (node with same parent) has one of these types, the current node cannot have the same type

### Applicable Rule Sets

**Parent matching applies to**: Rest, Treasure, Shop, Elite
**Sibling matching applies to**: Rest, Monster, Event, Elite, Shop

## Path Connectivity Algorithm

From `MapGenerator.java`:

### Path Generation (`createPaths`)
1. Generate 6 paths (unless "Uncertain Future" mod - then 1 path)
2. First two paths must start from different columns
3. Each path starts at a random column in row 0

### Path Propagation (`_createPaths`)
For each node, the next node is chosen:

1. **Edge cases**:
   - If at column 0: can only go to columns 0 or 1
   - If at column 6: can only go to columns 5 or 6
   - Otherwise: can go to x-1, x, or x+1

2. **Common ancestor check**:
   - Prevents paths from merging too quickly
   - `min_ancestor_gap = 3`: If paths share ancestor within 3 floors, redirect
   - `max_ancestor_gap = 5`: Only check for common ancestors up to 5 floors back

3. **Cross-prevention**:
   - Paths cannot cross each other
   - If left neighbor has edge going right of current target, adjust
   - If right neighbor has edge going left of current target, adjust

### Redundant Edge Filtering
After generation, duplicate edges to same destination from row 0 are removed.

## Seed and RNG

### Map RNG Initialization
Each act uses a different offset from the base seed. The actual values from the dungeon constructors:

| Act | Formula | Actual Offset |
|-----|---------|---------------|
| Act 1 (Exordium) | `seed + 1` | +1 |
| Act 2 (The City) | `seed + 200` | +200 |
| Act 3 (The Beyond) | `seed + 600` | +600 |
| Act 4 (The Ending) | `seed + 1200` | +1200 |

```java
// From dungeon class constructors (not actNum * multiplier as some sources show)
mapRng = new Random(Settings.seed + actSpecificOffset);
```

**Note**: Earlier documentation showed `actNum * 100/200/300` which was an approximation. Our implementation in `core/generation/map.py:get_map_seed_offset()` uses the correct values above.

### RNG Usage
1. Path starting positions: `randRange(rng, 0, 6)` -> `[0, 6]` inclusive
2. Path directions: `randRange(rng, min, max)` where min/max depend on position
3. Room shuffling: `Collections.shuffle(roomList, rng.random)` - uses Fisher-Yates
4. Emerald Elite selection: `mapRng.random(0, eliteNodes.size() - 1)`

### Shuffle Algorithm (Critical Detail)

Room type distribution uses Fisher-Yates shuffle with the game's RNG:

```python
def _shuffle_with_rng(self, items: list):
    """Shuffle list using game's RNG (matches Collections.shuffle)."""
    # Fisher-Yates: iterate backwards, swap with random earlier element
    for i in range(len(items) - 1, 0, -1):
        j = self.rng.random(i)  # [0, i] INCLUSIVE - this is critical!
        items[i], items[j] = items[j], items[i]
```

The game's `random(i)` returns `[0, i]` inclusive (not exclusive). Using `[0, i)` would produce incorrect room distributions.

## Burning Elite (Emerald Key) Mechanic

When Final Act is available and Emerald Key not yet obtained:

### Placement
One random elite on the map is marked with `hasEmeraldKey = true`:
```java
if (Settings.isFinalActAvailable && !Settings.hasEmeraldKey) {
    MapRoomNode chosenNode = eliteNodes.get(mapRng.random(0, eliteNodes.size() - 1));
    chosenNode.hasEmeraldKey = true;
}
```

### Combat Buff
The elite receives one of 4 random buffs (using mapRng):

| Roll | Buff | Amount |
|------|------|--------|
| 0 | Strength | actNum + 1 |
| 1 | +25% Max HP | (healed to full) |
| 2 | Metallicize | actNum * 2 + 2 |
| 3 | Regeneration | actNum * 2 + 1 |

### Rewards
After defeating the burning elite, player chooses between:
- Taking the relic
- Taking the Emerald Key (forfeiting relic)

## Act-Specific Differences

### Exordium (Act 1)
- 3 weak enemies, 12 strong enemies, 10 elites pre-generated
- Card upgrade chance: 0%
- No differences in map structure

### The City (Act 2)
- 2 weak enemies, 12 strong enemies, 10 elites
- Card upgrade chance: 25% (12.5% at A12+)
- Map offset: seed + 200

### The Beyond (Act 3)
- 2 weak enemies, 12 strong enemies, 10 elites
- Card upgrade chance: 50% (25% at A12+)
- Map offset: seed + 600

### The Ending (Act 4)
- Completely different map structure (see above)
- Fixed encounters (Shield and Spear only)
- Map offset: seed + 1200
- 100% medium chest, 100% uncommon relic

## Technical Notes

### Node Offsets
Each node has random visual jitter:
```java
offsetX = random(-27, 27) * scale  // Mobile: -13 to 13
offsetY = random(-37, 37) * scale  // Mobile: -18 to 18
```

This is visual only and doesn't affect connectivity.

### Boss Node
The boss is not part of the 15-row grid. It's accessed via special edge from row 14 nodes with `isBoss = true` flag.

### Path Count Guarantee
The "donut check" ensures exactly one node in the final row leads to the boss (prevents multiple paths to boss splitting).

---

## Implementation

Our implementation is in `core/generation/map.py`. Key features:

- `MapGenerator` class generates maps matching game algorithm exactly
- `generate_act4_map()` creates the fixed Act 4 linear structure
- `get_map_seed_offset(act_num)` returns correct seed offsets
- `map_to_string()` creates ASCII visualization for debugging

Usage:
```python
from core.state.rng import Random
from core.generation.map import MapGenerator, MapGeneratorConfig, get_map_seed_offset

seed = 12345678
config = MapGeneratorConfig(ascension_level=20)
map_rng = Random(seed + get_map_seed_offset(1))  # Act 1
generator = MapGenerator(map_rng, config)
dungeon = generator.generate()
```
