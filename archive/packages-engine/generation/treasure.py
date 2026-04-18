"""
Slay the Spire - Treasure Room (Chest) Prediction

Implements treasure room reward prediction using treasureRng.

Treasure Room Mechanics (from decompiled ChestRoom.java):
1. First treasureRng call: Chest type (roll 0-99)
   - < 50: Small chest
   - < 83: Medium chest
   - else: Large chest

2. Second treasureRng call: Relic tier + gold decision (same roll)
   Small chest:  75% common, 25% uncommon, 0% rare, 50% gold (25g base)
   Medium chest: 35% common, 50% uncommon, 15% rare, 35% gold (50g base)
   Large chest:  0% common, 75% uncommon, 25% rare, 50% gold (75g base)

3. Third treasureRng call (if gold): Gold variance (0.9x to 1.1x)

Relic Selection:
- Relics come from pre-shuffled pools created at dungeon init
- Pools shuffled using relicRng.randomLong() (5 calls for 5 tiers)
- Selection pops from front of pool (index 0)
- When pool exhausted, falls back to higher tier

N'loth's Hungry Face Relic:
- First non-boss chest is empty
- Future chests have improved tier (Medium -> Large, Small -> Medium)
"""

import os
import importlib.util
from dataclasses import dataclass
from typing import List, Dict, Optional, Tuple
from enum import Enum


# Load modules
_core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))


def _load_module(name: str, filepath: str):
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


_rng_mod = _load_module("rng", os.path.join(_core_dir, "state", "rng.py"))
Random = _rng_mod.Random
seed_to_long = _rng_mod.seed_to_long

_relics_mod = _load_module("relics_gen", os.path.join(_core_dir, "generation", "relics.py"))
predict_all_relic_pools = _relics_mod.predict_all_relic_pools
RelicPools = _relics_mod.RelicPools


# ============================================================================
# CONSTANTS
# ============================================================================

class ChestType(Enum):
    """Chest types with their properties."""
    SMALL = "Small"
    MEDIUM = "Medium"
    LARGE = "Large"


# Chest type thresholds (roll 0-99)
CHEST_TYPE_THRESHOLDS = {
    "small": 50,   # < 50 = Small
    "medium": 83,  # < 83 = Medium, else Large
}

# Relic tier thresholds by chest type (roll 0-99)
# Format: {"common": threshold, "uncommon": threshold}
# If roll < common -> COMMON
# If roll < uncommon -> UNCOMMON
# Else -> RARE
CHEST_RELIC_THRESHOLDS = {
    ChestType.SMALL: {"common": 75, "uncommon": 100},   # 75% common, 25% uncommon, 0% rare
    ChestType.MEDIUM: {"common": 35, "uncommon": 85},   # 35% common, 50% uncommon, 15% rare
    ChestType.LARGE: {"common": 0, "uncommon": 75},     # 0% common, 75% uncommon, 25% rare
}

# Gold chance by chest type (uses same roll as relic tier)
CHEST_GOLD_CHANCE = {
    ChestType.SMALL: 50,   # 50% chance
    ChestType.MEDIUM: 35,  # 35% chance
    ChestType.LARGE: 50,   # 50% chance
}

# Base gold amounts by chest type
CHEST_GOLD_BASE = {
    ChestType.SMALL: 25,
    ChestType.MEDIUM: 50,
    ChestType.LARGE: 75,
}

# Gold variance multiplier range
GOLD_VARIANCE_MIN = 0.9
GOLD_VARIANCE_MAX = 1.1


# ============================================================================
# DATA CLASSES
# ============================================================================

@dataclass
class TreasureReward:
    """Complete treasure room reward prediction."""
    chest_type: ChestType
    relic_tier: str  # "COMMON", "UNCOMMON", or "RARE"
    has_gold: bool
    gold_amount: int  # 0 if no gold


@dataclass
class ChestPrediction:
    """Full chest prediction including specific relic."""
    chest_type: ChestType
    relic_tier: str
    relic_name: str
    has_gold: bool
    gold_amount: int


# ============================================================================
# TREASURE RNG FUNCTIONS
# ============================================================================

def _determine_chest_type(roll: int) -> ChestType:
    """
    Determine chest type from roll (0-99).

    From ChestRoom.onChestOpen():
    - < 50: Small
    - < 83: Medium
    - else: Large
    """
    if roll < CHEST_TYPE_THRESHOLDS["small"]:
        return ChestType.SMALL
    elif roll < CHEST_TYPE_THRESHOLDS["medium"]:
        return ChestType.MEDIUM
    else:
        return ChestType.LARGE


def _determine_relic_tier(roll: int, chest_type: ChestType) -> str:
    """
    Determine relic tier from roll (0-99) based on chest type.

    Returns "COMMON", "UNCOMMON", or "RARE"
    """
    thresholds = CHEST_RELIC_THRESHOLDS[chest_type]

    if roll < thresholds["common"]:
        return "COMMON"
    elif roll < thresholds["uncommon"]:
        return "UNCOMMON"
    else:
        return "RARE"


