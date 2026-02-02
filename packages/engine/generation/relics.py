"""
Slay the Spire - Relic Pool Generation and Prediction

Replicates the exact relic pool initialization and shuffling from the game.

Key mechanics:
1. Relic pools are populated from HashMaps containing ALL relics (not just target tier)
2. relicRng is seeded with Settings.seed
3. Each pool (common, uncommon, rare, shop, boss) is shuffled in order
4. Shuffle uses Collections.shuffle(pool, new java.util.Random(relicRng.randomLong()))
5. Boss swap takes bossRelicPool.remove(0) - the first relic after shuffle

CRITICAL: The HashMap iteration order depends on ALL relics in the map, not just
the tier being extracted. This is because populateRelicPool iterates over the
full HashMap and filters by tier.

RNG Stream Usage:
- relicRng.randomLong() called for each pool shuffle (5 total for initialization)
- relicRng.random(0, 99) used for relic tier roll chances

Pool Consumption (Java Parity):
- returnRandomRelicKey: removes from FRONT (index 0) - FIFO
- returnEndRandomRelicKey: removes from END (index -1) - LIFO
- Boss relics ALWAYS remove from front (index 0)

Cascade Logic (when pool empty):
- COMMON -> tries UNCOMMON -> tries RARE -> returns "Circlet"
- UNCOMMON -> tries RARE -> returns "Circlet"
- RARE -> returns "Circlet"
- SHOP -> tries UNCOMMON (no further cascade in Java)
- BOSS -> returns "Red Circlet"

Tier Roll Chances (Acts 1-3):
- 0-49: COMMON (50%)
- 50-82: UNCOMMON (33%)
- 83-99: RARE (17%)

Act 4 (The Ending):
- 0-99: UNCOMMON (100%) - no commons in Act 4
"""

import os
import importlib.util
from dataclasses import dataclass, field
from typing import List, Dict, Optional, Tuple
from copy import deepcopy

# Load modules
_core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def _load_module(name: str, filepath: str):
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

_rng_mod = _load_module("rng", os.path.join(_core_dir, "state", "rng.py"))
_hashmap_mod = _load_module("java_hashmap", os.path.join(_core_dir, "utils", "java_hashmap.py"))


# ============================================================================
# ALL SHARED RELICS (in RelicLibrary.initialize() order)
# Format: (relic_id, tier)
# ============================================================================

SHARED_RELICS = [
    ("Abacus", "UNCOMMON"),
    ("Akabeko", "COMMON"),
    ("Anchor", "COMMON"),
    ("Ancient Tea Set", "COMMON"),
    ("Art of War", "UNCOMMON"),
    ("Astrolabe", "BOSS"),
    ("Bag of Marbles", "COMMON"),
    ("Bag of Preparation", "UNCOMMON"),
    ("Bird Faced Urn", "RARE"),
    ("Black Star", "BOSS"),
    ("Blood Vial", "COMMON"),
    ("Bloody Idol", "RARE"),
    ("Blue Candle", "UNCOMMON"),
    ("The Boot", "COMMON"),
    ("Bottled Flame", "UNCOMMON"),
    ("Bottled Lightning", "UNCOMMON"),
    ("Bottled Tornado", "UNCOMMON"),
    ("Bronze Scales", "COMMON"),
    ("Busted Crown", "BOSS"),
    ("Calipers", "RARE"),
    ("Calling Bell", "BOSS"),
    ("CaptainsWheel", "RARE"),
    ("Cauldron", "SHOP"),
    ("Centennial Puzzle", "UNCOMMON"),
    ("Ceramic Fish", "COMMON"),
    ("Chemical X", "RARE"),
    ("Clockwork Souvenir", "SHOP"),
    ("Coffee Dripper", "BOSS"),
    ("Courier", "SHOP"),
    ("CultistMask", "SPECIAL"),
    ("Cursed Key", "BOSS"),
    ("Darkstone Periapt", "UNCOMMON"),
    ("Dead Branch", "RARE"),
    ("Dollys Mirror", "SHOP"),
    ("Dream Catcher", "COMMON"),
    ("Du-Vu Doll", "RARE"),
    ("Ectoplasm", "BOSS"),
    ("Empty Cage", "BOSS"),
    ("Enchiridion", "SPECIAL"),
    ("Eternal Feather", "UNCOMMON"),
    ("FaceOfCleric", "RARE"),
    ("Fossilized Helix", "RARE"),
    ("Frozen Egg 2", "UNCOMMON"),
    ("Frozen Eye", "SHOP"),
    ("Fusion Hammer", "BOSS"),
    ("Gambling Chip", "UNCOMMON"),
    ("Ginger", "RARE"),
    ("Girya", "RARE"),
    ("Golden Idol", "COMMON"),
    ("Gremlin Horn", "UNCOMMON"),
    ("GremlinMask", "SPECIAL"),
    ("HandDrill", "UNCOMMON"),
    ("Happy Flower", "COMMON"),
    ("HornCleat", "COMMON"),
    ("Ice Cream", "RARE"),
    ("Incense Burner", "RARE"),
    ("Ink Bottle", "UNCOMMON"),
    ("Juzu Bracelet", "UNCOMMON"),
    ("Kunai", "UNCOMMON"),
    ("Lantern", "COMMON"),
    ("Letter Opener", "UNCOMMON"),
    ("Lizard Tail", "RARE"),
    ("Mango", "RARE"),
    ("Mark of the Bloom", "SPECIAL"),
    ("Matryoshka", "RARE"),
    ("Maw Bank", "COMMON"),
    ("Meal Ticket", "COMMON"),
    ("Meat on the Bone", "UNCOMMON"),
    ("Medical Kit", "RARE"),
    ("Membership Card", "SHOP"),
    ("Mercury Hourglass", "UNCOMMON"),
    ("Molten Egg 2", "UNCOMMON"),
    ("Mummified Hand", "RARE"),
    ("MutagenicStrength", "SPECIAL"),
    ("Necronomicon", "RARE"),
    ("NeowsBlessing", "SPECIAL"),
    ("Nilrys Codex", "RARE"),
    ("Nloth's Gift", "SPECIAL"),
    ("N'loth's Hungry Face", "SPECIAL"),
    ("Nunchaku", "UNCOMMON"),
    ("Oddly Smooth Stone", "COMMON"),
    ("Odd Mushroom", "UNCOMMON"),
    ("Old Coin", "COMMON"),
    ("Omamori", "RARE"),
    ("Orange Pellets", "RARE"),
    ("Orichalcum", "COMMON"),
    ("Ornamental Fan", "UNCOMMON"),
    ("Orrery", "SHOP"),
    ("Pandora's Box", "BOSS"),
    ("Pantograph", "RARE"),
    ("Peace Pipe", "RARE"),
    ("Pear", "RARE"),
    ("Pen Nib", "COMMON"),
    ("Philosopher's Stone", "BOSS"),
    ("Pocketwatch", "UNCOMMON"),
    ("Potion Belt", "COMMON"),
    ("Prayer Wheel", "RARE"),
    ("PreservedInsect", "UNCOMMON"),
    ("PrismaticShard", "SPECIAL"),
    ("Question Card", "SHOP"),
    ("Red Mask", "SPECIAL"),
    ("Regal Pillow", "COMMON"),
    ("Runic Dome", "BOSS"),
    ("Runic Pyramid", "BOSS"),
    ("SacredBark", "BOSS"),
    ("Shovel", "RARE"),
    ("Shuriken", "UNCOMMON"),
    ("Singing Bowl", "RARE"),
    ("SlaversCollar", "BOSS"),
    ("Sling", "COMMON"),
    ("Smiling Mask", "COMMON"),
    ("Snecko Eye", "BOSS"),
    ("Sozu", "BOSS"),
    ("Spirit Poop", "SPECIAL"),
    ("SsserpentHead", "SPECIAL"),
    ("StoneCalendar", "RARE"),
    ("Strange Spoon", "RARE"),
    ("Strawberry", "RARE"),
    ("StrikeDummy", "SPECIAL"),
    ("Sundial", "UNCOMMON"),
    ("Thread and Needle", "UNCOMMON"),
    ("Tiny Chest", "UNCOMMON"),
    ("Tiny House", "BOSS"),
    ("Toolbox", "RARE"),
    ("Torii", "UNCOMMON"),
    ("Toxic Egg 2", "UNCOMMON"),
    ("Toy Ornithopter", "COMMON"),
    ("Tungsten Rod", "RARE"),
    ("Turnip", "RARE"),
    ("Unceasing Top", "UNCOMMON"),
    ("Vajra", "COMMON"),
    ("Velvet Choker", "BOSS"),
    ("Waffle", "RARE"),
    ("War Paint", "UNCOMMON"),
    ("WarpedTongs", "UNCOMMON"),
    ("Whetstone", "UNCOMMON"),
    ("White Beast Statue", "RARE"),
    ("Wing Boots", "RARE"),
]

