"""
Slay the Spire - Encounter Generation System

Replicates the exact encounter generation algorithm from the decompiled source.

Key mechanics:
1. Monsters are pre-generated at dungeon creation using monsterRng
2. Each act has weak, strong, and elite pools with weighted probabilities
3. No back-to-back repeats (or 2-back for non-elites)
4. Weak encounters for first floors, strong for the rest

RNG Usage:
- monsterRng.random() returns float [0, 1) for weighted selection
- Boss shuffle uses monsterRng.randomLong() with Java's Random

Act-specific details:
- Act 1 (Exordium): 3 weak, 13 strong (1 first + 12 populate), 10 elite
- Act 2 (The City): 2 weak, 13 strong (1 first + 12 populate), 10 elite
- Act 3 (The Beyond): 2 weak, 13 strong (1 first + 12 populate), 10 elite
- Act 4 (The Ending): Fixed encounters - no RNG needed
  - 1 Elite: Spire Shield and Spire Spear
  - 1 Rest Site
  - 1 Boss: Corrupt Heart
  - Requires 3 keys (Ruby, Emerald, Sapphire) to access
"""

import os
import importlib.util
from dataclasses import dataclass
from typing import List, Dict, Tuple, Optional
import random


# Load RNG module
_core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def _load_module(name: str, filepath: str):
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

_rng_module = _load_module("rng", os.path.join(_core_dir, "state", "rng.py"))
Random = _rng_module.Random
seed_to_long = _rng_module.seed_to_long


@dataclass
class MonsterInfo:
    """Monster info with spawn weight."""
    name: str
    weight: float


def normalize_weights(monsters: List[MonsterInfo]) -> List[MonsterInfo]:
    """
    Normalize weights and sort by weight ascending.

    Matches Java's MonsterInfo.normalizeWeights():
    1. Sort by weight (ascending)
    2. Divide each weight by total
    """
    # Sort by weight ascending
    monsters = sorted(monsters, key=lambda m: m.weight)

    # Calculate total
    total = sum(m.weight for m in monsters)

    # Normalize
    for m in monsters:
        m.weight = m.weight / total

    return monsters


def roll_monster(monsters: List[MonsterInfo], roll: float) -> str:
    """
    Select a monster based on roll value using cumulative weights.

    Matches Java's MonsterInfo.roll():
    - Iterates through sorted list
    - Returns first monster where cumulative weight > roll

    Args:
        monsters: Normalized list of monsters (sorted by weight)
        roll: Random float in [0, 1)

    Returns:
        Monster name
    """
    current_weight = 0.0
    for m in monsters:
        current_weight += m.weight
        if roll < current_weight:
            return m.name
    return "ERROR"


# ============================================================================
# EXORDIUM (ACT 1) ENCOUNTER POOLS
# ============================================================================

def get_exordium_weak_pool() -> List[MonsterInfo]:
    """Weak monster pool for Act 1 floors 1-3."""
    return normalize_weights([
        MonsterInfo("Cultist", 2.0),
        MonsterInfo("Jaw Worm", 2.0),
        MonsterInfo("2 Louse", 2.0),
        MonsterInfo("Small Slimes", 2.0),
    ])


def get_exordium_strong_pool() -> List[MonsterInfo]:
    """Strong monster pool for Act 1 floors 4+."""
    return normalize_weights([
        MonsterInfo("Blue Slaver", 2.0),
        MonsterInfo("Gremlin Gang", 1.0),
        MonsterInfo("Looter", 2.0),
        MonsterInfo("Large Slime", 2.0),
        MonsterInfo("Lots of Slimes", 1.0),
        MonsterInfo("Exordium Thugs", 1.5),
        MonsterInfo("Exordium Wildlife", 1.5),
        MonsterInfo("Red Slaver", 1.0),
        MonsterInfo("3 Louse", 2.0),
        MonsterInfo("2 Fungi Beasts", 2.0),
    ])


def get_exordium_elite_pool() -> List[MonsterInfo]:
    """Elite monster pool for Act 1."""
    return normalize_weights([
        MonsterInfo("Gremlin Nob", 1.0),
        MonsterInfo("Lagavulin", 1.0),
        MonsterInfo("3 Sentries", 1.0),
    ])


