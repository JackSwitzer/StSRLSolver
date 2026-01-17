"""
Slay the Spire - Encounter Generation System

Replicates the exact encounter generation algorithm from the decompiled source.

Key mechanics:
1. Monsters are pre-generated at dungeon creation using monsterRng
2. Each act has weak, strong, and elite pools with weighted probabilities
3. No back-to-back repeats (or 2-back for non-elites)
4. Weak encounters for floors 1-3, strong for 4+

RNG Usage:
- monsterRng.random() returns float [0, 1) for weighted selection
"""

import os
import importlib.util
from dataclasses import dataclass
from typing import List, Dict, Tuple, Optional


# Load RNG module
_core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

def _load_module(name: str, filepath: str):
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

_rng_module = _load_module("rng", os.path.join(_core_dir, "state", "rng.py"))
Random = _rng_module.Random


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


def generate_exordium_encounters(rng: Random) -> Tuple[List[str], List[str]]:
    """
    Generate all Act 1 encounters.

    Returns:
        (normal_encounters, elite_encounters)
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

    # Remaining strong enemies (11 more for total of 12)
    populate_monster_list(monster_list, strong_pool, rng, 11)

    # Elite enemies
    elite_pool = get_exordium_elite_pool()
    populate_monster_list(elite_list, elite_pool, rng, 10, is_elite=True)

    return monster_list, elite_list


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
# TESTING
# ============================================================================

if __name__ == "__main__":
    from ..state.rng import seed_to_long

    seed_str = "1ABCD"
    seed = seed_to_long(seed_str)

    print(f"=== Encounter Generation for Seed {seed_str} ===\n")

    monster_rng = Random(seed)

    normal, elite = generate_exordium_encounters(monster_rng)

    print(f"Normal encounters ({len(normal)}):")
    for i, m in enumerate(normal, 1):
        floor_type = "WEAK" if i <= 3 else "STRONG"
        print(f"  {i}. {m} ({floor_type})")

    print(f"\nElite encounters ({len(elite)}):")
    for i, m in enumerate(elite, 1):
        print(f"  {i}. {m}")

    # Test HP generation
    print(f"\n=== HP Generation ===")
    hp_rng = Random(seed + 1)  # monsterHpRng is seeded with seed + floorNum

    for enemy in ["Jaw Worm", "Cultist", "Gremlin Nob"]:
        hp = get_enemy_hp(enemy, Random(seed + 1))  # Fresh RNG for each test
        print(f"  {enemy}: {hp} HP")