def _check_gold(roll: int, chest_type: ChestType) -> bool:
    """
    Check if chest contains gold based on roll and chest type.

    Uses the same roll as relic tier determination.
    """
    return roll < CHEST_GOLD_CHANCE[chest_type]


def _calculate_gold(rng: Random, chest_type: ChestType) -> int:
    """
    Calculate gold amount with variance.

    Base gold * random(0.9, 1.1)
    """
    base = CHEST_GOLD_BASE[chest_type]
    variance = rng.random_float_range(GOLD_VARIANCE_MIN, GOLD_VARIANCE_MAX)
    return int(base * variance)


# ============================================================================
# PREDICTION FUNCTIONS
# ============================================================================

def predict_chest(
    seed: int,
    treasure_counter: int,
    has_nloths_face: bool = False,
    nloths_face_triggered: bool = False,
) -> TreasureReward:
    """
    Predict treasure chest contents for a given seed and counter.

    Args:
        seed: The game seed (long value, not string)
        treasure_counter: Current treasureRng counter (number of calls made)
        has_nloths_face: Whether player has N'loth's Hungry Face relic
        nloths_face_triggered: Whether the first empty chest has already occurred

    Returns:
        TreasureReward with chest type, relic tier, gold info

    RNG Flow:
    1. treasureRng.random(99) -> chest type
    2. treasureRng.random(99) -> relic tier + gold check
    3. if gold: treasureRng.random_float_range(0.9, 1.1) -> gold variance
    """
    # Create RNG at the specified counter
    treasure_rng = Random(seed, treasure_counter)

    # First call: Chest type
    chest_roll = treasure_rng.random_int(99)
    chest_type = _determine_chest_type(chest_roll)

    # N'loth's Hungry Face: upgrade chest tier (after first empty chest)
    if has_nloths_face and nloths_face_triggered:
        if chest_type == ChestType.SMALL:
            chest_type = ChestType.MEDIUM
        elif chest_type == ChestType.MEDIUM:
            chest_type = ChestType.LARGE
        # Large stays Large

    # N'loth's Hungry Face: first chest is empty
    if has_nloths_face and not nloths_face_triggered:
        return TreasureReward(
            chest_type=chest_type,
            relic_tier="NONE",
            has_gold=False,
            gold_amount=0,
        )

    # Second call: Relic tier and gold (same roll)
    tier_gold_roll = treasure_rng.random_int(99)
    relic_tier = _determine_relic_tier(tier_gold_roll, chest_type)
    has_gold = _check_gold(tier_gold_roll, chest_type)

    # Third call (conditional): Gold amount
    gold_amount = 0
    if has_gold:
        gold_amount = _calculate_gold(treasure_rng, chest_type)

    return TreasureReward(
        chest_type=chest_type,
        relic_tier=relic_tier,
        has_gold=has_gold,
        gold_amount=gold_amount,
    )


def predict_chest_relic(
    seed: int,
    tier: str,
    player_class: str = "WATCHER",
    relics_obtained: Optional[Dict[str, int]] = None,
) -> str:
    """
    Predict the specific relic from a chest based on shuffled pools.

    Args:
        seed: The game seed (long value)
        tier: Relic tier ("COMMON", "UNCOMMON", or "RARE")
        player_class: Player class for class-specific relics
        relics_obtained: Dict mapping tier -> number already taken from that pool
                        e.g., {"COMMON": 2, "UNCOMMON": 1} means 2 common and 1 uncommon
                        relics have been obtained from chests so far

    Returns:
        The relic ID that would be obtained

    Note: Relic pools are shuffled at dungeon init using relicRng.
    Each pool is consumed front-to-back (index 0 first).
    """
    if relics_obtained is None:
        relics_obtained = {}

    # Get the shuffled pools
    pools = predict_all_relic_pools(seed, player_class)

    # Determine which index to take from
    tier_index = relics_obtained.get(tier, 0)

    # Get the appropriate pool
    if tier == "COMMON":
        pool = pools.common
    elif tier == "UNCOMMON":
        pool = pools.uncommon
    elif tier == "RARE":
        pool = pools.rare
    else:
        return "Circlet"  # Fallback

    # Check if pool is exhausted
    if tier_index >= len(pool):
        # Fall back to higher tier
        if tier == "COMMON":
            return predict_chest_relic(seed, "UNCOMMON", player_class, relics_obtained)
        elif tier == "UNCOMMON":
            return predict_chest_relic(seed, "RARE", player_class, relics_obtained)
        else:
            return "Circlet"  # All pools exhausted

    return pool[tier_index]