def get_exordium_exclusions(last_monster: str) -> List[str]:
    """
    Get exclusions for first strong monster based on last weak monster.

    Prevents thematically similar encounters back-to-back.
    """
    exclusions = {
        "Looter": ["Exordium Thugs"],
        "Blue Slaver": ["Red Slaver", "Exordium Thugs"],
        "2 Louse": ["3 Louse"],
        "Small Slimes": ["Large Slime", "Lots of Slimes"],
    }
    return exclusions.get(last_monster, [])


EXORDIUM_BOSSES = ["The Guardian", "Hexaghost", "Slime Boss"]


# ============================================================================
# THE CITY (ACT 2) ENCOUNTER POOLS
# ============================================================================

def get_city_weak_pool() -> List[MonsterInfo]:
    """Weak monster pool for Act 2 first floors."""
    return normalize_weights([
        MonsterInfo("Spheric Guardian", 2.0),
        MonsterInfo("Chosen", 2.0),
        MonsterInfo("Shell Parasite", 2.0),
        MonsterInfo("3 Byrds", 2.0),
        MonsterInfo("2 Thieves", 2.0),
    ])


def get_city_strong_pool() -> List[MonsterInfo]:
    """Strong monster pool for Act 2."""
    return normalize_weights([
        MonsterInfo("Chosen and Byrds", 2.0),
        MonsterInfo("Sentry and Sphere", 2.0),
        MonsterInfo("Snake Plant", 6.0),
        MonsterInfo("Snecko", 4.0),
        MonsterInfo("Centurion and Healer", 6.0),
        MonsterInfo("Cultist and Chosen", 3.0),
        MonsterInfo("3 Cultists", 3.0),
        MonsterInfo("Shelled Parasite and Fungi", 3.0),
    ])


def get_city_elite_pool() -> List[MonsterInfo]:
    """Elite monster pool for Act 2."""
    return normalize_weights([
        MonsterInfo("Gremlin Leader", 1.0),
        MonsterInfo("Slavers", 1.0),
        MonsterInfo("Book of Stabbing", 1.0),
    ])


def get_city_exclusions(last_monster: str) -> List[str]:
    """
    Get exclusions for first strong monster in Act 2.

    Prevents thematically similar encounters back-to-back.

    Note: Java's TheCity.generateExclusions() only handles:
    - Spheric Guardian -> Sentry and Sphere
    - 3 Byrds -> Chosen and Byrds
    - Chosen -> Chosen and Byrds, Cultist and Chosen

    There is NO case for Shell Parasite or 2 Thieves in Java!
    """
    exclusions = {
        "Chosen": ["Chosen and Byrds", "Cultist and Chosen"],
        "Spheric Guardian": ["Sentry and Sphere"],
        "3 Byrds": ["Chosen and Byrds"],
        # NO Shell Parasite case - Java doesn't have this exclusion!
    }
    return exclusions.get(last_monster, [])


CITY_BOSSES = ["Automaton", "Collector", "Champ"]


# ============================================================================
# THE BEYOND (ACT 3) ENCOUNTER POOLS
# ============================================================================

def get_beyond_weak_pool() -> List[MonsterInfo]:
    """Weak monster pool for Act 3 first floors."""
    return normalize_weights([
        MonsterInfo("3 Darklings", 2.0),
        MonsterInfo("Orb Walker", 2.0),
        MonsterInfo("3 Shapes", 2.0),
    ])


def get_beyond_strong_pool() -> List[MonsterInfo]:
    """Strong monster pool for Act 3."""
    return normalize_weights([
        MonsterInfo("Spire Growth", 1.0),
        MonsterInfo("Transient", 1.0),
        MonsterInfo("4 Shapes", 1.0),
        MonsterInfo("Maw", 1.0),
        MonsterInfo("Sphere and 2 Shapes", 1.0),
        MonsterInfo("Jaw Worm Horde", 1.0),
        MonsterInfo("3 Darklings", 1.0),
        MonsterInfo("Writhing Mass", 1.0),
    ])


def get_beyond_elite_pool() -> List[MonsterInfo]:
    """Elite monster pool for Act 3."""
    return normalize_weights([
        MonsterInfo("Giant Head", 2.0),
        MonsterInfo("Nemesis", 2.0),
        MonsterInfo("Reptomancer", 2.0),
    ])


