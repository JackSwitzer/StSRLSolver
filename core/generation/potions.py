"""
Slay the Spire - Potion Drop Prediction System

Implements exact potion drop prediction matching the game's algorithm:
1. Drop check: potionRng.random(0, 99) < chance
2. Rarity roll: potionRng.random(0, 99) to determine COMMON/UNCOMMON/RARE
3. Selection: Loop potionRng.random(pool.size()-1) until matching rarity

This module predicts:
- Whether a potion will drop
- Which specific potion will be selected
- The new blizzard modifier state

Reference: AbstractRoom.addPotionToRewards() and AbstractDungeon.returnRandomPotion()
"""

from dataclasses import dataclass
from typing import Optional, List, Tuple
import os
import importlib.util

# Setup import path
_core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def _load_module(name: str, filepath: str):
    """Load a module directly from file path."""
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


# Load dependencies
_rng_module = _load_module("rng", os.path.join(_core_dir, "state", "rng.py"))
Random = _rng_module.Random
seed_to_long = _rng_module.seed_to_long

_potions_module = _load_module("potions", os.path.join(_core_dir, "content", "potions.py"))
Potion = _potions_module.Potion
PotionRarity = _potions_module.PotionRarity
PlayerClass = _potions_module.PlayerClass
ALL_POTIONS = _potions_module.ALL_POTIONS


# ============================================================================
# POTION POOL DEFINITION - EXACT GAME ORDER
# ============================================================================

# The exact order potions are added to PotionHelper.potions ArrayList
# Class-specific potions come first, then universal potions
# This order is CRITICAL for seed-deterministic selection

WATCHER_POTION_POOL = [
    # Watcher-specific (lines 101-105 in PotionHelper.java)
    "BottledMiracle",
    "StancePotion",
    "Ambrosia",
    # Universal potions (lines 122-151)
    "Block Potion",
    "Dexterity Potion",
    "Energy Potion",
    "Explosive Potion",
    "Fire Potion",
    "Strength Potion",
    "Swift Potion",
    "Weak Potion",
    "FearPotion",
    "AttackPotion",
    "SkillPotion",
    "PowerPotion",
    "ColorlessPotion",
    "SteroidPotion",
    "SpeedPotion",
    "BlessingOfTheForge",
    "Regen Potion",
    "Ancient Potion",
    "LiquidBronze",
    "GamblersBrew",
    "EssenceOfSteel",
    "DuplicationPotion",
    "DistilledChaos",
    "LiquidMemories",
    "CultistPotion",
    "Fruit Juice",
    "SneckoOil",
    "FairyPotion",
    "SmokeBomb",
    "EntropicBrew",
]

IRONCLAD_POTION_POOL = [
    # Ironclad-specific
    "BloodPotion",
    "ElixirPotion",
    "HeartOfIron",
    # Universal potions (same order as above)
    "Block Potion",
    "Dexterity Potion",
    "Energy Potion",
    "Explosive Potion",
    "Fire Potion",
    "Strength Potion",
    "Swift Potion",
    "Weak Potion",
    "FearPotion",
    "AttackPotion",
    "SkillPotion",
    "PowerPotion",
    "ColorlessPotion",
    "SteroidPotion",
    "SpeedPotion",
    "BlessingOfTheForge",
    "Regen Potion",
    "Ancient Potion",
    "LiquidBronze",
    "GamblersBrew",
    "EssenceOfSteel",
    "DuplicationPotion",
    "DistilledChaos",
    "LiquidMemories",
    "CultistPotion",
    "Fruit Juice",
    "SneckoOil",
    "FairyPotion",
    "SmokeBomb",
    "EntropicBrew",
]

