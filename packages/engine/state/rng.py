"""
XorShift128 RNG - Exact replication of game's Random class.

The game uses libGDX's RandomXS128 (XorShift128) algorithm.
All RNG in the game is deterministic from the seed.

RNG Streams (from AbstractDungeon):
- monsterRng: Which monsters spawn
- aiRng: Enemy AI decisions (reseeded per floor: seed + floorNum)
- cardRng: Card rewards
- shuffleRng: Deck shuffle order (reseeded per floor)
- relicRng: Relic rewards
- potionRng: Potion drops
- eventRng: Event outcomes
- treasureRng: Chest contents
- merchantRng: Shop contents
- mapRng: Map generation
- monsterHpRng: Enemy HP rolls (reseeded per floor)
- cardRandomRng: Random card effects (reseeded per floor)
- miscRng: Misc effects (reseeded per floor)
"""

from dataclasses import dataclass
from typing import Optional, List
import struct


class XorShift128:
    """
    XorShift128 PRNG - matches libGDX RandomXS128.

    This is the exact algorithm used by Slay the Spire.
    State is two 64-bit integers (seed0, seed1).
    """

    def __init__(self, seed: int, seed1: Optional[int] = None):
        """
        Initialize with a 64-bit seed or explicit (seed0, seed1) state.

        Args:
            seed: Either the initial seed (if seed1 is None) or seed0 state
            seed1: If provided, use (seed, seed1) as direct state values
        """
        if seed1 is not None:
            # Two-argument form: set state directly (used by copy())
            self.seed0 = seed & 0xFFFFFFFFFFFFFFFF
            self.seed1 = seed1 & 0xFFFFFFFFFFFFFFFF
        else:
            # Single-argument form: derive state via murmur hash
            # Java uses Long.MIN_VALUE when seed is 0
            if seed == 0:
                seed = -0x8000000000000000  # Long.MIN_VALUE
            # libGDX RandomXS128 initialization
            self.seed0 = self._murmur_hash3(seed)
            self.seed1 = self._murmur_hash3(self.seed0)

    @staticmethod
    def _murmur_hash3(x: int) -> int:
        """MurmurHash3 finalizer - used for seed initialization."""
        x = x & 0xFFFFFFFFFFFFFFFF  # 64-bit mask
        x ^= x >> 33
        x = (x * 0xff51afd7ed558ccd) & 0xFFFFFFFFFFFFFFFF
        x ^= x >> 33
        x = (x * 0xc4ceb9fe1a85ec53) & 0xFFFFFFFFFFFFFFFF
        x ^= x >> 33
        return x

    def _next_long(self) -> int:
        """Generate next 64-bit value - core XorShift128 algorithm.

        Matches libGDX RandomXS128.nextLong() exactly:
        ```java
        long s1 = seed0;
        final long s0 = seed1;
        seed0 = s0;
        s1 ^= s1 << 23;
        return (seed1 = (s1 ^ s0 ^ (s1 >>> 17) ^ (s0 >>> 26))) + s0;
        ```
        """
        s1 = self.seed0
        s0 = self.seed1
        self.seed0 = s0

        # First XOR with left shift
        s1 ^= (s1 << 23) & 0xFFFFFFFFFFFFFFFF

        # Apply all remaining XORs simultaneously (Java does this in one expression)
        # s1 ^ s0 ^ (s1 >>> 17) ^ (s0 >>> 26)
        self.seed1 = (s1 ^ s0 ^ (s1 >> 17) ^ (s0 >> 26)) & 0xFFFFFFFFFFFFFFFF

        # Result as unsigned 64-bit
        result = (self.seed0 + self.seed1) & 0xFFFFFFFFFFFFFFFF

        # Convert to signed 64-bit (Java's long is signed)
        if result >= 0x8000000000000000:
            result -= 0x10000000000000000

        return result

    def next_int(self, bound: int) -> int:
        """Random int in [0, bound).

        Matches libGDX RandomXS128.nextInt(n) -> nextLong(n):
        ```java
        for (;;) {
            final long bits = nextLong() >>> 1;
            final long value = bits % n;
            if (bits - value + (n - 1) >= 0) return value;
        }
        ```
        """
        if bound <= 0:
            raise ValueError("bound must be positive")

        while True:
            # >>> 1 in Java (unsigned right shift by 1)
            bits = (self._next_long() & 0xFFFFFFFFFFFFFFFF) >> 1
            val = bits % bound
            # Check for bias rejection
            if bits - val + (bound - 1) >= 0:
                return int(val)

    def next_float(self) -> float:
        """Random float in [0, 1).

        Matches libGDX: (nextLong() >>> 40) * NORM_FLOAT
        where NORM_FLOAT = 1.0 / (1L << 24)
        """
        return ((self._next_long() & 0xFFFFFFFFFFFFFFFF) >> 40) / (1 << 24)

    def next_double(self) -> float:
        """Random double in [0, 1).

        Matches libGDX: (nextLong() >>> 11) * NORM_DOUBLE
        where NORM_DOUBLE = 1.0 / (1L << 53)
        """
        return ((self._next_long() & 0xFFFFFFFFFFFFFFFF) >> 11) / (1 << 53)

    def next_boolean(self) -> bool:
        """Random boolean - checks least significant bit."""
        return (self._next_long() & 1) != 0

    def get_state(self, index: int) -> int:
        """Get state value (0 = seed0, 1 = seed1)."""
        if index == 0:
            return self.seed0
        return self.seed1

    def copy(self) -> 'XorShift128':
        """Create a copy with same state.

        Matches Java: new RandomXS128(this.random.getState(0), this.random.getState(1))
        """
        return XorShift128(self.seed0, self.seed1)