def get_beyond_exclusions(last_monster: str) -> List[str]:
    """
    Get exclusions for first strong monster in Act 3.

    Prevents thematically similar encounters back-to-back.

    Java source (TheBeyond.generateExclusions):
    - "3 Darklings" -> excludes "3 Darklings" (same encounter in strong pool)
    - "Orb Walker" -> excludes "Orb Walker" (not in strong pool, but matches Java)
    - "3 Shapes" -> excludes "4 Shapes" (shape variants)
    """
    exclusions = {
        "3 Darklings": ["3 Darklings"],
        "Orb Walker": ["Orb Walker"],
        "3 Shapes": ["4 Shapes"],
    }
    return exclusions.get(last_monster, [])


BEYOND_BOSSES = ["Awakened One", "Time Eater", "Donu and Deca"]


# ============================================================================
# THE ENDING (ACT 4) - FIXED ENCOUNTERS
# ============================================================================

# Act 4 has no random encounters - everything is fixed
ENDING_ELITE = "Spire Shield and Spire Spear"
ENDING_BOSS = "Corrupt Heart"
ENDING_BOSSES = ["Corrupt Heart"]  # Fixed - no shuffle

# Act 4 structure:
# - Floor 1: Elite fight (Spire Shield and Spire Spear)
# - Floor 2: Rest site
# - Floor 3: Boss fight (Corrupt Heart)

# Key requirements to access Act 4
ACT_4_KEYS = ["Ruby Key", "Emerald Key", "Sapphire Key"]


def get_ending_encounters() -> Dict:
    """
    Get Act 4 (The Ending) encounter information.

    Act 4 has fixed encounters - no RNG is used.

    Returns:
        Dictionary with:
        - elite: The elite encounter (Spire Shield and Spire Spear)
        - boss: The boss (Corrupt Heart)
        - structure: Description of act layout
    """
    return {
        "elite": ENDING_ELITE,
        "boss": ENDING_BOSS,
        "structure": [
            {"floor": 1, "type": "elite", "encounter": ENDING_ELITE},
            {"floor": 2, "type": "rest", "encounter": None},
            {"floor": 3, "type": "boss", "encounter": ENDING_BOSS},
        ],
        "key_requirement": ACT_4_KEYS,
    }


# ============================================================================
# ENCOUNTER LIST GENERATION
# ============================================================================

def populate_monster_list(
    monster_list: List[str],
    monsters: List[MonsterInfo],
    rng: Random,
    count: int,
    is_elite: bool = False,
) -> None:
    """
    Populate encounter list with monsters.

    Matches Java's AbstractDungeon.populateMonsterList():
    - No immediate repeat (same as last)
    - For non-elites: no repeat of 2-back either
    - Rerolls if would repeat

    Args:
        monster_list: List to populate (modified in place)
        monsters: Pool of monsters to choose from
        rng: Monster RNG stream
        count: Number of monsters to add
        is_elite: Whether this is elite list (different repeat rules)
    """
    i = 0
    while i < count:
        roll = rng.random_float()
        to_add = roll_monster(monsters, roll)

        if not monster_list:
            # First monster, always add
            monster_list.append(to_add)
            i += 1
        elif to_add == monster_list[-1]:
            # Same as last, reroll (don't increment i)
            continue
        elif not is_elite and len(monster_list) > 1 and to_add == monster_list[-2]:
            # Same as 2-back for non-elites, reroll
            continue
        else:
            monster_list.append(to_add)
            i += 1


def populate_first_strong_enemy(
    monster_list: List[str],
    monsters: List[MonsterInfo],
    rng: Random,
    exclusions: List[str],
) -> None:
    """
    Add first strong enemy with exclusions.

    Matches Java's AbstractDungeon.populateFirstStrongEnemy():
    - Rerolls while rolled monster is in exclusions
    """
    while True:
        roll = rng.random_float()
        m = roll_monster(monsters, roll)
        if m not in exclusions:
            monster_list.append(m)
            return