SILENT_POTION_POOL = [
    # Silent-specific
    "Poison Potion",
    "CunningPotion",
    "GhostInAJar",
    # Universal potions
    "Block Potion",
    "Dexterity Potion",
    "Energy Potion",
    "Explosive Potion",
    "Fire Potion",
    "Strength Potion",
    "Swift Potion",
    "Weak Potion",
    "FearPotion",
    "AttackPotion",
    "SkillPotion",
    "PowerPotion",
    "ColorlessPotion",
    "SteroidPotion",
    "SpeedPotion",
    "BlessingOfTheForge",
    "Regen Potion",
    "Ancient Potion",
    "LiquidBronze",
    "GamblersBrew",
    "EssenceOfSteel",
    "DuplicationPotion",
    "DistilledChaos",
    "LiquidMemories",
    "CultistPotion",
    "Fruit Juice",
    "SneckoOil",
    "FairyPotion",
    "SmokeBomb",
    "EntropicBrew",
]

DEFECT_POTION_POOL = [
    # Defect-specific
    "FocusPotion",
    "PotionOfCapacity",
    "EssenceOfDarkness",
    # Universal potions
    "Block Potion",
    "Dexterity Potion",
    "Energy Potion",
    "Explosive Potion",
    "Fire Potion",
    "Strength Potion",
    "Swift Potion",
    "Weak Potion",
    "FearPotion",
    "AttackPotion",
    "SkillPotion",
    "PowerPotion",
    "ColorlessPotion",
    "SteroidPotion",
    "SpeedPotion",
    "BlessingOfTheForge",
    "Regen Potion",
    "Ancient Potion",
    "LiquidBronze",
    "GamblersBrew",
    "EssenceOfSteel",
    "DuplicationPotion",
    "DistilledChaos",
    "LiquidMemories",
    "CultistPotion",
    "Fruit Juice",
    "SneckoOil",
    "FairyPotion",
    "SmokeBomb",
    "EntropicBrew",
]


def get_potion_pool_for_class(player_class: str) -> List[str]:
    """
    Get the potion pool in exact game order for a player class.

    Args:
        player_class: "WATCHER", "IRONCLAD", "SILENT", or "DEFECT"

    Returns:
        List of potion IDs in the exact order they appear in the game
    """
    pools = {
        "WATCHER": WATCHER_POTION_POOL,
        "IRONCLAD": IRONCLAD_POTION_POOL,
        "SILENT": SILENT_POTION_POOL,
        "THE_SILENT": SILENT_POTION_POOL,
        "DEFECT": DEFECT_POTION_POOL,
    }
    return pools.get(player_class, WATCHER_POTION_POOL)


# ============================================================================
# CONSTANTS
# ============================================================================

# Rarity thresholds from PotionHelper.java
POTION_COMMON_CHANCE = 65  # roll < 65 = COMMON
POTION_UNCOMMON_CHANCE = 25  # roll < 90 = UNCOMMON (65+25)
# else RARE

# Base drop chance from AbstractRoom.addPotionToRewards()
BASE_DROP_CHANCE = 40

# Blizzard mod step
BLIZZARD_MOD_STEP = 10


# ============================================================================
# PREDICTION RESULT
# ============================================================================

@dataclass
class PotionPrediction:
    """Result of potion drop prediction."""
    will_drop: bool
    potion_id: Optional[str]
    potion: Optional[Potion]
    rarity: Optional[PotionRarity]
    new_blizzard_mod: int
    new_potion_counter: int

    # Debug info
    drop_roll: Optional[int] = None
    drop_chance: Optional[int] = None
    rarity_roll: Optional[int] = None
    selection_attempts: int = 0


# ============================================================================
# CORE PREDICTION FUNCTIONS
# ============================================================================

def _get_potion_rarity(potion_id: str) -> PotionRarity:
    """Get the rarity of a potion by ID."""
    potion = ALL_POTIONS.get(potion_id)
    if potion:
        return potion.rarity
    # Fallback - shouldn't happen
    return PotionRarity.COMMON


