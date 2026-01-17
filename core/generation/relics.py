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
- relicRng.random(0, 99) used for relic roll chances
"""

import os
import importlib.util
from dataclasses import dataclass
from typing import List, Dict, Optional, Tuple

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
        ("ChampionBelt", "RARE"),
        ("CharonsAshes", "RARE"),
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
        ("Paper Krane", "UNCOMMON"),
        ("Ring of the Serpent", "BOSS"),
        ("SnakeRing", "STARTER"),
        ("Snecko Skull", "COMMON"),
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
        ("GoldPlatedCables", "UNCOMMON"),
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


# Relics that returnRandomScreenlessRelic skips (require card selection screen)
SCREENLESS_BLOCKED = {"Bottled Flame", "Bottled Lightning", "Bottled Tornado", "Whetstone"}


def predict_calling_bell_relics(seed: int, player_class: str = "WATCHER") -> Tuple[str, str, str]:
    """
    Predict which 3 relics Calling Bell would give.

    Calling Bell grants:
    1. returnRandomScreenlessRelic(COMMON)
    2. returnRandomScreenlessRelic(UNCOMMON)
    3. returnRandomScreenlessRelic(RARE)

    returnRandomScreenlessRelic skips: Bottled Flame, Bottled Lightning,
    Bottled Tornado, Whetstone (relics that require card selection).

    Note: Boss swap happens AFTER relic pools are initialized, so pools
    are already shuffled when Calling Bell effect triggers.

    Args:
        seed: The game seed (long value)
        player_class: Player class name

    Returns:
        Tuple of (common_relic, uncommon_relic, rare_relic)
    """
    pools = predict_all_relic_pools(seed, player_class)

    # Simulate pool consumption:
    # Boss swap takes pools.boss[0] (Calling Bell itself)
    # Then Calling Bell effect pulls from pools

    # Common: take first that isn't blocked
    common_idx = 0
    while common_idx < len(pools.common) and pools.common[common_idx] in SCREENLESS_BLOCKED:
        common_idx += 1
    common_relic = pools.common[common_idx] if common_idx < len(pools.common) else "Circlet"

    # Uncommon: take first that isn't blocked
    uncommon_idx = 0
    while uncommon_idx < len(pools.uncommon) and pools.uncommon[uncommon_idx] in SCREENLESS_BLOCKED:
        uncommon_idx += 1
    uncommon_relic = pools.uncommon[uncommon_idx] if uncommon_idx < len(pools.uncommon) else "Circlet"

    # Rare: take first that isn't blocked
    rare_idx = 0
    while rare_idx < len(pools.rare) and pools.rare[rare_idx] in SCREENLESS_BLOCKED:
        rare_idx += 1
    rare_relic = pools.rare[rare_idx] if rare_idx < len(pools.rare) else "Circlet"

    return (common_relic, uncommon_relic, rare_relic)


# ============================================================================
# TESTING
# ============================================================================

def main():
    """Test relic pool prediction."""
    import sys

    seed_str = sys.argv[1] if len(sys.argv) > 1 else "TEST123"
    seed = _rng_mod.seed_to_long(seed_str)

    print(f"{'='*60}")
    print(f"BOSS RELIC PREDICTION FOR SEED: {seed_str}")
    print(f"Seed value: {seed}")
    print(f"{'='*60}")

    # Show pool before shuffle
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


if __name__ == "__main__":
    main()
