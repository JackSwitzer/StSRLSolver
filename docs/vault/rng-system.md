# Slay the Spire RNG System

Complete analysis of the deterministic RNG system based on decompiled source code.

## XorShift128 Implementation

**Location**: `com.megacrit.cardcrawl.random.Random`

The game uses libGDX's `RandomXS128` which implements the XorShift128+ algorithm - a fast, high-quality PRNG.

### Random.java Core Implementation

```java
public class Random {
    public RandomXS128 random;
    public int counter = 0;  // Tracks number of RNG calls for save/load

    // Construct with seed
    public Random(Long seed) {
        this.random = new RandomXS128(seed);
    }

    // Construct with seed + replay N calls (for save restoration)
    public Random(Long seed, int counter) {
        this.random = new RandomXS128(seed);
        for (int i = 0; i < counter; ++i) {
            this.random(999);  // Advance state
        }
    }

    // All methods increment counter before returning
    public int random(int range) {
        ++this.counter;
        return this.random.nextInt(range + 1);  // INCLUSIVE upper bound
    }

    public int random(int start, int end) {
        ++this.counter;
        return start + this.random.nextInt(end - start + 1);  // INCLUSIVE
    }

    public boolean randomBoolean() {
        ++this.counter;
        return this.random.nextBoolean();
    }

    public boolean randomBoolean(float chance) {
        ++this.counter;
        return this.random.nextFloat() < chance;
    }

    public float random() {
        ++this.counter;
        return this.random.nextFloat();
    }

    public long randomLong() {
        ++this.counter;
        return this.random.nextLong();
    }
}
```

### Critical Note: INCLUSIVE Ranges

Unlike typical random functions, `random(range)` and `random(start, end)` are **INCLUSIVE**:
- `random(99)` returns 0-99 (100 values)
- `random(0, 99)` returns 0-99 (100 values)
- `random.nextInt(range + 1)` is the underlying call

## RNG Streams

The game uses **13 separate RNG streams**, each initialized from `Settings.seed`:

### Persistent Streams (Saved Across Sessions)

| Stream | Purpose | Counter Saved |
|--------|---------|---------------|
| `monsterRng` | Monster encounter selection, boss shuffle | `monster_seed_count` |
| `eventRng` | Event room outcomes | `event_seed_count` |
| `merchantRng` | Shop prices, sale selection | `merchant_seed_count` |
| `cardRng` | Card reward rarity rolls, upgrades | `card_seed_count` |
| `treasureRng` | Chest type, treasure rewards | `treasure_seed_count` |
| `relicRng` | Relic pool shuffle, tier rolls | `relic_seed_count` |
| `potionRng` | Potion drops, rarity | `potion_seed_count` |

### Floor-Based Streams (Reseeded Each Floor)

These use `Settings.seed + floorNum` as their seed:

| Stream | Purpose |
|--------|---------|
| `monsterHpRng` | Monster HP within ranges |
| `aiRng` | Monster AI decisions (move selection) |
| `shuffleRng` | Deck shuffling in combat |
| `cardRandomRng` | Random card selection during combat |
| `miscRng` | Miscellaneous (event outcomes, etc) |

### Special Streams

| Stream | Seed Formula | Purpose |
|--------|--------------|---------|
| `mapRng` | Act-dependent offset | Map generation |
| `NeowEvent.rng` | `Settings.seed` | Neow blessing selection |

## Seed Initialization

### Game Start (`AbstractDungeon.generateSeeds()`)

```java
public static void generateSeeds() {
    monsterRng = new Random(Settings.seed);
    eventRng = new Random(Settings.seed);
    merchantRng = new Random(Settings.seed);
    cardRng = new Random(Settings.seed);
    treasureRng = new Random(Settings.seed);
    relicRng = new Random(Settings.seed);
    monsterHpRng = new Random(Settings.seed);
    potionRng = new Random(Settings.seed);
    aiRng = new Random(Settings.seed);
    shuffleRng = new Random(Settings.seed);
    cardRandomRng = new Random(Settings.seed);
    miscRng = new Random(Settings.seed);
}
```

### Floor Transition (`AbstractDungeon.nextRoom()`)

```java
// Called when entering a new floor
monsterHpRng = new Random(Settings.seed + (long)floorNum);
aiRng = new Random(Settings.seed + (long)floorNum);
shuffleRng = new Random(Settings.seed + (long)floorNum);
cardRandomRng = new Random(Settings.seed + (long)floorNum);
miscRng = new Random(Settings.seed + (long)floorNum);
```