# Class-specific relics (in RelicLibrary.initialize() order)
CLASS_RELICS = {
    "WATCHER": [
        ("CloakClasp", "RARE"),
        ("Damaru", "COMMON"),
        ("GoldenEye", "RARE"),
        ("HolyWater", "BOSS"),
        ("Melange", "SHOP"),
        ("PureWater", "STARTER"),
        ("VioletLotus", "BOSS"),
        ("TeardropLocket", "UNCOMMON"),
        ("Yang", "UNCOMMON"),
    ],
    "IRONCLAD": [
        ("Black Blood", "BOSS"),
        ("Brimstone", "SHOP"),
        ("Burning Blood", "STARTER"),
        ("Champion Belt", "RARE"),
        ("Charon's Ashes", "RARE"),
        ("Magic Flower", "RARE"),
        ("Mark of Pain", "BOSS"),
        ("Paper Frog", "UNCOMMON"),
        ("Red Skull", "COMMON"),
        ("Runic Cube", "BOSS"),
        ("Self Forming Clay", "UNCOMMON"),
    ],
    "SILENT": [
        ("HoveringKite", "BOSS"),
        ("Ninja Scroll", "UNCOMMON"),
        ("Paper Crane", "UNCOMMON"),
        ("Ring of the Serpent", "BOSS"),
        ("Ring of the Snake", "STARTER"),
        ("Snake Skull", "COMMON"),
        ("The Specimen", "RARE"),
        ("Tingsha", "RARE"),
        ("Tough Bandages", "RARE"),
        ("TwistedFunnel", "SHOP"),
        ("WristBlade", "BOSS"),
    ],
    "DEFECT": [
        ("Cracked Core", "STARTER"),
        ("DataDisk", "COMMON"),
        ("Emotion Chip", "RARE"),
        ("FrozenCore", "BOSS"),
        ("Cables", "UNCOMMON"),
        ("Inserter", "BOSS"),
        ("Nuclear Battery", "BOSS"),
        ("Runic Capacitor", "SHOP"),
        ("Symbiotic Virus", "UNCOMMON"),
    ],
}


# ============================================================================
# JAVA COLLECTIONS.SHUFFLE IMPLEMENTATION
# ============================================================================

def java_collections_shuffle(lst: List, java_random_seed: int) -> List:
    """
    Replicate Java's Collections.shuffle() with a seeded java.util.Random.

    From Collections.java:
        public static void shuffle(List<?> list, Random rnd) {
            int size = list.size();
            for (int i=size; i>1; i--)
                swap(list, i-1, rnd.nextInt(i));
        }

    Args:
        lst: List to shuffle (will be modified in place)
        java_random_seed: Seed for java.util.Random (signed 64-bit)

    Returns:
        The shuffled list (same reference as input)
    """
    # Initialize Java Random state
    # Java: seed = (seed ^ 0x5DEECE66DL) & ((1L << 48) - 1)
    MULTIPLIER = 0x5DEECE66D
    ADDEND = 0xB
    MASK = (1 << 48) - 1

    seed = (java_random_seed ^ MULTIPLIER) & MASK

    def next_bits(bits: int) -> int:
        nonlocal seed
        seed = (seed * MULTIPLIER + ADDEND) & MASK
        return seed >> (48 - bits)

    def next_int(n: int) -> int:
        """Java's Random.nextInt(n) - returns random int in [0, n)"""
        if n <= 0:
            raise ValueError("n must be positive")

        # Special case for powers of 2
        if (n & (n - 1)) == 0:
            return (n * next_bits(31)) >> 31

        # General case with rejection sampling
        while True:
            bits = next_bits(31)
            val = bits % n
            if bits - val + (n - 1) >= 0:
                return val

    # Fisher-Yates shuffle (Java's algorithm)
    size = len(lst)
    for i in range(size, 1, -1):
        j = next_int(i)
        lst[i - 1], lst[j] = lst[j], lst[i - 1]

    return lst


