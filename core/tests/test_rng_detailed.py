"""
Detailed RNG trace showing EVERY call with its actual bound.

The key insight: random(99) and random(34) both advance the RNG once,
but the OUTPUT depends on the bound due to modulo operation.
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
XorShift128 = rng_module.XorShift128
seed_to_long = rng_module.seed_to_long


def trace_raw_rng():
    """Trace the raw nextLong values to verify XorShift128."""
    seed = seed_to_long("1ABCD")
    print(f"Seed: 1ABCD = {seed}")

    # Create XorShift directly to see raw values
    xor = XorShift128(seed)

    print("\nRaw XorShift128 state after init:")
    print(f"  seed0: {xor.seed0}")
    print(f"  seed1: {xor.seed1}")

    print("\nFirst 10 nextLong() values (raw 64-bit):")
    for i in range(10):
        val = xor._next_long()
        print(f"  {i}: {val} (hex: {hex(val)})")


def trace_with_bounds():
    """Trace RNG calls with actual bounds used in card rewards."""
    seed = seed_to_long("1ABCD")

    # Test 1: Get the sequence of values we expect
    rng = Random(seed)

    print("\n" + "="*60)
    print("CARD REWARD SIMULATION - FLOOR 1")
    print("="*60)

    # Load pools
    _card_lib_module = load_module(
        "card_library_order",
        os.path.join(core_dir, "utils", "card_library_order.py")
    )

    pools = {
        "COMMON": _card_lib_module.get_watcher_pool_by_rarity("COMMON"),
        "UNCOMMON": _card_lib_module.get_watcher_pool_by_rarity("UNCOMMON"),
        "RARE": _card_lib_module.get_watcher_pool_by_rarity("RARE"),
    }

    RARE_THRESHOLD = 3
    UNCOMMON_THRESHOLD = 37
    blizzard = 5

    cards_picked = []

    for card_num in range(1, 4):
        print(f"\n--- Card {card_num} ---")

        # Roll rarity
        raw_roll = rng.random(99)
        adjusted = raw_roll + blizzard
        print(f"  Rarity roll: random(99)={raw_roll}, adjusted={adjusted}")

        if adjusted < RARE_THRESHOLD:
            rarity = "RARE"
            blizzard = 5
        elif adjusted < RARE_THRESHOLD + UNCOMMON_THRESHOLD:
            rarity = "UNCOMMON"
        else:
            rarity = "COMMON"
            blizzard -= 1

        pool = pools[rarity]
        pool_size = len(pool)
        print(f"  Rarity: {rarity} (pool size: {pool_size})")

        # Roll index
        idx = rng.random(pool_size - 1)
        card_id = pool[idx]
        print(f"  Index roll: random({pool_size - 1})={idx}")
        print(f"  Card: {card_id} (position {idx} in {rarity} pool)")

        # Check duplicate (the game rerolls, but we shouldn't hit duplicates in floor 1)
        if card_id in cards_picked:
            print(f"  !!! DUPLICATE - would reroll")

        cards_picked.append(card_id)

    print(f"\n  Floor 1 result: {cards_picked}")
    print(f"  Expected: ['LikeWater', 'BowlingBash', 'DeceiveReality']")

    # Now trace Floor 2
    print("\n" + "="*60)
    print("CARD REWARD SIMULATION - FLOOR 2")
    print("="*60)

    cards_picked = []

    for card_num in range(1, 4):
        print(f"\n--- Card {card_num} ---")

        # Roll rarity
        raw_roll = rng.random(99)
        adjusted = raw_roll + blizzard
        print(f"  Rarity roll: random(99)={raw_roll}, adjusted={adjusted}")

        if adjusted < RARE_THRESHOLD:
            rarity = "RARE"
            blizzard = 5
        elif adjusted < RARE_THRESHOLD + UNCOMMON_THRESHOLD:
            rarity = "UNCOMMON"
        else:
            rarity = "COMMON"
            blizzard -= 1

        pool = pools[rarity]
        pool_size = len(pool)
        print(f"  Rarity: {rarity} (pool size: {pool_size})")

        # Roll index
        idx = rng.random(pool_size - 1)
        card_id = pool[idx]
        print(f"  Index roll: random({pool_size - 1})={idx}")
        print(f"  Card: {card_id} (position {idx} in {rarity} pool)")

        if card_id in cards_picked:
            print(f"  !!! DUPLICATE - would reroll")

        cards_picked.append(card_id)

    print(f"\n  Floor 2 result: {cards_picked}")
    print(f"  Expected: ['SashWhip', 'Evaluate', 'Worship']")


def verify_pool_positions():
    """Verify pool positions for expected cards."""
    _card_lib_module = load_module(
        "card_library_order",
        os.path.join(core_dir, "utils", "card_library_order.py")
    )

    pools = {
        "COMMON": _card_lib_module.get_watcher_pool_by_rarity("COMMON"),
        "UNCOMMON": _card_lib_module.get_watcher_pool_by_rarity("UNCOMMON"),
        "RARE": _card_lib_module.get_watcher_pool_by_rarity("RARE"),
    }

    print("\n" + "="*60)
    print("POOL POSITIONS")
    print("="*60)

    expected_floor1 = [("LikeWater", "UNCOMMON"), ("BowlingBash", "COMMON"), ("DeceiveReality", "UNCOMMON")]
    expected_floor2 = [("SashWhip", "COMMON"), ("Evaluate", "COMMON"), ("Worship", "UNCOMMON")]

    print("\nFloor 1 expected:")
    for card_id, rarity in expected_floor1:
        pool = pools[rarity]
        if card_id in pool:
            pos = pool.index(card_id)
            print(f"  {card_id}: position {pos} in {rarity} (pool size {len(pool)})")
        else:
            print(f"  {card_id}: NOT FOUND in {rarity}!")

    print("\nFloor 2 expected:")
    for card_id, rarity in expected_floor2:
        pool = pools[rarity]
        if card_id in pool:
            pos = pool.index(card_id)
            print(f"  {card_id}: position {pos} in {rarity} (pool size {len(pool)})")
        else:
            print(f"  {card_id}: NOT FOUND in {rarity}!")


def reverse_engineer():
    """
    Try to reverse-engineer what RNG values would produce the expected output.
    """
    _card_lib_module = load_module(
        "card_library_order",
        os.path.join(core_dir, "utils", "card_library_order.py")
    )

    pools = {
        "COMMON": _card_lib_module.get_watcher_pool_by_rarity("COMMON"),
        "UNCOMMON": _card_lib_module.get_watcher_pool_by_rarity("UNCOMMON"),
        "RARE": _card_lib_module.get_watcher_pool_by_rarity("RARE"),
    }

    print("\n" + "="*60)
    print("REVERSE ENGINEERING EXPECTED OUTPUT")
    print("="*60)

    expected = [
        # Floor 1
        ("LikeWater", "UNCOMMON", 5),  # blizzard 5
        ("BowlingBash", "COMMON", 5),  # blizzard still 5 (uncommon doesn't change it)
        ("DeceiveReality", "UNCOMMON", 4),  # blizzard 4 (common reduced it)
        # Floor 2
        ("SashWhip", "COMMON", 4),  # blizzard 4
        ("Evaluate", "COMMON", 3),  # blizzard 3
        ("Worship", "UNCOMMON", 2),  # blizzard 2
    ]

    RARE_THRESHOLD = 3
    UNCOMMON_THRESHOLD = 37

    print("\nFor each card, what rarity roll range would produce it?")
    print("(adjusted_roll must be in range to get the rarity)")

    for card_id, rarity, blizzard in expected:
        pool = pools[rarity]
        if card_id not in pool:
            print(f"  {card_id}: NOT IN POOL!")
            continue

        idx = pool.index(card_id)
        pool_size = len(pool)

        # What rarity roll range?
        if rarity == "RARE":
            raw_min = 0 - blizzard
            raw_max = RARE_THRESHOLD - 1 - blizzard
        elif rarity == "UNCOMMON":
            raw_min = RARE_THRESHOLD - blizzard
            raw_max = RARE_THRESHOLD + UNCOMMON_THRESHOLD - 1 - blizzard
        else:  # COMMON
            raw_min = RARE_THRESHOLD + UNCOMMON_THRESHOLD - blizzard
            raw_max = 99

        raw_min = max(0, raw_min)
        raw_max = min(99, raw_max)

        print(f"\n  {card_id} ({rarity}):")
        print(f"    Rarity roll must be in [{raw_min}, {raw_max}] (blizzard={blizzard})")
        print(f"    Index roll must be {idx} (pool size={pool_size}, so random({pool_size-1})={idx})")


if __name__ == "__main__":
    trace_raw_rng()
    verify_pool_positions()
    trace_with_bounds()
    reverse_engineer()