def shuffle_bosses(bosses: List[str], rng: Random, should_shuffle: bool = True) -> List[str]:
    """
    Shuffle boss list using Java's Random seeded with monsterRng.randomLong().

    Java Behavior (from Exordium/TheCity/TheBeyond.initializeBoss):
    - If daily run OR all bosses seen: shuffle boss list with monsterRng.randomLong()
    - If specific boss not seen: use deterministic boss (NO RNG consumed)

    Matches Java's: Collections.shuffle(bossList, new java.util.Random(monsterRng.randomLong()))

    Args:
        bosses: List of boss names
        rng: Monster RNG stream
        should_shuffle: Whether to shuffle (True if all bosses seen or daily run)

    Returns:
        Shuffled list of bosses (or original if should_shuffle=False)
    """
    if not should_shuffle:
        # No shuffle - return original order, no RNG consumed
        return bosses.copy()

    # Get seed from monsterRng.randomLong()
    java_seed = rng.random_long()

    # Create Python random with same seed (Java Random behavior)
    # Java's Random uses (seed ^ 0x5DEECE66D) & ((1L << 48) - 1)
    # but Python's random.seed() handles this differently.
    # We'll use the linear congruential generator approach to match Java.
    shuffled = bosses.copy()
    _java_shuffle(shuffled, java_seed)
    return shuffled


def _java_shuffle(lst: List, seed: int) -> None:
    """
    Shuffle list using Java's Collections.shuffle(list, Random(seed)).

    Replicate Java's Collections.shuffle(list, new Random(seed)).

    Java's Random uses a linear congruential generator (LCG):
    seed = (seed * 0x5DEECE66D + 0xB) & ((1 << 48) - 1)

    Collections.shuffle uses Random.nextInt(n) which:
    1. For powers of 2: return (int)((n * (long)next(31)) >> 31)
    2. Otherwise: rejection sampling with next(31) % n
    """
    # Initialize Java Random state
    # Java Random constructor: seed = (seed ^ 0x5DEECE66DL) & ((1L << 48) - 1)
    state = (seed ^ 0x5DEECE66D) & ((1 << 48) - 1)

    def next_bits(bits: int) -> int:
        nonlocal state
        state = (state * 0x5DEECE66D + 0xB) & ((1 << 48) - 1)
        return state >> (48 - bits)

    def next_int(bound: int) -> int:
        if bound <= 0:
            raise ValueError("bound must be positive")

        # Special case: power of 2
        if (bound & -bound) == bound:
            return (bound * next_bits(31)) >> 31

        # Rejection sampling
        while True:
            bits = next_bits(31)
            val = bits % bound
            if bits - val + (bound - 1) >= 0:
                return val

    # Fisher-Yates shuffle (Java's implementation)
    # for (int i = size; i > 1; i--)
    #     swap(list, i-1, rnd.nextInt(i));
    for i in range(len(lst), 1, -1):
        j = next_int(i)
        lst[i - 1], lst[j] = lst[j], lst[i - 1]


def generate_exordium_encounters(
    rng: Random,
    all_bosses_seen: bool = True,
) -> Tuple[List[str], List[str], str]:
    """
    Generate all Act 1 encounters.

    Java Behavior (from Exordium.initializeBoss):
    - If daily run OR all bosses seen: shuffle boss list with monsterRng.randomLong()
    - If specific boss not seen: use deterministic boss (NO RNG consumed)

    Args:
        rng: Monster RNG stream
        all_bosses_seen: Whether player has seen all bosses (default True for A20)

    Returns:
        (normal_encounters, elite_encounters, boss)
    """
    monster_list: List[str] = []
    elite_list: List[str] = []

    # Weak enemies (floors 1-3)
    weak_pool = get_exordium_weak_pool()
    populate_monster_list(monster_list, weak_pool, rng, 3)

    # First strong enemy with exclusions
    strong_pool = get_exordium_strong_pool()
    exclusions = get_exordium_exclusions(monster_list[-1])
    populate_first_strong_enemy(monster_list, strong_pool, rng, exclusions)

    # Remaining strong enemies
    # IMPORTANT: Java calls populateMonsterList(monsters, count=12, false) AFTER populateFirstStrongEnemy
    # This means 1 + 12 = 13 strong monsters total, NOT 12!
    populate_monster_list(monster_list, strong_pool, rng, 12)

    # Elite enemies
    elite_pool = get_exordium_elite_pool()
    populate_monster_list(elite_list, elite_pool, rng, 10, is_elite=True)

    # Boss shuffle (only if all bosses seen, otherwise deterministic)
    bosses = shuffle_bosses(EXORDIUM_BOSSES.copy(), rng, should_shuffle=all_bosses_seen)

    return monster_list, elite_list, bosses[0]