### Map Generation (Per Act)

```java
// Exordium (Act 1)
mapRng = new Random(Settings.seed + actNum);  // +0

// The City (Act 2)
mapRng = new Random(Settings.seed + actNum * 100);  // +100

// The Beyond (Act 3)
mapRng = new Random(Settings.seed + actNum * 200);  // +200

// The Ending (Act 4)
mapRng = new Random(Settings.seed + actNum * 300);  // +300
```

## Counter System for Save/Load

### Purpose

The counter tracks how many times each RNG has been called. On load, the game:
1. Creates a new Random with the original seed
2. Advances it by calling `random(999)` N times (where N = saved counter)

```java
public Random(Long seed, int counter) {
    this.random = new RandomXS128(seed);
    for (int i = 0; i < counter; ++i) {
        this.random(999);  // Advance state
    }
}
```

### Save File Fields

```java
// From SaveFile
long seed;
int monster_seed_count;
int event_seed_count;
int merchant_seed_count;
int card_seed_count;
int treasure_seed_count;
int relic_seed_count;
int potion_seed_count;
int card_random_seed_randomizer;  // Special: cardBlizzRandomizer
```

### Load Sequence

```java
public static void loadSeeds(SaveFile save) {
    monsterRng = new Random(Settings.seed, save.monster_seed_count);
    eventRng = new Random(Settings.seed, save.event_seed_count);
    merchantRng = new Random(Settings.seed, save.merchant_seed_count);
    cardRng = new Random(Settings.seed, save.card_seed_count);
    cardBlizzRandomizer = save.card_random_seed_randomizer;
    treasureRng = new Random(Settings.seed, save.treasure_seed_count);
    relicRng = new Random(Settings.seed, save.relic_seed_count);
    potionRng = new Random(Settings.seed, save.potion_seed_count);
}
```

## RNG Usage by System

### Monster Selection (`monsterRng`)

```java
// Boss list shuffle
Collections.shuffle(bossList, new Random(monsterRng.randomLong()));

// Monster encounter roll
MonsterInfo.roll(monsters, monsterRng.random());
```

### Card Rewards (`cardRng`)

```java
// Rarity roll with "card blizz" modifier
int roll = cardRng.random(99);
roll += cardBlizzRandomizer;

// Upgrade chance
if (cardRng.randomBoolean(cardUpgradedChance) && c.canUpgrade()) {
    c.upgrade();
}
```

### Card Blizz Randomizer

Special mechanic to prevent rare card floods:
```java
// On act transition, cardRng counter is bumped to next 250 threshold
if (cardRng.counter > 0 && cardRng.counter < 250) {
    cardRng.setCounter(250);
} else if (cardRng.counter > 250 && cardRng.counter < 500) {
    cardRng.setCounter(500);
}
```

### Monster AI (`aiRng`)

```java
// Move selection
if (AbstractDungeon.aiRng.randomBoolean(0.5f)) {
    this.setMove(ATTACK);
} else {
    this.setMove(DEFEND);
}
```

### Monster HP (`monsterHpRng`)

```java
// HP within range
super(NAME, ID, AbstractDungeon.monsterHpRng.random(8, 12), ...);

// Damage roll
biteDamage = AbstractDungeon.monsterHpRng.random(5, 7);
```

### Deck Shuffle (`shuffleRng`)

```java
public void shuffle() {
    Collections.shuffle(this.group,
        new java.util.Random(AbstractDungeon.shuffleRng.randomLong()));
}
```

### Shop Prices (`merchantRng`)

```java
// Card price variation
tmpPrice = basePrice * merchantRng.random(0.9f, 1.1f);

// Sale card selection
saleCard = coloredCards.get(merchantRng.random(0, 4));
```

### Relic Pool (`relicRng`)

```java
// Pool shuffle at game start
Collections.shuffle(commonRelicPool, new Random(relicRng.randomLong()));
Collections.shuffle(uncommonRelicPool, new Random(relicRng.randomLong()));
Collections.shuffle(rareRelicPool, new Random(relicRng.randomLong()));
Collections.shuffle(shopRelicPool, new Random(relicRng.randomLong()));
Collections.shuffle(bossRelicPool, new Random(relicRng.randomLong()));

// Tier roll
int roll = relicRng.random(0, 99);
```

### Event Outcomes (`eventRng`)

```java
float roll = eventRng.random();
// Used in EventHelper.roll() to determine room type
```

### Treasure (`treasureRng`)