# ============================================================================
# RELIC POOL GENERATION
# ============================================================================

def _build_full_hashmaps(player_class: str = "WATCHER"):
    """
    Build the full relic HashMaps as the game does.

    Returns:
        (shared_hashmap, class_hashmap)
    """
    # Build sharedRelics HashMap (ALL shared relics)
    shared_map = _hashmap_mod.JavaHashMap(16)
    for relic_id, tier in SHARED_RELICS:
        shared_map.put(relic_id, tier)

    # Build class-specific HashMap
    class_map = _hashmap_mod.JavaHashMap(16)
    class_relics = CLASS_RELICS.get(player_class, [])
    for relic_id, tier in class_relics:
        class_map.put(relic_id, tier)

    return shared_map, class_map


def get_boss_relic_pool_order(player_class: str = "WATCHER") -> List[str]:
    """
    Get boss relic pool in the correct iteration order.

    This simulates populateRelicPool() which:
    1. Iterates sharedRelics HashMap
    2. Iterates class-specific HashMap
    3. Filters to BOSS tier only

    The iteration order depends on the FULL HashMap contents,
    not just the boss relics.

    Args:
        player_class: Player class name

    Returns:
        List of boss relic IDs in iteration order
    """
    shared_map, class_map = _build_full_hashmaps(player_class)

    boss_pool = []

    # From sharedRelics
    for relic_id, tier in shared_map.items_in_iteration_order():
        if tier == "BOSS":
            boss_pool.append(relic_id)

    # From class-specific relics
    for relic_id, tier in class_map.items_in_iteration_order():
        if tier == "BOSS":
            boss_pool.append(relic_id)

    return boss_pool


def predict_boss_relic_pool(seed: int, player_class: str = "WATCHER") -> List[str]:
    """
    Predict the shuffled boss relic pool for a seed.

    Process:
    1. relicRng initialized with seed
    2. randomLong() called 4 times (common, uncommon, rare, shop shuffles)
    3. Boss pool shuffle uses 5th randomLong() for seed

    Args:
        seed: The game seed (long value)
        player_class: Player class name

    Returns:
        Shuffled boss relic pool (position 0 is what Neow swap gives)
    """
    # Get pool in correct iteration order
    pool = get_boss_relic_pool_order(player_class)

    # Create relicRng
    relic_rng = _rng_mod.Random(seed)

    # Advance RNG for prior pool shuffles:
    # 1. commonRelicPool shuffle
    # 2. uncommonRelicPool shuffle
    # 3. rareRelicPool shuffle
    # 4. shopRelicPool shuffle
    for _ in range(4):
        relic_rng.random_long()

    # Get boss pool shuffle seed
    boss_shuffle_seed = relic_rng.random_long()

    # Shuffle the pool
    shuffled = java_collections_shuffle(pool.copy(), boss_shuffle_seed)

    return shuffled


def predict_neow_boss_swap(seed: int, player_class: str = "WATCHER") -> str:
    """
    Predict which boss relic Neow's boss swap would give.

    Args:
        seed: The game seed (long value)
        player_class: Player class name

    Returns:
        The boss relic ID that would be received
    """
    pool = predict_boss_relic_pool(seed, player_class)
    return pool[0] if pool else "Red Circlet"


# ============================================================================
# ALL RELIC POOL PREDICTIONS
# ============================================================================

def get_relic_pool_order(tier: str, player_class: str = "WATCHER") -> List[str]:
    """
    Get relic pool for a specific tier in the correct iteration order.

    Args:
        tier: COMMON, UNCOMMON, RARE, SHOP, or BOSS
        player_class: Player class name

    Returns:
        List of relic IDs in iteration order
    """
    shared_map, class_map = _build_full_hashmaps(player_class)

    pool = []

    # From sharedRelics
    for relic_id, relic_tier in shared_map.items_in_iteration_order():
        if relic_tier == tier:
            pool.append(relic_id)

    # From class-specific relics
    for relic_id, relic_tier in class_map.items_in_iteration_order():
        if relic_tier == tier:
            pool.append(relic_id)

    return pool


@dataclass
class RelicPools:
    """All shuffled relic pools for a seed."""
    common: List[str]
    uncommon: List[str]
    rare: List[str]
    shop: List[str]
    boss: List[str]


def predict_all_relic_pools(seed: int, player_class: str = "WATCHER") -> RelicPools:
    """
    Predict all shuffled relic pools for a seed.

    Process (from initializeRelicList):
    1. relicRng initialized with seed
    2. Shuffle common pool with randomLong()
    3. Shuffle uncommon pool with randomLong()
    4. Shuffle rare pool with randomLong()
    5. Shuffle shop pool with randomLong()
    6. Shuffle boss pool with randomLong()

    Args:
        seed: The game seed (long value)
        player_class: Player class name

    Returns:
        RelicPools with all shuffled pools
    """
    # Create relicRng
    relic_rng = _rng_mod.Random(seed)

    # Get pools in iteration order and shuffle each
    common_pool = get_relic_pool_order("COMMON", player_class)
    common_seed = relic_rng.random_long()
    common_shuffled = java_collections_shuffle(common_pool.copy(), common_seed)

    uncommon_pool = get_relic_pool_order("UNCOMMON", player_class)
    uncommon_seed = relic_rng.random_long()
    uncommon_shuffled = java_collections_shuffle(uncommon_pool.copy(), uncommon_seed)

    rare_pool = get_relic_pool_order("RARE", player_class)
    rare_seed = relic_rng.random_long()
    rare_shuffled = java_collections_shuffle(rare_pool.copy(), rare_seed)

    shop_pool = get_relic_pool_order("SHOP", player_class)
    shop_seed = relic_rng.random_long()
    shop_shuffled = java_collections_shuffle(shop_pool.copy(), shop_seed)

    boss_pool = get_relic_pool_order("BOSS", player_class)
    boss_seed = relic_rng.random_long()
    boss_shuffled = java_collections_shuffle(boss_pool.copy(), boss_seed)

    return RelicPools(
        common=common_shuffled,
        uncommon=uncommon_shuffled,
        rare=rare_shuffled,
        shop=shop_shuffled,
        boss=boss_shuffled,
    )