class Random:
    """
    Game's Random class wrapper.

    Matches com.megacrit.cardcrawl.random.Random exactly.
    Tracks counter for save/load state restoration.

    Java method signatures matched:
    - random(int range) -> [0, range] inclusive
    - random(int start, int end) -> [start, end] inclusive
    - random(long range) -> [0, range) using nextDouble
    - random(long start, long end) -> [start, end) using nextDouble
    - randomLong() -> raw 64-bit value
    - randomBoolean() -> nextBoolean()
    - randomBoolean(float chance) -> nextFloat() < chance
    - random() -> nextFloat() [0, 1)
    - random(float range) -> nextFloat() * range
    - random(float start, float end) -> start + nextFloat() * (end - start)
    """

    def __init__(self, seed: int, counter: int = 0):
        """
        Initialize RNG with seed and optional counter.

        Args:
            seed: 64-bit seed value
            counter: Number of RNG calls to skip (for save restoration)

        Note: Java constructor Random(Long seed, int counter) advances by
        calling random(999) `counter` times. The counter field is NOT
        set to the input value - it tracks actual calls made.
        """
        self._rng = XorShift128(seed)
        self.counter = 0

        # Skip ahead if counter specified (save restoration)
        # Java: for (int i = 0; i < counter; ++i) { this.random(999); }
        for _ in range(counter):
            self.random_int(999)

    def random_int(self, range_val: int) -> int:
        """
        Random int in [0, range_val] INCLUSIVE.

        Java: public int random(int range) {
            ++this.counter;
            return this.random.nextInt(range + 1);
        }
        """
        self.counter += 1
        return self._rng.next_int(range_val + 1)

    def random_int_range(self, start: int, end: int) -> int:
        """
        Random int in [start, end] INCLUSIVE.

        Java: public int random(int start, int end) {
            ++this.counter;
            return start + this.random.nextInt(end - start + 1);
        }
        """
        self.counter += 1
        return start + self._rng.next_int(end - start + 1)

    def random_long_range(self, range_val: int) -> int:
        """
        Random long in [0, range_val) using nextDouble.

        Java: public long random(long range) {
            ++this.counter;
            return (long)(this.random.nextDouble() * (double)range);
        }

        Note: This is NOT the same as random_int - uses nextDouble not nextInt!
        """
        self.counter += 1
        return int(self._rng.next_double() * range_val)

    def random_long_start_end(self, start: int, end: int) -> int:
        """
        Random long in [start, end) using nextDouble.

        Java: public long random(long start, long end) {
            ++this.counter;
            return start + (long)(this.random.nextDouble() * (double)(end - start));
        }
        """
        self.counter += 1
        return start + int(self._rng.next_double() * (end - start))

    def random_long(self) -> int:
        """
        Random 64-bit integer (raw nextLong).

        Java: public long randomLong() {
            ++this.counter;
            return this.random.nextLong();
        }
        """
        self.counter += 1
        return self._rng._next_long()

    def random_boolean(self, chance: float = None) -> bool:
        """
        Random boolean.

        With no argument (or None): uses nextBoolean() (50% chance)
        With float argument: uses nextFloat() < chance

        Java overloads:
        - public boolean randomBoolean() { return this.random.nextBoolean(); }
        - public boolean randomBoolean(float chance) { return this.random.nextFloat() < chance; }
        """
        self.counter += 1
        if chance is None:
            return self._rng.next_boolean()
        return self._rng.next_float() < chance

    def random_boolean_chance(self, chance: float) -> bool:
        """
        Random boolean with given chance of True.

        Explicit name for the float-argument version.

        Java: public boolean randomBoolean(float chance) {
            ++this.counter;
            return this.random.nextFloat() < chance;
        }
        """
        self.counter += 1
        return self._rng.next_float() < chance

    def random_float(self) -> float:
        """
        Random float in [0, 1).

        Java: public float random() {
            ++this.counter;
            return this.random.nextFloat();
        }
        """
        self.counter += 1
        return self._rng.next_float()

    def random_float_max(self, range_val: float) -> float:
        """
        Random float in [0, range_val).

        Java: public float random(float range) {
            ++this.counter;
            return this.random.nextFloat() * range;
        }
        """
        self.counter += 1
        return self._rng.next_float() * range_val

    def random_float_range(self, start: float, end: float) -> float:
        """
        Random float in [start, end).

        Java: public float random(float start, float end) {
            ++this.counter;
            return start + this.random.nextFloat() * (end - start);
        }
        """
        self.counter += 1
        return start + self._rng.next_float() * (end - start)

    def set_counter(self, target_counter: int) -> None:
        """
        Advance RNG to reach target counter.

        Java: public void setCounter(int targetCounter) {
            if (this.counter < targetCounter) {
                int count = targetCounter - this.counter;
                for (int i = 0; i < count; ++i) {
                    this.randomBoolean();
                }
            } else {
                logger.info("ERROR: Counter is already higher than target counter!");
            }
        }
        """
        if self.counter < target_counter:
            count = target_counter - self.counter
            for _ in range(count):
                self.random_boolean()
        # Java logs error if counter >= target, but we silently ignore

    def copy(self) -> 'Random':
        """
        Create a copy with same state.

        Java: public Random copy() {
            Random copied = new Random();
            copied.random = new RandomXS128(this.random.getState(0), this.random.getState(1));
            copied.counter = this.counter;
            return copied;
        }
        """
        new = Random.__new__(Random)
        new._rng = XorShift128(
            self._rng.get_state(0),
            self._rng.get_state(1)
        )
        new.counter = self.counter
        return new

    # ============ ALIASES FOR BACKWARD COMPATIBILITY ============
    # These match the method names used in the rest of the codebase

    def random(self, range_val: int) -> int:
        """Alias for random_int - matches existing codebase usage."""
        return self.random_int(range_val)

    def random_range(self, start: int, end: int) -> int:
        """Alias for random_int_range - matches existing codebase usage."""
        return self.random_int_range(start, end)