```java
// Chest type
int roll = treasureRng.random(0, 99);

// Gold reward
addGoldToRewards(treasureRng.random(25, 35));
```

### Neow Rewards (`NeowEvent.rng`)

```java
// Initialized fresh when Neow blessing starts
rng = new Random(Settings.seed);

// Reward selection
reward = possibleRewards.get(NeowEvent.rng.random(0, size - 1));
```

## Seed String Format

Seeds are displayed as base-35 strings (0-9, A-Z excluding O = 35 characters):

```java
private static final String CHARACTERS = "0123456789ABCDEFGHIJKLMNPQRSTUVWXYZ";
// Note: 10 digits + 25 letters (no O) = 35 characters

// Convert seed to display string
public static String getString(long seed) {
    // Converts to base-35 representation
}

// Convert display string to seed
public static long getLong(String seedStr) {
    // Parses base-35 representation
    // 'O' is converted to '0' to prevent confusion
}
```

**Implementation Note**: The character set is 0-9 (10 chars) + A-Z excluding O (25 chars) = 35 total characters.
Our Python implementation in `core/state/rng.py` correctly uses base-35.

## Python XorShift128+ Implementation

For simulation/analysis:

```python
"""
XorShift128+ PRNG implementation matching libGDX/StS.
Based on Sebastiano Vigna's xorshift128+ algorithm.
"""

class XorShift128Plus:
    """
    XorShift128+ as implemented in libGDX RandomXS128.
    State: two 64-bit integers (s0, s1)
    """

    MASK64 = (1 << 64) - 1

    def __init__(self, seed: int):
        """Initialize with a seed (64-bit signed long)."""
        # Convert signed long to unsigned
        seed = seed & self.MASK64

        # libGDX uses murmur hash to derive initial state
        self.s0 = self._murmur_hash3(seed)
        self.s1 = self._murmur_hash3(self.s0)

        # Ensure non-zero state
        if self.s0 == 0 and self.s1 == 0:
            self.s0 = 1

    def _murmur_hash3(self, x: int) -> int:
        """MurmurHash3 finalizer for state initialization."""
        x = x & self.MASK64
        x ^= x >> 33
        x = (x * 0xff51afd7ed558ccd) & self.MASK64
        x ^= x >> 33
        x = (x * 0xc4ceb9fe1a85ec53) & self.MASK64
        x ^= x >> 33
        return x

    def next_long(self) -> int:
        """Generate next 64-bit value and advance state."""
        s1 = self.s0
        s0 = self.s1
        self.s0 = s0

        s1 ^= (s1 << 23) & self.MASK64
        s1 ^= s1 >> 17
        s1 ^= s0
        s1 ^= s0 >> 26

        self.s1 = s1

        return (self.s0 + self.s1) & self.MASK64

    def next_int(self, bound: int) -> int:
        """Random int in [0, bound) - NOT inclusive like StS wrapper."""
        if bound <= 0:
            raise ValueError("bound must be positive")

        # Use rejection sampling for unbiased results
        n = bound
        if (n & -n) == n:  # Power of 2
            return (self.next_long() & (n - 1))

        bits = self.next_long() & 0x7FFFFFFFFFFFFFFF
        return bits % bound

    def next_float(self) -> float:
        """Random float in [0, 1)."""
        return (self.next_long() & ((1 << 24) - 1)) / (1 << 24)

    def next_double(self) -> float:
        """Random double in [0, 1)."""
        return (self.next_long() & ((1 << 53) - 1)) / (1 << 53)

    def next_boolean(self) -> bool:
        """Random boolean."""
        return (self.next_long() & 1) != 0


class StSRandom:
    """
    Wrapper matching com.megacrit.cardcrawl.random.Random behavior.
    Tracks counter for save/load state restoration.
    """

    def __init__(self, seed: int, counter: int = 0):
        """
        Initialize RNG with seed and optionally replay counter calls.

        Args:
            seed: 64-bit seed (can be negative, will be treated as unsigned)
            counter: Number of RNG calls to replay (for save restoration)
        """
        self._rng = XorShift128Plus(seed)
        self.counter = 0

        # Replay RNG calls to reach saved state
        for _ in range(counter):
            self.random(999)

    def random(self, range_or_end: int, end: int = None) -> int:
        """
        Random int with INCLUSIVE upper bound (StS behavior).

        random(n) -> [0, n] inclusive
        random(start, end) -> [start, end] inclusive
        """
        self.counter += 1

        if end is None:
            # random(range) -> [0, range] inclusive
            return self._rng.next_int(range_or_end + 1)
        else:
            # random(start, end) -> [start, end] inclusive
            start = range_or_end
            return start + self._rng.next_int(end - start + 1)

    def random_boolean(self, chance: float = 0.5) -> bool:
        """Random boolean, optionally with custom probability."""
        self.counter += 1
        if chance == 0.5:
            return self._rng.next_boolean()
        return self._rng.next_float() < chance

    def random_float(self, start: float = None, end: float = None) -> float:
        """
        Random float.

        random_float() -> [0, 1)
        random_float(range) -> [0, range)
        random_float(start, end) -> [start, end)
        """
        self.counter += 1

        if start is None:
            return self._rng.next_float()
        elif end is None:
            return self._rng.next_float() * start
        else:
            return start + self._rng.next_float() * (end - start)

    def random_long(self) -> int:
        """Random 64-bit long."""
        self.counter += 1
        val = self._rng.next_long()
        # Convert to signed
        if val >= (1 << 63):
            val -= (1 << 64)
        return val


# Example usage
if __name__ == "__main__":
    # Simulate game seed
    seed = 12345678

    # Create RNG streams like the game does
    monster_rng = StSRandom(seed)
    card_rng = StSRandom(seed)

    # Floor-based RNG (floor 5)
    floor_num = 5
    ai_rng = StSRandom(seed + floor_num)
    shuffle_rng = StSRandom(seed + floor_num)

    # Simulate some calls
    print(f"Monster roll: {monster_rng.random(99)}")  # 0-99 inclusive
    print(f"Card rarity roll: {card_rng.random(99)}")
    print(f"AI decision: {ai_rng.random_boolean(0.5)}")

    # Check counters
    print(f"Monster RNG counter: {monster_rng.counter}")
    print(f"Card RNG counter: {card_rng.counter}")

    # Restore from save (seed + counter)
    saved_counter = monster_rng.counter
    restored_rng = StSRandom(seed, saved_counter)
    # restored_rng is now at the same state as monster_rng
```