def generate_city_encounters(
    rng: Random,
    all_bosses_seen: bool = True,
) -> Tuple[List[str], List[str], str]:
    """
    Generate all Act 2 encounters.

    Java Behavior (from TheCity.initializeBoss):
    - If daily run OR all bosses seen: shuffle boss list with monsterRng.randomLong()
    - If specific boss not seen: use deterministic boss (NO RNG consumed)

    Args:
        rng: Monster RNG stream
        all_bosses_seen: Whether player has seen all bosses (default True for A20)

    Returns:
        (normal_encounters, elite_encounters, boss)
    """
    monster_list: List[str] = []
    elite_list: List[str] = []

    # Weak enemies (floors 1-2)
    weak_pool = get_city_weak_pool()
    populate_monster_list(monster_list, weak_pool, rng, 2)

    # First strong enemy with exclusions
    strong_pool = get_city_strong_pool()
    exclusions = get_city_exclusions(monster_list[-1])
    populate_first_strong_enemy(monster_list, strong_pool, rng, exclusions)

    # Remaining strong enemies
    # IMPORTANT: Java calls populateMonsterList(monsters, count=12, false) AFTER populateFirstStrongEnemy
    # This means 1 + 12 = 13 strong monsters total, NOT 12!
    populate_monster_list(monster_list, strong_pool, rng, 12)

    # Elite enemies
    elite_pool = get_city_elite_pool()
    populate_monster_list(elite_list, elite_pool, rng, 10, is_elite=True)

    # Boss shuffle (only if all bosses seen, otherwise deterministic)
    bosses = shuffle_bosses(CITY_BOSSES.copy(), rng, should_shuffle=all_bosses_seen)

    return monster_list, elite_list, bosses[0]


def generate_beyond_encounters(
    rng: Random,
    all_bosses_seen: bool = True,
) -> Tuple[List[str], List[str], str]:
    """
    Generate all Act 3 encounters.

    Java Behavior (from TheBeyond.initializeBoss):
    - If daily run OR all bosses seen: shuffle boss list with monsterRng.randomLong()
    - If specific boss not seen: use deterministic boss (NO RNG consumed)

    Args:
        rng: Monster RNG stream
        all_bosses_seen: Whether player has seen all bosses (default True for A20)

    Returns:
        (normal_encounters, elite_encounters, boss)
    """
    monster_list: List[str] = []
    elite_list: List[str] = []

    # Weak enemies (floors 1-2)
    weak_pool = get_beyond_weak_pool()
    populate_monster_list(monster_list, weak_pool, rng, 2)

    # First strong enemy with exclusions
    strong_pool = get_beyond_strong_pool()
    exclusions = get_beyond_exclusions(monster_list[-1])
    populate_first_strong_enemy(monster_list, strong_pool, rng, exclusions)

    # Remaining strong enemies
    # IMPORTANT: Java calls populateMonsterList(monsters, count=12, false) AFTER populateFirstStrongEnemy
    # This means 1 + 12 = 13 strong monsters total, NOT 12!
    populate_monster_list(monster_list, strong_pool, rng, 12)

    # Elite enemies
    elite_pool = get_beyond_elite_pool()
    populate_monster_list(elite_list, elite_pool, rng, 10, is_elite=True)

    # Boss shuffle (only if all bosses seen, otherwise deterministic)
    bosses = shuffle_bosses(BEYOND_BOSSES.copy(), rng, should_shuffle=all_bosses_seen)

    return monster_list, elite_list, bosses[0]


def generate_ending_encounters(rng: Optional[Random] = None) -> Tuple[List[str], List[str], str]:
    """
    Generate Act 4 (The Ending) encounters.

    Act 4 has FIXED encounters - no RNG is used.
    The RNG parameter is accepted for API consistency but ignored.

    Returns:
        (normal_encounters, elite_encounters, boss)
        - normal_encounters: Empty list (no normal fights)
        - elite_encounters: [ENDING_ELITE] (Spire Shield and Spire Spear)
        - boss: ENDING_BOSS (Corrupt Heart)
    """
    # Act 4 has no normal encounters
    monster_list: List[str] = []

    # Act 4 has one fixed elite fight
    elite_list: List[str] = [ENDING_ELITE]

    # Act 4 has one fixed boss
    boss = ENDING_BOSS

    return monster_list, elite_list, boss


# ============================================================================
# PREDICTION API
# ============================================================================