def predict_potion_drop(
    potion_rng: Random,
    blizzard_mod: int = 0,
    player_class: str = "WATCHER",
    room_type: str = "monster",
    has_white_beast_statue: bool = False,
    has_sozu: bool = False,
    current_rewards: int = 0,
    monsters_escaped: bool = False,
) -> PotionPrediction:
    """
    Predict whether a potion will drop and which one.

    This function modifies the potion_rng state to match what the game does.

    Algorithm:
    1. Calculate drop chance: 40 + blizzardMod (or 100 with White Beast Statue)
    2. Roll potionRng.random(0, 99) for drop check
    3. If drop: roll potionRng.random(0, 99) for rarity
    4. Loop calling potionRng.random(pool.size()-1) until matching rarity

    Args:
        potion_rng: The potionRng Random instance (will be mutated!)
        blizzard_mod: Current blizzard modifier (starts at 0)
        player_class: Player class for class-specific potions
        room_type: "monster", "elite", or "event"
        has_white_beast_statue: Guarantees potion drop
        has_sozu: Prevents potion drops entirely
        current_rewards: Number of rewards already (4+ = no potion)
        monsters_escaped: True if monsters escaped (e.g., Smoke Bomb used)

    Returns:
        PotionPrediction with all prediction details
    """
    # Initial counter for tracking
    initial_counter = potion_rng.counter

    # Sozu prevents all potion drops (no RNG consumed)
    if has_sozu:
        return PotionPrediction(
            will_drop=False,
            potion_id=None,
            potion=None,
            rarity=None,
            new_blizzard_mod=blizzard_mod,
            new_potion_counter=potion_rng.counter,
        )

    # Calculate drop chance
    # Java logic: MonsterRoomElite always gets 40 + blizzardMod
    # MonsterRoom only gets chance if monsters did NOT escape
    # EventRoom always gets 40 + blizzardMod
    if has_white_beast_statue:
        chance = 100
    elif room_type == "elite":
        chance = BASE_DROP_CHANCE + blizzard_mod
        chance = max(0, chance)
    elif room_type == "monster":
        if monsters_escaped:
            # Monsters escaped - chance stays at 0, but RNG still consumed
            chance = 0
        else:
            chance = BASE_DROP_CHANCE + blizzard_mod
            chance = max(0, chance)
    elif room_type == "event":
        chance = BASE_DROP_CHANCE + blizzard_mod
        chance = max(0, chance)
    else:
        # Unknown room type - no drop, no RNG consumed
        return PotionPrediction(
            will_drop=False,
            potion_id=None,
            potion=None,
            rarity=None,
            new_blizzard_mod=blizzard_mod,
            new_potion_counter=potion_rng.counter,
        )

    # Check rewards cap (4+ rewards = no potion)
    if current_rewards >= 4:
        chance = 0

    # Roll for drop - ALWAYS consumes RNG even if chance is 0
    drop_roll = potion_rng.random(99)  # random(0, 99) in Java

    if drop_roll >= chance:
        # No drop - blizzard mod increases
        return PotionPrediction(
            will_drop=False,
            potion_id=None,
            potion=None,
            rarity=None,
            new_blizzard_mod=blizzard_mod + BLIZZARD_MOD_STEP,
            new_potion_counter=potion_rng.counter,
            drop_roll=drop_roll,
            drop_chance=chance,
        )

    # Drop confirmed - blizzard mod decreases
    new_blizzard_mod = blizzard_mod - BLIZZARD_MOD_STEP

    # Roll for rarity
    rarity_roll = potion_rng.random(99)

    if rarity_roll < POTION_COMMON_CHANCE:
        target_rarity = PotionRarity.COMMON
    elif rarity_roll < POTION_COMMON_CHANCE + POTION_UNCOMMON_CHANCE:
        target_rarity = PotionRarity.UNCOMMON
    else:
        target_rarity = PotionRarity.RARE

    # Select potion by looping until matching rarity
    potion_pool = get_potion_pool_for_class(player_class)
    pool_size = len(potion_pool)

    attempts = 0
    max_attempts = 1000  # Safety limit
    selected_id = None

    while attempts < max_attempts:
        idx = potion_rng.random(pool_size - 1)
        candidate_id = potion_pool[idx]
        candidate_rarity = _get_potion_rarity(candidate_id)
        attempts += 1

        if candidate_rarity == target_rarity:
            selected_id = candidate_id
            break

    # Get full potion object
    selected_potion = ALL_POTIONS.get(selected_id) if selected_id else None

    return PotionPrediction(
        will_drop=True,
        potion_id=selected_id,
        potion=selected_potion,
        rarity=target_rarity,
        new_blizzard_mod=new_blizzard_mod,
        new_potion_counter=potion_rng.counter,
        drop_roll=drop_roll,
        drop_chance=chance,
        rarity_roll=rarity_roll,
        selection_attempts=attempts,
    )