# ============================================================================
# MUTABLE RELIC POOL STATE (for tracking pool consumption through a run)
# ============================================================================

# Relics that returnRandomScreenlessRelic skips (require card selection screen)
SCREENLESS_BLOCKED = {"Bottled Flame", "Bottled Lightning", "Bottled Tornado", "Whetstone"}

# Relics that returnRandomNonCampfireRelic skips (campfire-related)
NON_CAMPFIRE_BLOCKED = {"Peace Pipe", "Shovel", "Girya"}


# ============================================================================
# RELIC canSpawn() VALIDATION
# ============================================================================
# Some relics have canSpawn() checks that prevent them from appearing unless
# certain conditions are met. These checks happen AFTER the relic is removed
# from the pool - if canSpawn() returns false, the game recursively tries
# to get another relic.

# Boss relics with canSpawn() conditions:
# - Ectoplasm: only spawns in Act 1 (actNum <= 1)
# - Black Blood: requires Burning Blood starter (Ironclad only)
# - Ring of the Serpent: requires Ring of the Snake starter (Silent only)
# - Frozen Core: requires Cracked Core starter (Defect only)
# - Holy Water: requires Pure Water starter (Watcher only)

# Map of starter relics to their boss upgrade relics
STARTER_TO_BOSS_RELIC = {
    "Burning Blood": "Black Blood",       # Ironclad
    "Ring of the Snake": "Ring of the Serpent",  # Silent
    "Cracked Core": "FrozenCore",         # Defect
    "PureWater": "HolyWater",             # Watcher
}

# Reverse map: boss relic -> required starter relic
BOSS_RELIC_REQUIRES_STARTER = {v: k for k, v in STARTER_TO_BOSS_RELIC.items()}


def can_boss_relic_spawn(relic_id: str, act_num: int = 1,
                          owned_relics: Optional[List[str]] = None) -> bool:
    """
    Check if a boss relic can spawn based on game conditions.

    This replicates the canSpawn() method logic from Java for boss relics.

    Args:
        relic_id: The boss relic ID to check
        act_num: Current act number (1-indexed, so Act 1 = 1)
        owned_relics: List of relics the player currently owns

    Returns:
        True if the relic can spawn, False otherwise
    """
    owned = set(owned_relics) if owned_relics else set()

    # Ectoplasm: only spawns in Act 1 (actNum <= 1 in Java, which means Act 1)
    if relic_id == "Ectoplasm":
        return act_num <= 1

    # Class-specific boss relics require their starter relic
    if relic_id in BOSS_RELIC_REQUIRES_STARTER:
        required_starter = BOSS_RELIC_REQUIRES_STARTER[relic_id]
        return required_starter in owned

    # All other boss relics can always spawn
    return True


@dataclass
class RelicPoolState:
    """
    Mutable relic pool state that tracks consumption through a run.

    This replicates the game's actual pool state with proper FIFO/LIFO removal.

    Java Behavior:
    - returnRandomRelicKey: removes from FRONT (index 0) - used for most relic drops
    - returnEndRandomRelicKey: removes from END (index -1) - rarely used
    - Boss relics ALWAYS remove from front (index 0)

    Cascade Logic (when pool empty):
    - COMMON -> UNCOMMON -> RARE -> "Circlet"
    - UNCOMMON -> RARE -> "Circlet"
    - RARE -> "Circlet"
    - SHOP -> UNCOMMON (no further cascade)
    - BOSS -> "Red Circlet"
    """
    common: List[str] = field(default_factory=list)
    uncommon: List[str] = field(default_factory=list)
    rare: List[str] = field(default_factory=list)
    shop: List[str] = field(default_factory=list)
    boss: List[str] = field(default_factory=list)

    # Track relics already owned (for canSpawn checks)
    owned_relics: List[str] = field(default_factory=list)

    # Current act number (1-indexed, used for canSpawn checks like Ectoplasm)
    act_num: int = 1

    def copy(self) -> 'RelicPoolState':
        """Create a deep copy of the pool state."""
        return RelicPoolState(
            common=self.common.copy(),
            uncommon=self.uncommon.copy(),
            rare=self.rare.copy(),
            shop=self.shop.copy(),
            boss=self.boss.copy(),
            owned_relics=self.owned_relics.copy(),
            act_num=self.act_num,
        )

    def _get_pool(self, tier: str) -> List[str]:
        """Get the pool list for a tier."""
        tier = tier.upper()
        if tier == "COMMON":
            return self.common
        elif tier == "UNCOMMON":
            return self.uncommon
        elif tier == "RARE":
            return self.rare
        elif tier == "SHOP":
            return self.shop
        elif tier == "BOSS":
            return self.boss
        else:
            raise ValueError(f"Unknown tier: {tier}")

    def take_from_front(self, tier: str) -> Optional[str]:
        """
        Take a relic from the front of a pool (FIFO).

        This matches returnRandomRelicKey behavior.
        Returns None if pool is empty.
        """
        pool = self._get_pool(tier)
        if pool:
            return pool.pop(0)
        return None

    def take_from_end(self, tier: str) -> Optional[str]:
        """
        Take a relic from the end of a pool (LIFO).

        This matches returnEndRandomRelicKey behavior.
        Returns None if pool is empty.
        """
        pool = self._get_pool(tier)
        if pool:
            return pool.pop(-1)
        return None

    def remove_relic(self, relic_id: str) -> bool:
        """
        Remove a specific relic from all pools.

        Used when player already owns a relic (relicsToRemoveOnStart logic).
        Returns True if relic was found and removed.
        """
        for pool in [self.common, self.uncommon, self.rare, self.shop, self.boss]:
            if relic_id in pool:
                pool.remove(relic_id)
                return True
        return False

    def mark_owned(self, relic_id: str):
        """Mark a relic as owned and remove from pools."""
        if relic_id not in self.owned_relics:
            self.owned_relics.append(relic_id)
        self.remove_relic(relic_id)