## Key Insights for RL/Simulation

1. **Full Determinism**: Given seed + all counters, entire game is reproducible
2. **Floor-Based Combat RNG**: Combat outcomes only depend on `seed + floorNum` for AI/HP/shuffle
3. **Persistent Run RNG**: Card rewards, relics, monsters are determined by run-wide counters
4. **Neow is Fresh**: Neow RNG starts from seed with counter=0, independent of run
5. **Map is Act-Based**: Each act's map uses different seed offset

### Simulation Strategy

- Track all 13 RNG streams separately
- For combat simulation, only need `aiRng`, `shuffleRng`, `cardRandomRng`, `miscRng`
- For run planning, need all persistent streams
- Can "peek ahead" by copying RNG state and simulating

### Counter Bumping

Card rarity uses a "blizz" system that bumps the counter to thresholds (250, 500, 750) on act transitions. This prevents degenerate rare card sequences.

---

## Implementation Notes

### Shuffle Algorithm (Fisher-Yates)

Java's `Collections.shuffle()` uses Fisher-Yates algorithm, which we replicate exactly:

```python
def shuffle_with_rng(items: list, rng: Random):
    """Shuffle list using game's RNG (matches Collections.shuffle)."""
    # Fisher-Yates shuffle - iterate backwards, swap with random earlier element
    for i in range(len(items) - 1, 0, -1):
        j = rng.random(i)  # [0, i] inclusive - this is critical!
        items[i], items[j] = items[j], items[i]
```

**Critical Detail**: The game uses `rng.random(i)` which returns `[0, i]` inclusive, not `[0, i)`. This matches Java's `Random.nextInt(n+1)` behavior wrapped in StS's Random class.

Our implementation in `core/generation/map.py` correctly uses this approach for room type shuffling.

### Map Seed Offsets

The actual map seed offsets used by the game:

| Act | Offset Formula | Our Implementation |
|-----|----------------|-------------------|
| Act 1 (Exordium) | `seed + 1` | `get_map_seed_offset(1) = 1` |
| Act 2 (The City) | `seed + 200` | `get_map_seed_offset(2) = 200` |
| Act 3 (The Beyond) | `seed + 600` | `get_map_seed_offset(3) = 600` |
| Act 4 (The Ending) | `seed + 1200` | `get_map_seed_offset(4) = 1200` |

Note: Earlier documentation may have shown `actNum * 100/200/300` which was an approximation. The actual offsets are defined in dungeon class constructors.