def predict_act_encounters(
    seed: str,
    act: int,
    monster_rng_counter: int = 0,
    all_bosses_seen: bool = True,
) -> Dict:
    """
    Predict encounters for a specific act given the seed and RNG state.

    Java Behavior (from Exordium/TheCity/TheBeyond.initializeBoss):
    - If daily run OR all bosses seen: shuffle boss list with monsterRng.randomLong()
    - If specific boss not seen: use deterministic boss (NO RNG consumed)

    The monsterRng is a persistent stream that advances through all acts.
    To predict future acts, you need to track the counter through prior acts.

    Args:
        seed: Seed string (e.g., "1ABCD")
        act: Act number (1, 2, 3, or 4)
        monster_rng_counter: Current monsterRng counter (from prior acts)
        all_bosses_seen: Whether player has seen all bosses (default True for A20)

    Returns:
        Dictionary with:
        - "monsters": List of normal encounters
        - "elites": List of elite encounters
        - "boss": Boss name
        - "monster_rng_counter": Final counter after generating this act
        - "fixed": True for Act 4 (no RNG used)
        - "shuffled": Whether boss list was shuffled (affects RNG state)
    """
    # Act 4 has fixed encounters - no RNG needed
    if act == 4:
        monsters, elites, boss = generate_ending_encounters()
        return {
            "monsters": monsters,
            "elites": elites,
            "boss": boss,
            "monster_rng_counter": monster_rng_counter,  # No RNG consumed
            "fixed": True,
            "shuffled": False,
            "key_requirement": ACT_4_KEYS,
        }

    seed_long = seed_to_long(seed) if isinstance(seed, str) else seed

    # Create RNG at the given counter state
    rng = Random(seed_long, monster_rng_counter)

    if act == 1:
        monsters, elites, boss = generate_exordium_encounters(rng, all_bosses_seen)
    elif act == 2:
        monsters, elites, boss = generate_city_encounters(rng, all_bosses_seen)
    elif act == 3:
        monsters, elites, boss = generate_beyond_encounters(rng, all_bosses_seen)
    else:
        raise ValueError(f"Invalid act: {act}. Must be 1, 2, 3, or 4.")

    return {
        "monsters": monsters,
        "elites": elites,
        "boss": boss,
        "monster_rng_counter": rng.counter,
        "shuffled": all_bosses_seen,
    }


def predict_all_acts(
    seed: str,
    include_act4: bool = True,
    all_bosses_seen: bool = True,
) -> Dict:
    """
    Predict encounters for all acts.

    Java Behavior (from Exordium/TheCity/TheBeyond.initializeBoss):
    - If daily run OR all bosses seen: shuffle boss list with monsterRng.randomLong()
    - If specific boss not seen: use deterministic boss (NO RNG consumed)

    Args:
        seed: Seed string (e.g., "1ABCD")
        include_act4: Whether to include Act 4 (default True)
        all_bosses_seen: Whether player has seen all bosses (default True for A20)

    Returns:
        Dictionary with act1, act2, act3, act4 keys, each containing
        monsters, elites, boss, and the RNG counter after that act.
        Act 4 has fixed encounters (no RNG).
    """
    result = {}
    counter = 0

    acts = [1, 2, 3, 4] if include_act4 else [1, 2, 3]

    for act in acts:
        act_data = predict_act_encounters(seed, act, counter, all_bosses_seen)
        result[f"act{act}"] = act_data
        counter = act_data["monster_rng_counter"]

    return result


def get_monsterrng_calls_for_act(act: int) -> Dict:
    """
    Get information about RNG calls made during act generation.

    This is useful for understanding how the counter advances.

    Returns dict with:
    - weak_count: Number of weak monsters generated
    - strong_count: Number of strong monsters (including first with exclusions)
    - elite_count: Number of elite monsters
    - boss_calls: Number of RNG calls for boss shuffle (1 randomLong)
    - min_calls: Minimum total calls (no rerolls)
    - typical_rerolls: Estimated typical rerolls due to duplicate prevention
    """
    # Act 4 uses no RNG - all encounters are fixed
    if act == 4:
        return {
            "weak_count": 0,
            "strong_count": 0,
            "elite_count": 1,  # Fixed: Spire Shield and Spire Spear
            "boss_calls": 0,  # No shuffle needed - Corrupt Heart is fixed
            "min_calls": 0,
            "note": "Act 4 has fixed encounters - no RNG is consumed",
            "fixed": True,
        }

    if act == 1:
        weak = 3
        strong = 12 + 1  # +1 for first strong with exclusions
        elite = 10
    elif act in (2, 3):
        weak = 2
        strong = 12 + 1
        elite = 10
    else:
        raise ValueError(f"Invalid act: {act}. Must be 1, 2, 3, or 4.")

    min_calls = weak + strong + elite + 1  # +1 for boss randomLong

    return {
        "weak_count": weak,
        "strong_count": strong,
        "elite_count": elite,
        "boss_calls": 1,
        "min_calls": min_calls,
        "note": "Actual calls may be higher due to rerolls for duplicate prevention",
    }