def create_relic_pool_state(seed: int, player_class: str = "WATCHER",
                            owned_relics: Optional[List[str]] = None,
                            act_num: int = 1) -> RelicPoolState:
    """
    Create a RelicPoolState initialized for a seed.

    This is the main entry point for tracking relic pools through a run.

    Args:
        seed: The game seed (long value)
        player_class: Player class name
        owned_relics: List of relics already owned (will be removed from pools)
        act_num: Current act number (1-indexed, used for canSpawn checks)

    Returns:
        Mutable RelicPoolState ready for consumption tracking
    """
    pools = predict_all_relic_pools(seed, player_class)
    state = RelicPoolState(
        common=pools.common.copy(),
        uncommon=pools.uncommon.copy(),
        rare=pools.rare.copy(),
        shop=pools.shop.copy(),
        boss=pools.boss.copy(),
        act_num=act_num,
    )

    # Remove already-owned relics from pools
    if owned_relics:
        for relic in owned_relics:
            state.mark_owned(relic)

    return state


def restore_relic_pool_state_from_counters(
    seed: int,
    player_class: str,
    common_consumed: int,
    uncommon_consumed: int,
    rare_consumed: int,
    shop_consumed: int,
    boss_consumed: int,
    owned_relics: Optional[List[str]] = None,
    act_num: int = 1
) -> RelicPoolState:
    """
    Restore pool state by simulating N relics consumed from each pool.

    This is useful when you know how many relics have been taken from each
    pool but don't have the full list of consumed relics.

    IMPORTANT: This assumes relics were consumed in FIFO order (front of pool).
    If any screenless/filter logic was used, this won't be perfectly accurate.

    Args:
        seed: The game seed
        player_class: Player class name
        common_consumed: Number of common relics already taken
        uncommon_consumed: Number of uncommon relics already taken
        rare_consumed: Number of rare relics already taken
        shop_consumed: Number of shop relics already taken
        boss_consumed: Number of boss relics already taken
        owned_relics: Optional list of owned relics (removes from pools)
        act_num: Current act number (1-indexed, used for canSpawn checks)

    Returns:
        RelicPoolState with appropriate relics already consumed
    """
    state = create_relic_pool_state(seed, player_class, owned_relics, act_num)

    # Consume from front of each pool
    for _ in range(common_consumed):
        if state.common:
            state.common.pop(0)

    for _ in range(uncommon_consumed):
        if state.uncommon:
            state.uncommon.pop(0)

    for _ in range(rare_consumed):
        if state.rare:
            state.rare.pop(0)

    for _ in range(shop_consumed):
        if state.shop:
            state.shop.pop(0)

    for _ in range(boss_consumed):
        if state.boss:
            state.boss.pop(0)

    return state


def get_relic_rng_at_counter(seed: int, counter: int) -> '_rng_mod.Random':
    """
    Get relicRng at a specific counter value.

    The relicRng is advanced during initialization:
    - 5 randomLong() calls for pool shuffles (counter += 5)

    Then during gameplay:
    - Each returnRandomRelicTier() calls random(0, 99) (counter += 1)

    Args:
        seed: The game seed
        counter: Target counter value

    Returns:
        Random instance at specified counter
    """
    return _rng_mod.Random(seed, counter)


# ============================================================================
# TIER ROLLING (matches returnRandomRelicTier)
# ============================================================================

# Default tier chances (Acts 1-3)
DEFAULT_COMMON_CHANCE = 50
DEFAULT_UNCOMMON_CHANCE = 33
# RARE is implicit: 100 - 50 - 33 = 17

# Act 4 (The Ending) tier chances
ACT4_COMMON_CHANCE = 0
ACT4_UNCOMMON_CHANCE = 100
# RARE is implicit: 100 - 0 - 100 = 0 (but actually all uncommon)


def roll_relic_tier(relic_rng: '_rng_mod.Random',
                    common_chance: int = DEFAULT_COMMON_CHANCE,
                    uncommon_chance: int = DEFAULT_UNCOMMON_CHANCE) -> str:
    """
    Roll a random relic tier using relicRng.

    Matches Java's returnRandomRelicTier():
    ```java
    public static AbstractRelic.RelicTier returnRandomRelicTier() {
        int roll = relicRng.random(0, 99);
        if (roll < commonRelicChance) {
            return AbstractRelic.RelicTier.COMMON;
        }
        if (roll < commonRelicChance + uncommonRelicChance) {
            return AbstractRelic.RelicTier.UNCOMMON;
        }
        return AbstractRelic.RelicTier.RARE;
    }
    ```

    Args:
        relic_rng: The relicRng instance (will be mutated)
        common_chance: % chance for common (default 50 for Acts 1-3)
        uncommon_chance: % chance for uncommon (default 33 for Acts 1-3)

    Returns:
        "COMMON", "UNCOMMON", or "RARE"
    """
    roll = relic_rng.random_int_range(0, 99)

    if roll < common_chance:
        return "COMMON"
    if roll < common_chance + uncommon_chance:
        return "UNCOMMON"
    return "RARE"


# ============================================================================
# RELIC ACQUISITION FUNCTIONS (with cascade logic)
# ============================================================================