def predict_full_chest(
    seed: int,
    treasure_counter: int,
    player_class: str = "WATCHER",
    relics_obtained: Optional[Dict[str, int]] = None,
    has_nloths_face: bool = False,
    nloths_face_triggered: bool = False,
) -> ChestPrediction:
    """
    Predict complete chest contents including specific relic.

    Combines predict_chest and predict_chest_relic for convenience.

    Args:
        seed: The game seed (long value)
        treasure_counter: Current treasureRng counter
        player_class: Player class
        relics_obtained: Dict mapping tier -> count already obtained
        has_nloths_face: N'loth's Hungry Face relic
        nloths_face_triggered: Whether first empty chest occurred

    Returns:
        ChestPrediction with all details
    """
    # Get base chest prediction
    reward = predict_chest(seed, treasure_counter, has_nloths_face, nloths_face_triggered)

    # Get specific relic if not empty
    if reward.relic_tier == "NONE":
        relic_name = "None"
    else:
        relic_name = predict_chest_relic(seed, reward.relic_tier, player_class, relics_obtained)

    return ChestPrediction(
        chest_type=reward.chest_type,
        relic_tier=reward.relic_tier,
        relic_name=relic_name,
        has_gold=reward.has_gold,
        gold_amount=reward.gold_amount,
    )


def predict_treasure_sequence(
    seed: int,
    num_chests: int,
    player_class: str = "WATCHER",
    starting_counter: int = 0,
    has_nloths_face: bool = False,
) -> List[ChestPrediction]:
    """
    Predict a sequence of treasure chests.

    Useful for planning routes through the map.

    Args:
        seed: The game seed (long value)
        num_chests: Number of chests to predict
        player_class: Player class
        starting_counter: Initial treasureRng counter
        has_nloths_face: N'loth's Hungry Face relic

    Returns:
        List of ChestPrediction objects
    """
    predictions = []
    relics_obtained: Dict[str, int] = {"COMMON": 0, "UNCOMMON": 0, "RARE": 0}
    counter = starting_counter
    nloths_triggered = False

    for _ in range(num_chests):
        pred = predict_full_chest(
            seed, counter, player_class, relics_obtained.copy(),
            has_nloths_face, nloths_triggered
        )
        predictions.append(pred)

        # Update state for next chest
        # Count RNG calls: 1 for chest type, 1 for tier/gold, 1 if gold
        counter += 2
        if pred.has_gold:
            counter += 1

        # Update relics obtained
        if pred.relic_tier in relics_obtained:
            relics_obtained[pred.relic_tier] += 1

        # Update N'loth's state
        if has_nloths_face and not nloths_triggered:
            nloths_triggered = True

    return predictions


# ============================================================================
# UTILITY FUNCTIONS
# ============================================================================

def get_treasure_counter_after_chest(
    has_gold: bool,
    starting_counter: int = 0,
) -> int:
    """
    Calculate treasureRng counter after opening a chest.

    Each chest consumes:
    - 1 call for chest type
    - 1 call for relic tier / gold check
    - 1 call for gold variance (only if gold drops)

    Args:
        has_gold: Whether the chest contained gold
        starting_counter: Counter before opening

    Returns:
        Counter after opening
    """
    calls = 2  # chest type + tier/gold
    if has_gold:
        calls += 1
    return starting_counter + calls


# ============================================================================
# TESTING
# ============================================================================

def main():
    """Test treasure prediction."""
    import sys

    seed_str = sys.argv[1] if len(sys.argv) > 1 else "TESTSEED"
    seed = seed_to_long(seed_str)

    print(f"{'='*60}")
    print(f"TREASURE PREDICTION FOR SEED: {seed_str}")
    print(f"Seed value: {seed}")
    print(f"{'='*60}")

    print("\n=== First 5 Treasure Chests ===")
    predictions = predict_treasure_sequence(seed, 5, "WATCHER")

    counter = 0
    for i, pred in enumerate(predictions, 1):
        gold_str = f", {pred.gold_amount}g" if pred.has_gold else ""
        print(f"\nChest {i} (counter={counter}):")
        print(f"  Type: {pred.chest_type.value}")
        print(f"  Relic: {pred.relic_name} ({pred.relic_tier})")
        print(f"  Gold: {pred.has_gold}{gold_str}")

        # Update counter
        counter += 2
        if pred.has_gold:
            counter += 1

    print("\n=== With N'loth's Hungry Face ===")
    predictions_nloth = predict_treasure_sequence(seed, 3, "WATCHER", has_nloths_face=True)

    for i, pred in enumerate(predictions_nloth, 1):
        gold_str = f", {pred.gold_amount}g" if pred.has_gold else ""
        print(f"\nChest {i}:")
        print(f"  Type: {pred.chest_type.value}")
        print(f"  Relic: {pred.relic_name} ({pred.relic_tier})")
        print(f"  Gold: {pred.has_gold}{gold_str}")


if __name__ == "__main__":
    main()