# ============================================================================
# ENEMY HP GENERATION
# ============================================================================

# HP ranges from decompiled source (base values, no ascension modifier yet)
ENEMY_HP_RANGES = {
    # Act 1 weak
    "Cultist": (48, 54),
    "Jaw Worm": (40, 44),
    "2 Louse": None,  # Multiple enemies with individual HP
    "Small Slimes": None,

    # Act 1 strong
    "Blue Slaver": (46, 50),
    "Red Slaver": (46, 50),
    "Looter": (44, 48),
    "Gremlin Gang": None,  # Multiple enemies
    "Large Slime": None,  # Splits
    "Lots of Slimes": None,
    "Exordium Thugs": None,
    "Exordium Wildlife": None,
    "3 Louse": None,
    "2 Fungi Beasts": None,

    # Act 1 elites
    "Gremlin Nob": (82, 86),
    "Lagavulin": (109, 111),
    "3 Sentries": None,  # 38-42 each

    # Individual enemies
    "Louse_Normal": (10, 15),
    "Louse_Large": (11, 17),
    "Spike_Slime_S": (10, 14),
    "Acid_Slime_S": (8, 12),
    "Spike_Slime_M": (28, 32),
    "Acid_Slime_M": (28, 32),
    "Spike_Slime_L": (64, 70),
    "Acid_Slime_L": (65, 69),
    "Sentry": (38, 42),
    "Fungi Beast": (22, 28),
    "Gremlin_Fat": (13, 17),
    "Gremlin_Sneaky": (10, 14),
    "Gremlin_Mad": (12, 16),
    "Gremlin_Wizard": (12, 14),
    "Gremlin_Shield": (12, 15),

    # Act 4 enemies (The Ending)
    # Note: Act 4 enemies have fixed HP (no range)
    "Spire Shield": (110, 110),  # A8+: (125, 125)
    "Spire Spear": (160, 160),  # A8+: (180, 180)
    "Corrupt Heart": (750, 750),  # A9+: (800, 800)
}


def get_enemy_hp(enemy_name: str, rng: Random, ascension: int = 0) -> int:
    """
    Get HP for a single enemy using monsterHpRng.

    Args:
        enemy_name: Internal enemy name
        rng: Monster HP RNG stream (monsterHpRng)
        ascension: Ascension level (affects HP at A7+)

    Returns:
        HP value
    """
    hp_range = ENEMY_HP_RANGES.get(enemy_name)

    if hp_range is None:
        return 0  # Multi-enemy encounter, handled separately

    min_hp, max_hp = hp_range

    # Roll HP
    hp = rng.random_int_range(min_hp, max_hp)

    # A7+ increases HP by ~7%
    if ascension >= 7:
        hp = int(hp * 1.07)

    return hp


# ============================================================================
# BOSS PREDICTION API
# ============================================================================

def predict_all_bosses(
    seed,
    ascension: int = 0,
    include_act4: bool = True,
    all_bosses_seen: bool = True,
) -> Dict[int, str]:
    """
    Predict bosses for all acts (1-3, optionally 4).

    This is a convenience function that returns just the boss for each act.
    Uses the full encounter generation to properly advance the monsterRng.

    Args:
        seed: Seed string (e.g., "ABC123") or numeric seed
        ascension: Ascension level (currently unused, for API compatibility)
        include_act4: Whether to include Act 4 (default True)
        all_bosses_seen: Whether player has seen all bosses (default True for A20)

    Returns:
        Dictionary mapping act number to boss name: {1: boss1, 2: boss2, 3: boss3, 4: boss4}
    """
    if isinstance(seed, str):
        seed_long = seed_to_long(seed)
    else:
        seed_long = seed

    monster_rng = Random(seed_long)
    bosses = {}

    # Act 1
    _, _, boss1 = generate_exordium_encounters(monster_rng, all_bosses_seen)
    bosses[1] = boss1

    # Act 2
    _, _, boss2 = generate_city_encounters(monster_rng, all_bosses_seen)
    bosses[2] = boss2

    # Act 3
    _, _, boss3 = generate_beyond_encounters(monster_rng, all_bosses_seen)
    bosses[3] = boss3

    # Act 4 - Fixed boss
    if include_act4:
        bosses[4] = ENDING_BOSS

    return bosses