def get_relic_from_pool(pool_state: RelicPoolState, tier: str,
                        from_end: bool = False) -> str:
    """
    Get a relic from pool with cascade logic and canSpawn validation.

    Cascade order (matches Java):
    - COMMON -> UNCOMMON -> RARE -> "Circlet"
    - UNCOMMON -> RARE -> "Circlet"
    - RARE -> "Circlet"
    - SHOP -> UNCOMMON (then follows UNCOMMON cascade)
    - BOSS -> "Red Circlet" (no cascade)

    canSpawn Logic (matches Java returnRandomRelicKey):
    - Relic is removed from pool first
    - Then canSpawn() is checked
    - If canSpawn() returns false, recursively try returnEndRandomRelicKey
    - This means relics that fail canSpawn ARE consumed from the pool

    Args:
        pool_state: The mutable pool state
        tier: Starting tier to try
        from_end: If True, use LIFO (returnEndRandomRelicKey), else FIFO

    Returns:
        Relic ID or "Circlet"/"Red Circlet" if all pools empty
    """
    tier = tier.upper()
    take_fn = pool_state.take_from_end if from_end else pool_state.take_from_front

    if tier == "BOSS":
        # Boss always takes from front, no cascade
        relic = pool_state.take_from_front("BOSS")
        if not relic:
            return "Red Circlet"

        # Check canSpawn - if fails, try again (recursively)
        # This matches Java: if (!RelicLibrary.getRelic(retVal).canSpawn()) {
        #     return AbstractDungeon.returnEndRandomRelicKey(tier);
        # }
        # Note: For BOSS tier, returnEndRandomRelicKey also uses remove(0),
        # so from_end doesn't matter - boss pool always takes from front
        if not can_boss_relic_spawn(relic, pool_state.act_num, pool_state.owned_relics):
            # Relic was already consumed from pool, try to get another
            return get_relic_from_pool(pool_state, "BOSS", from_end=True)

        return relic

    if tier == "SHOP":
        # Shop pool, cascade to UNCOMMON only (Java doesn't cascade further from shop)
        relic = take_fn("SHOP")
        if relic:
            return relic
        # Java cascades shop -> uncommon, but uses returnRandomRelicKey which cascades normally
        return get_relic_from_pool(pool_state, "UNCOMMON", from_end)

    # Standard cascade: COMMON -> UNCOMMON -> RARE -> Circlet
    cascade_order = {
        "COMMON": ["COMMON", "UNCOMMON", "RARE"],
        "UNCOMMON": ["UNCOMMON", "RARE"],
        "RARE": ["RARE"],
    }

    for try_tier in cascade_order.get(tier, [tier]):
        relic = take_fn(try_tier)
        if relic:
            return relic

    return "Circlet"


def get_relic_with_filter(pool_state: RelicPoolState, tier: str,
                          blocked_relics: set, from_end: bool = False) -> str:
    """
    Get a relic while skipping certain blocked relics.

    Matches returnRandomScreenlessRelic and returnRandomNonCampfireRelic logic:
    The Java code repeatedly calls returnRandomRelicKey until it gets one
    that isn't blocked. This means blocked relics ARE removed from the pool.

    Args:
        pool_state: The mutable pool state
        tier: Tier to get from
        blocked_relics: Set of relic IDs to skip (but still consume)
        from_end: If True, use LIFO

    Returns:
        First non-blocked relic, or fallback if pool exhausted
    """
    while True:
        relic = get_relic_from_pool(pool_state, tier, from_end)
        if relic in ("Circlet", "Red Circlet"):
            return relic
        if relic not in blocked_relics:
            return relic
        # Relic was blocked, it's already removed from pool, try again


def get_screenless_relic(pool_state: RelicPoolState, tier: str) -> str:
    """
    Get a relic that doesn't require a card selection screen.

    Matches returnRandomScreenlessRelic - skips:
    - Bottled Flame, Bottled Lightning, Bottled Tornado, Whetstone
    """
    return get_relic_with_filter(pool_state, tier, SCREENLESS_BLOCKED)


def get_non_campfire_relic(pool_state: RelicPoolState, tier: str) -> str:
    """
    Get a relic that isn't campfire-related.

    Matches returnRandomNonCampfireRelic - skips:
    - Peace Pipe, Shovel, Girya
    """
    return get_relic_with_filter(pool_state, tier, NON_CAMPFIRE_BLOCKED)


# ============================================================================
# HIGH-LEVEL RELIC ACQUISITION CONTEXTS
# ============================================================================

def get_combat_relic(pool_state: RelicPoolState, relic_rng: '_rng_mod.Random',
                     common_chance: int = DEFAULT_COMMON_CHANCE,
                     uncommon_chance: int = DEFAULT_UNCOMMON_CHANCE) -> str:
    """
    Get a relic from combat rewards (normal monster, elite).

    Process:
    1. Roll tier using relicRng
    2. Pull from that tier with cascade

    Args:
        pool_state: Mutable pool state
        relic_rng: The relicRng instance
        common_chance: % for common (varies by act)
        uncommon_chance: % for uncommon

    Returns:
        Relic ID
    """
    tier = roll_relic_tier(relic_rng, common_chance, uncommon_chance)
    return get_relic_from_pool(pool_state, tier)


def get_elite_relic(pool_state: RelicPoolState, relic_rng: '_rng_mod.Random',
                    common_chance: int = DEFAULT_COMMON_CHANCE,
                    uncommon_chance: int = DEFAULT_UNCOMMON_CHANCE) -> str:
    """
    Get a relic from elite combat.

    Note: Black Star relic doubles this (call twice).
    Same logic as get_combat_relic.
    """
    return get_combat_relic(pool_state, relic_rng, common_chance, uncommon_chance)


def get_event_relic(pool_state: RelicPoolState, tier: str) -> str:
    """
    Get a relic from an event with specified tier.

    Many events give specific-tier relics (e.g., "random rare relic").
    No tier rolling, just direct pool access.
    """
    return get_relic_from_pool(pool_state, tier)


def get_shop_relic(pool_state: RelicPoolState) -> str:
    """
    Get a relic from the shop pool.

    Shop has its own pool, cascades to UNCOMMON if empty.
    """
    return get_relic_from_pool(pool_state, "SHOP")


def get_boss_relic(pool_state: RelicPoolState) -> str:
    """
    Get the next boss relic (takes from front).

    Used for boss rewards and Neow boss swap.
    """
    return get_relic_from_pool(pool_state, "BOSS")


