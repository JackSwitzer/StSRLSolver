"""
Test different cardRng starting offsets to find the correct one.

The game likely consumes cardRng calls during Neow blessing selection
or other early-game processes before the first combat reward.

Actual game data for seed 1ABCD:
- Floor 1: Like Water (U), Bowling Bash (C), Deceive Reality (U)
- Floor 2: Sash Whip (C), Evaluate (C), Worship (U)
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

import importlib.util

def load_module(name, filepath):
    spec = importlib.util.spec_from_file_location(name, filepath)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module

core_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
rng_module = load_module("rng", os.path.join(core_dir, "state", "rng.py"))

Random = rng_module.Random
seed_to_long = rng_module.seed_to_long


def load_card_pools():
    """Load card pools in HashMap order."""
    _card_lib_module = load_module(
        "card_library_order",
        os.path.join(core_dir, "utils", "card_library_order.py")
    )
    return {
        "COMMON": _card_lib_module.get_watcher_pool_by_rarity("COMMON"),
        "UNCOMMON": _card_lib_module.get_watcher_pool_by_rarity("UNCOMMON"),
        "RARE": _card_lib_module.get_watcher_pool_by_rarity("RARE"),
    }


def simulate_card_reward(rng, pools, blizzard_offset):
    """Simulate one card reward (3 cards)."""
    RARE_THRESHOLD = 3
    UNCOMMON_THRESHOLD = 37

    cards = []
    cards_picked = set()

    for _ in range(3):
        # Roll rarity
        roll = rng.random(99)
        adjusted = roll + blizzard_offset

        if adjusted < RARE_THRESHOLD:
            rarity = "RARE"
            blizzard_offset = 5
            pool = pools["RARE"]
        elif adjusted < RARE_THRESHOLD + UNCOMMON_THRESHOLD:
            rarity = "UNCOMMON"
            pool = pools["UNCOMMON"]
        else:
            rarity = "COMMON"
            blizzard_offset -= 1
            pool = pools["COMMON"]

        # Pick card
        available = [c for c in pool if c not in cards_picked]
        idx = rng.random(len(available) - 1)
        card_id = available[idx]
        cards_picked.add(card_id)
        cards.append((card_id, rarity[0]))  # (id, first letter of rarity)

    return cards, blizzard_offset


# Card name to ID mapping (for verification)
NAME_TO_ID = {
    "Like Water": "LikeWater",
    "Bowling Bash": "BowlingBash",
    "Deceive Reality": "DeceiveReality",
    "Sash Whip": "SashWhip",
    "Evaluate": "Evaluate",
    "Worship": "Worship",
    "Fear No Evil": "FearNoEvil",
    "Empty Fist": "EmptyFist",
    "Foresight": "Wireheading",  # Java ID differs!
    "Wireheading": "Wireheading",
}

# Expected results
FLOOR_1_EXPECTED = [("LikeWater", "U"), ("BowlingBash", "C"), ("DeceiveReality", "U")]
FLOOR_2_EXPECTED = [("SashWhip", "C"), ("Evaluate", "C"), ("Worship", "U")]


def test_offset(offset):
    """Test with a given starting cardRng offset."""
    seed = seed_to_long("1ABCD")
    rng = Random(seed)

    # Skip ahead by offset
    for _ in range(offset):
        rng.random(99)

    pools = load_card_pools()
    blizzard = 5

    # Floor 1
    floor1, blizzard = simulate_card_reward(rng, pools, blizzard)
    # Floor 2
    floor2, blizzard = simulate_card_reward(rng, pools, blizzard)

    floor1_match = floor1 == FLOOR_1_EXPECTED
    floor2_match = floor2 == FLOOR_2_EXPECTED

    return floor1, floor2, floor1_match, floor2_match


def main():
    print("Testing different cardRng starting offsets...")
    print(f"Expected Floor 1: {FLOOR_1_EXPECTED}")
    print(f"Expected Floor 2: {FLOOR_2_EXPECTED}")
    print()

    pools = load_card_pools()
    print(f"UNCOMMON pool positions for verification:")
    print(f"  LikeWater: {pools['UNCOMMON'].index('LikeWater')}")
    print(f"  DeceiveReality: {pools['UNCOMMON'].index('DeceiveReality')}")
    print(f"  Worship: {pools['UNCOMMON'].index('Worship')}")
    print(f"COMMON pool positions:")
    print(f"  BowlingBash: {pools['COMMON'].index('BowlingBash')}")
    print(f"  SashWhip: {pools['COMMON'].index('SashWhip')}")
    print(f"  Evaluate: {pools['COMMON'].index('Evaluate')}")
    print()

    # Test offsets 0-20
    found_match = False
    for offset in range(50):
        floor1, floor2, f1_match, f2_match = test_offset(offset)

        if f1_match or f2_match:
            status = ""
            if f1_match:
                status += " FLOOR1_MATCH!"
            if f2_match:
                status += " FLOOR2_MATCH!"
            print(f"Offset {offset:2d}: F1={floor1} F2={floor2}{status}")
            found_match = True
        elif offset < 10:  # Always show first 10
            print(f"Offset {offset:2d}: F1={floor1} F2={floor2}")

    if not found_match:
        print("\nNo matches found in offsets 0-49")

    # Also try raw sequence analysis
    print("\n" + "="*60)
    print("RAW RNG SEQUENCE ANALYSIS")
    print("="*60)

    seed = seed_to_long("1ABCD")
    rng = Random(seed)

    # For Floor 1, we need:
    # Card 1: UNCOMMON (roll < 40), index 9 for LikeWater
    # Card 2: COMMON (roll >= 40), index 1 for BowlingBash
    # Card 3: UNCOMMON (roll < 40), index 17 for DeceiveReality

    print("\nLooking for rarity pattern U,C,U followed by index pattern...")
    print(f"Need: UNCOMMON idx={pools['UNCOMMON'].index('LikeWater')}")
    print(f"      COMMON idx={pools['COMMON'].index('BowlingBash')}")
    print(f"      UNCOMMON idx={pools['UNCOMMON'].index('DeceiveReality')}")

    # Generate raw sequence
    raw_seq = []
    rng2 = Random(seed)
    for i in range(100):
        raw_seq.append(rng2.random(99))

    print(f"\nFirst 50 random(99) values:")
    for i in range(50):
        print(f"  {i:2d}: {raw_seq[i]:2d}", end="")
        if (i + 1) % 10 == 0:
            print()


if __name__ == "__main__":
    main()