def seed_to_long(seed_string: str) -> int:
    """
    Convert seed string (e.g., "ABC123XYZ") to long value.

    The game uses base-35 encoding: 0-9 + A-Z excluding O.
    O is automatically replaced with 0 when parsing.

    If the seed is already a pure numeric string (from save file),
    it's treated as a plain integer, not base-35.
    """
    # If seed is numeric (including negative), it's already the numeric form (from save file)
    # Java stores seeds as signed longs, so negative values are valid
    if seed_string.lstrip('-').isdigit():
        return int(seed_string)

    # Game's exact character set from SeedHelper.java
    CHARACTERS = "0123456789ABCDEFGHIJKLMNPQRSTUVWXYZ"

    # Normalize: uppercase and replace O with 0 (game does this)
    seed_string = seed_string.upper().replace("O", "0")

    result = 0
    for char in seed_string:
        remainder = CHARACTERS.find(char)
        if remainder == -1:
            continue  # Skip invalid characters
        result *= len(CHARACTERS)  # 35
        result += remainder

    return result


def long_to_seed(seed_long: int) -> str:
    """
    Convert long value back to seed string.

    Matches SeedHelper.getString() from game.
    """
    CHARACTERS = "0123456789ABCDEFGHIJKLMNPQRSTUVWXYZ"
    char_count = len(CHARACTERS)  # 35

    if seed_long == 0:
        return "0"

    # Handle as unsigned 64-bit
    leftover = seed_long & 0xFFFFFFFFFFFFFFFF

    result = []
    while leftover != 0:
        remainder = leftover % char_count
        leftover = leftover // char_count
        result.append(CHARACTERS[remainder])

    return ''.join(reversed(result))