def get_boss_relic_choices(pool_state: RelicPoolState, count: int = 3) -> List[str]:
    """
    Get boss relic choices without consuming them (for preview).

    Returns the next N boss relics that would be offered, filtering out
    relics that cannot spawn due to canSpawn() checks.

    Note: This is a preview - relics that fail canSpawn are not consumed
    from the pool until actually taken. In the real game, the BossRelicSelectScreen
    iterates through the pool and shows relics that can spawn.

    Does NOT modify the pool state.
    """
    choices = []
    for relic in pool_state.boss:
        if can_boss_relic_spawn(relic, pool_state.act_num, pool_state.owned_relics):
            choices.append(relic)
            if len(choices) >= count:
                break
    return choices


def take_boss_relic_choice(pool_state: RelicPoolState, chosen_relic: str) -> str:
    """
    Take a specific boss relic from the choices.

    After boss fight, player chooses 1 of 3 boss relics.
    This removes the chosen relic from the pool.

    Note: The other 2 relics stay in the pool and can be offered again later.
    Relics that fail canSpawn() checks are not shown but remain in the pool
    until they're consumed through normal pool iteration.

    Returns the chosen relic ID.
    """
    if chosen_relic in pool_state.boss:
        pool_state.boss.remove(chosen_relic)
        pool_state.mark_owned(chosen_relic)
        return chosen_relic
    return "Red Circlet"


# ============================================================================
# CALLING BELL SPECIAL HANDLING
# ============================================================================

def get_calling_bell_relics(pool_state: RelicPoolState) -> Tuple[str, str, str]:
    """
    Get the 3 relics Calling Bell would give.

    IMPORTANT: This CONSUMES relics from the pools (mutates pool_state).

    Calling Bell gives:
    1. returnRandomScreenlessRelic(COMMON)
    2. returnRandomScreenlessRelic(UNCOMMON)
    3. returnRandomScreenlessRelic(RARE)

    Note: If Calling Bell is obtained via boss swap, boss[0] (Calling Bell)
    is already consumed. The relic pools are already affected.
    """
    common = get_screenless_relic(pool_state, "COMMON")
    uncommon = get_screenless_relic(pool_state, "UNCOMMON")
    rare = get_screenless_relic(pool_state, "RARE")

    return (common, uncommon, rare)


def predict_calling_bell_relics(seed: int, player_class: str = "WATCHER",
                                 assume_boss_swap: bool = True) -> Tuple[str, str, str]:
    """
    Predict which 3 relics Calling Bell would give.

    Calling Bell grants:
    1. returnRandomScreenlessRelic(COMMON)
    2. returnRandomScreenlessRelic(UNCOMMON)
    3. returnRandomScreenlessRelic(RARE)

    returnRandomScreenlessRelic skips: Bottled Flame, Bottled Lightning,
    Bottled Tornado, Whetstone (relics that require card selection).

    IMPORTANT: If Calling Bell is obtained via Neow boss swap, the boss pool
    has already consumed boss[0] (Calling Bell itself). This affects which
    boss relics would be available later, but doesn't directly affect the
    common/uncommon/rare pools that Calling Bell pulls from.

    Args:
        seed: The game seed (long value)
        player_class: Player class name
        assume_boss_swap: If True, assumes Calling Bell was obtained via
                         Neow boss swap (removes it from boss pool)

    Returns:
        Tuple of (common_relic, uncommon_relic, rare_relic)
    """
    # Create mutable pool state
    pool_state = create_relic_pool_state(seed, player_class)

    # If obtained via boss swap, Calling Bell (boss[0]) is consumed
    if assume_boss_swap:
        # Verify Calling Bell is actually at boss[0] for this seed
        if pool_state.boss and pool_state.boss[0] == "Calling Bell":
            pool_state.take_from_front("BOSS")
        # If Calling Bell isn't at boss[0], it wasn't obtained via boss swap
        # for this seed, which is a logic error but we proceed anyway

    # Now get the 3 relics Calling Bell would give
    return get_calling_bell_relics(pool_state)


def predict_calling_bell_relics_at_pool_state(
    pool_state: RelicPoolState,
    consume: bool = True
) -> Tuple[str, str, str]:
    """
    Predict Calling Bell relics given current pool state.

    Use this when you've already tracked pool consumption through the run
    and want to see what Calling Bell would give at the current state.

    Args:
        pool_state: Current pool state (will be modified if consume=True)
        consume: If True, actually removes relics from pools

    Returns:
        Tuple of (common_relic, uncommon_relic, rare_relic)
    """
    if not consume:
        pool_state = pool_state.copy()

    return get_calling_bell_relics(pool_state)


# ============================================================================
# TESTING
# ============================================================================