def predict_potion_from_seed(
    seed: str,
    potion_counter: int = 0,
    blizzard_mod: int = 0,
    player_class: str = "WATCHER",
    room_type: str = "monster",
    has_white_beast_statue: bool = False,
    has_sozu: bool = False,
    current_rewards: int = 0,
    monsters_escaped: bool = False,
) -> PotionPrediction:
    """
    Predict potion drop from a seed string and counter.

    Convenience wrapper that creates the RNG from seed + counter.

    Args:
        seed: Seed string (e.g., "ABC123XYZ")
        potion_counter: Current potionRng counter value
        blizzard_mod: Current blizzard modifier
        player_class: Player class
        room_type: Room type
        has_white_beast_statue: White Beast Statue relic
        has_sozu: Sozu relic
        current_rewards: Number of existing rewards
        monsters_escaped: True if monsters escaped (e.g., Smoke Bomb used)

    Returns:
        PotionPrediction with all details
    """
    seed_long = seed_to_long(seed)
    potion_rng = Random(seed_long, potion_counter)

    return predict_potion_drop(
        potion_rng=potion_rng,
        blizzard_mod=blizzard_mod,
        player_class=player_class,
        room_type=room_type,
        has_white_beast_statue=has_white_beast_statue,
        has_sozu=has_sozu,
        current_rewards=current_rewards,
        monsters_escaped=monsters_escaped,
    )


def predict_multiple_potion_drops(
    potion_rng: Random,
    num_combats: int,
    blizzard_mod: int = 0,
    player_class: str = "WATCHER",
    room_type: str = "monster",
    has_white_beast_statue: bool = False,
    has_sozu: bool = False,
    monsters_escaped: bool = False,
) -> List[PotionPrediction]:
    """
    Predict potion drops for multiple consecutive combats.

    Useful for simulating a sequence of fights and seeing what potions
    will be available.

    Args:
        potion_rng: The potionRng Random instance (will be mutated!)
        num_combats: Number of combats to simulate
        blizzard_mod: Starting blizzard modifier
        player_class: Player class
        room_type: Room type for all combats
        has_white_beast_statue: White Beast Statue relic
        has_sozu: Sozu relic
        monsters_escaped: True if monsters escaped (applies to all combats)

    Returns:
        List of PotionPrediction for each combat
    """
    predictions = []
    current_blizzard = blizzard_mod

    for _ in range(num_combats):
        pred = predict_potion_drop(
            potion_rng=potion_rng,
            blizzard_mod=current_blizzard,
            player_class=player_class,
            room_type=room_type,
            has_white_beast_statue=has_white_beast_statue,
            has_sozu=has_sozu,
            monsters_escaped=monsters_escaped,
        )
        predictions.append(pred)
        current_blizzard = pred.new_blizzard_mod

    return predictions


# ============================================================================
# HELPER FUNCTIONS FOR INTEGRATION
# ============================================================================

def get_potion_by_id(potion_id: str) -> Optional[Potion]:
    """Get a Potion object by its ID."""
    return ALL_POTIONS.get(potion_id)