def predict_all_bosses_extended(
    seed,
    ascension: int = 0,
    include_act4: bool = True,
    all_bosses_seen: bool = True,
) -> Dict[int, List[str]]:
    """
    Predict bosses for all acts, returning lists to support A20 double boss.

    This is the main API used by server.py for boss predictions.

    Args:
        seed: Seed string (e.g., "ABC123") or numeric seed
        ascension: Ascension level (A20 has 2 bosses in Act 3)
        include_act4: Whether to include Act 4 (default True)
        all_bosses_seen: Whether player has seen all bosses (default True for A20)

    Returns:
        Dictionary mapping act number to list of boss names:
        {1: [boss1], 2: [boss2], 3: [boss3] or [boss3, boss3_second], 4: ["Corrupt Heart"]}
    """
    if isinstance(seed, str):
        seed_long = seed_to_long(seed)
    else:
        seed_long = seed

    monster_rng = Random(seed_long)
    bosses = {}

    # Act 1
    _, _, boss1 = generate_exordium_encounters(monster_rng, all_bosses_seen)
    bosses[1] = [boss1]

    # Act 2
    _, _, boss2 = generate_city_encounters(monster_rng, all_bosses_seen)
    bosses[2] = [boss2]

    # Act 3 - need to get full shuffle order for A20 double boss
    # We need to directly access the shuffled boss list for A20
    monster_list: List[str] = []
    elite_list: List[str] = []

    # Weak enemies (floors 1-2)
    weak_pool = get_beyond_weak_pool()
    populate_monster_list(monster_list, weak_pool, monster_rng, 2)

    # First strong enemy with exclusions
    strong_pool = get_beyond_strong_pool()
    exclusions = get_beyond_exclusions(monster_list[-1])
    populate_first_strong_enemy(monster_list, strong_pool, monster_rng, exclusions)

    # Remaining strong enemies
    populate_monster_list(monster_list, strong_pool, monster_rng, 12)

    # Elite enemies
    elite_pool = get_beyond_elite_pool()
    populate_monster_list(elite_list, elite_pool, monster_rng, 10, is_elite=True)

    # Boss shuffle - get full shuffled list for A20
    shuffled_bosses = shuffle_bosses(BEYOND_BOSSES.copy(), monster_rng, should_shuffle=all_bosses_seen)

    if ascension >= 20 and len(shuffled_bosses) > 1:
        # A20 has two bosses in Act 3
        bosses[3] = [shuffled_bosses[0], shuffled_bosses[1]]
    else:
        bosses[3] = [shuffled_bosses[0]]

    # Act 4 - Fixed boss
    if include_act4:
        bosses[4] = [ENDING_BOSS]

    return bosses


# ============================================================================
# TESTING
# ============================================================================

if __name__ == "__main__":
    seed_str = "1ABCD"

    print(f"=== Encounter Prediction for Seed {seed_str} ===\n")

    all_acts = predict_all_acts(seed_str, include_act4=True)

    act_names = {1: "Exordium", 2: "The City", 3: "The Beyond", 4: "The Ending"}

    for act_num in [1, 2, 3, 4]:
        act_key = f"act{act_num}"
        act_data = all_acts[act_key]

        print(f"--- Act {act_num}: {act_names[act_num]} ---")
        print(f"Boss: {act_data['boss']}")

        if act_data.get('fixed'):
            print("(Fixed encounters - no RNG used)")
            if act_data.get('key_requirement'):
                print(f"Key requirement: {', '.join(act_data['key_requirement'])}")
        else:
            print(f"RNG Counter after act: {act_data['monster_rng_counter']}")

        if act_data['monsters']:
            print(f"\nMonster List ({len(act_data['monsters'])}):")
            for i, m in enumerate(act_data['monsters'], 1):
                print(f"  {i:2}. {m}")
        else:
            print("\nMonster List: None (no normal encounters)")

        print(f"\nElite List ({len(act_data['elites'])}):")
        for i, m in enumerate(act_data['elites'], 1):
            print(f"  {i:2}. {m}")

        print()