def main():
    """Test relic pool prediction and pool state tracking."""
    import sys

    seed_str = sys.argv[1] if len(sys.argv) > 1 else "TEST123"
    seed = _rng_mod.seed_to_long(seed_str)

    print(f"{'='*60}")
    print(f"RELIC POOL SYSTEM TEST FOR SEED: {seed_str}")
    print(f"Seed value: {seed}")
    print(f"{'='*60}")

    # Show boss pool before shuffle
    print("\n=== BOSS RELIC POOL (Iteration Order) ===")
    pool_order = get_boss_relic_pool_order("WATCHER")
    for i, relic in enumerate(pool_order):
        print(f"  {i}: {relic}")

    # Show shuffled order
    print("\n=== SHUFFLED BOSS RELIC POOL ===")
    shuffled = predict_boss_relic_pool(seed, "WATCHER")
    for i, relic in enumerate(shuffled[:5]):
        marker = " <-- NEOW BOSS SWAP" if i == 0 else ""
        print(f"  {i}: {relic}{marker}")
    print("  ...")

    print(f"\n=== NEOW BOSS SWAP RESULT ===")
    boss_swap = predict_neow_boss_swap(seed, "WATCHER")
    print(f"  {boss_swap}")

    # Test the new mutable pool state system
    print(f"\n{'='*60}")
    print("MUTABLE POOL STATE DEMONSTRATION")
    print(f"{'='*60}")

    # Create pool state for this seed
    pool_state = create_relic_pool_state(seed, "WATCHER")

    print("\n=== INITIAL POOL SIZES ===")
    print(f"  Common: {len(pool_state.common)}")
    print(f"  Uncommon: {len(pool_state.uncommon)}")
    print(f"  Rare: {len(pool_state.rare)}")
    print(f"  Shop: {len(pool_state.shop)}")
    print(f"  Boss: {len(pool_state.boss)}")

    # Simulate Neow boss swap
    print("\n=== SIMULATING NEOW BOSS SWAP ===")
    boss_relic = get_boss_relic(pool_state)
    print(f"  Boss swap gave: {boss_relic}")
    print(f"  Boss pool remaining: {len(pool_state.boss)}")

    # If we got Calling Bell, show what it would give
    if boss_relic == "Calling Bell":
        print("\n=== CALLING BELL EFFECT ===")
        common, uncommon, rare = get_calling_bell_relics(pool_state)
        print(f"  Common: {common}")
        print(f"  Uncommon: {uncommon}")
        print(f"  Rare: {rare}")
        print(f"\n  Pool sizes after Calling Bell:")
        print(f"    Common: {len(pool_state.common)}")
        print(f"    Uncommon: {len(pool_state.uncommon)}")
        print(f"    Rare: {len(pool_state.rare)}")

    # Demonstrate tier rolling
    print("\n=== TIER ROLL DEMONSTRATION ===")
    relic_rng = _rng_mod.Random(seed)
    # Advance past pool shuffles (5 randomLong calls)
    for _ in range(5):
        relic_rng.random_long()

    print("  Next 5 tier rolls (Acts 1-3 chances: 50/33/17):")
    for i in range(5):
        tier = roll_relic_tier(relic_rng)
        print(f"    Roll {i+1}: {tier}")

    # Demonstrate combat relic acquisition
    print("\n=== SIMULATING COMBAT RELIC DROPS ===")
    pool_state_fresh = create_relic_pool_state(seed, "WATCHER")
    relic_rng_fresh = _rng_mod.Random(seed)
    # Advance past pool shuffles
    for _ in range(5):
        relic_rng_fresh.random_long()

    print("  First 5 combat relics:")
    for i in range(5):
        relic = get_combat_relic(pool_state_fresh, relic_rng_fresh)
        print(f"    Drop {i+1}: {relic}")

    # Test predict_calling_bell_relics with different scenarios
    print(f"\n{'='*60}")
    print("CALLING BELL PREDICTION TEST")
    print(f"{'='*60}")

    # Find a seed where Calling Bell is the boss swap
    test_seeds = ["BELL1", "BELL2", "BELL3", "CALLING", "BELLTEST"]
    for test_seed_str in test_seeds:
        test_seed = _rng_mod.seed_to_long(test_seed_str)
        test_pools = predict_all_relic_pools(test_seed, "WATCHER")
        if test_pools.boss and test_pools.boss[0] == "Calling Bell":
            print(f"\n  Seed '{test_seed_str}' has Calling Bell as boss swap!")
            common, uncommon, rare = predict_calling_bell_relics(test_seed, "WATCHER")
            print(f"    Common: {common}")
            print(f"    Uncommon: {uncommon}")
            print(f"    Rare: {rare}")
            break
    else:
        print("\n  (No test seed found with Calling Bell as boss swap)")
        print("  Using original seed for Calling Bell prediction:")
        common, uncommon, rare = predict_calling_bell_relics(seed, "WATCHER", assume_boss_swap=False)
        print(f"    Common: {common}")
        print(f"    Uncommon: {uncommon}")
        print(f"    Rare: {rare}")

    # Test canSpawn validation
    print(f"\n{'='*60}")
    print("BOSS RELIC canSpawn() VALIDATION TEST")
    print(f"{'='*60}")

    # Test Ectoplasm (only spawns in Act 1)
    print("\n=== Ectoplasm canSpawn (Act 1 only) ===")
    print(f"  Act 1: {can_boss_relic_spawn('Ectoplasm', act_num=1)}")  # Should be True
    print(f"  Act 2: {can_boss_relic_spawn('Ectoplasm', act_num=2)}")  # Should be False
    print(f"  Act 3: {can_boss_relic_spawn('Ectoplasm', act_num=3)}")  # Should be False

    # Test class-specific boss relics
    print("\n=== Class-specific Boss Relics ===")
    print("  HolyWater (Watcher boss):")
    print(f"    With PureWater: {can_boss_relic_spawn('HolyWater', owned_relics=['PureWater'])}")  # True
    print(f"    Without PureWater: {can_boss_relic_spawn('HolyWater', owned_relics=[])}")  # False

    print("  FrozenCore (Defect boss):")
    print(f"    With Cracked Core: {can_boss_relic_spawn('FrozenCore', owned_relics=['Cracked Core'])}")  # True
    print(f"    Without Cracked Core: {can_boss_relic_spawn('FrozenCore', owned_relics=[])}")  # False

    # Test boss relic choices with canSpawn filtering
    print("\n=== Boss Relic Choices with canSpawn Filtering ===")

    # Act 1 with Watcher starter - should filter out HolyWater if at front
    pool_state_act1 = create_relic_pool_state(seed, "WATCHER", owned_relics=["PureWater"], act_num=1)
    choices_act1 = get_boss_relic_choices(pool_state_act1, count=3)
    print(f"  Act 1 (with PureWater): {choices_act1}")

    # Act 2 - Ectoplasm should be filtered out
    pool_state_act2 = create_relic_pool_state(seed, "WATCHER", owned_relics=["PureWater"], act_num=2)
    # Find position of Ectoplasm in pool
    ecto_pos = pool_state_act2.boss.index("Ectoplasm") if "Ectoplasm" in pool_state_act2.boss else -1
    print(f"  Ectoplasm position in pool: {ecto_pos}")
    choices_act2 = get_boss_relic_choices(pool_state_act2, count=3)
    print(f"  Act 2 choices (Ectoplasm filtered): {choices_act2}")
    print(f"  Ectoplasm in choices: {'Ectoplasm' in choices_act2}")  # Should be False


if __name__ == "__main__":
    main()