def get_rng_calls_for_potion_selection(
    target_rarity: PotionRarity,
    player_class: str = "WATCHER",
    starting_seed0: int = None,
    starting_seed1: int = None,
) -> Tuple[int, str]:
    """
    Determine how many RNG calls are needed to select a potion of given rarity.

    This is useful for understanding RNG consumption patterns.

    Note: This creates a temporary RNG and doesn't affect any external state.

    Args:
        target_rarity: The rarity we're selecting for
        player_class: Player class
        starting_seed0: Optional seed0 for the RNG state
        starting_seed1: Optional seed1 for the RNG state

    Returns:
        Tuple of (num_calls, selected_potion_id)
    """
    # Use a dummy RNG if no state provided
    if starting_seed0 is None:
        temp_rng = Random(12345)
    else:
        from core.state.rng import XorShift128
        temp_rng = Random.__new__(Random)
        temp_rng._rng = XorShift128(starting_seed0, starting_seed1)
        temp_rng.counter = 0

    potion_pool = get_potion_pool_for_class(player_class)
    pool_size = len(potion_pool)

    calls = 0
    max_calls = 1000

    while calls < max_calls:
        idx = temp_rng.random(pool_size - 1)
        candidate_id = potion_pool[idx]
        candidate_rarity = _get_potion_rarity(candidate_id)
        calls += 1

        if candidate_rarity == target_rarity:
            return (calls, candidate_id)

    return (calls, None)


# ============================================================================
# TESTING
# ============================================================================

if __name__ == "__main__":
    print("=== Potion Drop Prediction Tests ===\n")

    # Test with known seed
    seed_str = "TESTSEED123"
    seed_long = seed_to_long(seed_str)
    print(f"Testing with seed: {seed_str} ({seed_long})\n")

    # Create potion RNG
    potion_rng = Random(seed_long)

    print("--- Simulating 10 Combat Potion Drops ---")
    blizzard = 0
    for i in range(10):
        pred = predict_potion_drop(
            potion_rng=potion_rng,
            blizzard_mod=blizzard,
            player_class="WATCHER",
            room_type="monster",
        )
        blizzard = pred.new_blizzard_mod

        if pred.will_drop:
            print(f"Combat {i+1}: DROP - {pred.potion_id} ({pred.rarity.name})")
            print(f"  Roll: {pred.drop_roll} < {pred.drop_chance}, "
                  f"Rarity roll: {pred.rarity_roll}, "
                  f"Selection attempts: {pred.selection_attempts}")
        else:
            print(f"Combat {i+1}: No drop (roll {pred.drop_roll} >= {pred.drop_chance})")
        print(f"  New blizzard mod: {blizzard}")

    print(f"\nFinal potionRng counter: {potion_rng.counter}")

    # Test with White Beast Statue
    print("\n--- With White Beast Statue (100% drop) ---")
    potion_rng2 = Random(seed_long)
    for i in range(3):
        pred = predict_potion_drop(
            potion_rng=potion_rng2,
            blizzard_mod=0,
            player_class="WATCHER",
            has_white_beast_statue=True,
        )
        print(f"Combat {i+1}: {pred.potion_id} ({pred.rarity.name})")

    # Test convenience function
    print("\n--- Using predict_potion_from_seed ---")
    pred = predict_potion_from_seed(
        seed="TESTSEED123",
        potion_counter=0,
        blizzard_mod=0,
        player_class="WATCHER",
    )
    print(f"First potion: {'DROP - ' + pred.potion_id if pred.will_drop else 'No drop'}")
    print(f"Counter after: {pred.new_potion_counter}")

    # Test potion pool
    print("\n--- Watcher Potion Pool (first 10) ---")
    pool = get_potion_pool_for_class("WATCHER")
    for i, pid in enumerate(pool[:10]):
        rarity = _get_potion_rarity(pid)
        print(f"  {i}: {pid} ({rarity.name})")
    print(f"Total pool size: {len(pool)}")