@dataclass
class GameRNG:
    """
    All RNG streams for a game run.

    Matches AbstractDungeon's RNG fields.
    """
    seed: int
    floor: int = 0

    # Persistent streams (seeded once at game start)
    monster_rng: Random = None
    map_rng: Random = None
    event_rng: Random = None
    merchant_rng: Random = None
    card_rng: Random = None
    treasure_rng: Random = None
    relic_rng: Random = None
    potion_rng: Random = None

    # Per-floor streams (reseeded each floor with seed + floorNum)
    monster_hp_rng: Random = None
    ai_rng: Random = None
    shuffle_rng: Random = None
    card_random_rng: Random = None
    misc_rng: Random = None

    def __post_init__(self):
        """Initialize all RNG streams from seed."""
        self._init_persistent_streams()
        self._init_floor_streams()

    def _init_persistent_streams(self):
        """Initialize streams that persist across floors."""
        self.monster_rng = Random(self.seed)
        self.map_rng = Random(self.seed)
        self.event_rng = Random(self.seed)
        self.merchant_rng = Random(self.seed)
        self.card_rng = Random(self.seed)
        self.treasure_rng = Random(self.seed)
        self.relic_rng = Random(self.seed)
        self.potion_rng = Random(self.seed)

    def _init_floor_streams(self):
        """Initialize/reseed per-floor streams."""
        floor_seed = self.seed + self.floor
        self.monster_hp_rng = Random(floor_seed)
        self.ai_rng = Random(floor_seed)
        self.shuffle_rng = Random(floor_seed)
        self.card_random_rng = Random(floor_seed)
        self.misc_rng = Random(floor_seed)

    def advance_floor(self):
        """Called when entering a new floor."""
        self.floor += 1
        self._init_floor_streams()

    def get_counters(self) -> dict:
        """Get all counter values for save state."""
        return {
            "monster_seed_count": self.monster_rng.counter,
            "event_seed_count": self.event_rng.counter,
            "merchant_seed_count": self.merchant_rng.counter,
            "card_seed_count": self.card_rng.counter,
            "treasure_seed_count": self.treasure_rng.counter,
            "relic_seed_count": self.relic_rng.counter,
            "potion_seed_count": self.potion_rng.counter,
        }

    @classmethod
    def from_save(cls, seed: int, counters: dict, floor: int) -> 'GameRNG':
        """Restore RNG state from save data."""
        rng = cls(seed=seed, floor=floor)

        # Restore persistent stream counters
        rng.monster_rng = Random(seed, counters.get("monster_seed_count", 0))
        rng.event_rng = Random(seed, counters.get("event_seed_count", 0))
        rng.merchant_rng = Random(seed, counters.get("merchant_seed_count", 0))
        rng.card_rng = Random(seed, counters.get("card_seed_count", 0))
        rng.treasure_rng = Random(seed, counters.get("treasure_seed_count", 0))
        rng.relic_rng = Random(seed, counters.get("relic_seed_count", 0))
        rng.potion_rng = Random(seed, counters.get("potion_seed_count", 0))

        return rng


# ============ TESTING ============

if __name__ == "__main__":
    # Test seed conversion
    seed_str = "4YUHY81W7GRHT"
    seed_long = seed_to_long(seed_str)
    print(f"Seed '{seed_str}' -> {seed_long}")

    # Test RNG
    rng = Random(seed_long)
    print(f"\nFirst 10 random_int(99) values:")
    for i in range(10):
        print(f"  {i}: {rng.random_int(99)}")

    # Test GameRNG
    game = GameRNG(seed=seed_long)
    print(f"\nAI RNG first move roll (floor 1): {game.ai_rng.random_int(99)}")

    game.advance_floor()
    print(f"AI RNG first move roll (floor 2): {game.ai_rng.random_int(99)}")
