"""
Comprehensive RNG Trace - Debug card reward discrepancy

Seed: 1ABCD
Floor 1 Combat: Jaw Worm (40 HP)
Floor 1 Reward: Like Water (U), Bowling Bash (C), Deceive Reality (U) ✓

Floor 2 Combat: Cultist (51 HP)
Floor 2 Reward ACTUAL: Sash Whip (C), Evaluate (C), Worship (U)
Floor 2 Reward PREDICTED: Fear No Evil (U), Empty Fist (C), Just Lucky (C) ✗

The rarities don't even match! Let's trace every RNG call.
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
rewards_module = load_module("rewards", os.path.join(core_dir, "generation", "rewards.py"))

Random = rng_module.Random
XorShift128 = rng_module.XorShift128
seed_to_long = rng_module.seed_to_long
CardRarity = rewards_module.CardRarity


class TracingRandom(Random):
    """Random that logs every call for debugging."""

    def __init__(self, seed, name="unnamed"):
        super().__init__(seed)
        self.name = name
        self.call_log = []

    def random_int(self, range_val):
        result = super().random_int(range_val)
        self.call_log.append({
            'method': 'random_int',
            'arg': range_val,
            'result': result,
            'counter': self.counter
        })
        print(f"  [{self.name}] random_int({range_val}) = {result} (counter={self.counter})")
        return result

    def random(self, range_val):
        """Alias that also logs."""
        return self.random_int(range_val)

    def random_boolean(self, chance=None):
        result = super().random_boolean(chance)
        self.call_log.append({
            'method': 'random_boolean',
            'arg': chance,
            'result': result,
            'counter': self.counter
        })
        print(f"  [{self.name}] random_boolean({chance}) = {result} (counter={self.counter})")
        return result

    def random_float(self):
        result = super().random_float()
        self.call_log.append({
            'method': 'random_float',
            'arg': None,
            'result': result,
            'counter': self.counter
        })
        print(f"  [{self.name}] random_float() = {result:.6f} (counter={self.counter})")
        return result


def trace_card_rewards():
    """Trace through card reward generation step by step."""
    seed_str = "1ABCD"
    seed = seed_to_long(seed_str)
    print(f"=== RNG Trace for Seed: {seed_str} ({seed}) ===\n")

    # Create tracing RNG
    card_rng = TracingRandom(seed, "cardRng")

    # Get card pools
    _card_lib_module = load_module(
        "card_library_order",
        os.path.join(core_dir, "utils", "card_library_order.py")
    )

    common_pool = _card_lib_module.get_watcher_pool_by_rarity("COMMON")
    uncommon_pool = _card_lib_module.get_watcher_pool_by_rarity("UNCOMMON")
    rare_pool = _card_lib_module.get_watcher_pool_by_rarity("RARE")

    print(f"Pool sizes: COMMON={len(common_pool)}, UNCOMMON={len(uncommon_pool)}, RARE={len(rare_pool)}")
    print(f"\nCOMMON pool order: {common_pool}")
    print(f"\nUNCOMMON pool order: {uncommon_pool}")
    print(f"\nRARE pool order: {rare_pool}")

    # Card blizzard state
    card_blizzard_offset = 5  # Starting offset

    # Rarity thresholds (normal room)
    RARE_THRESHOLD = 3
    UNCOMMON_THRESHOLD = 37

    print("\n" + "="*60)
    print("FLOOR 1 CARD REWARD")
    print("="*60)

    cards_picked = set()
    for card_num in range(3):
        print(f"\n--- Card {card_num + 1} ---")

        # Roll rarity
        print(f"Rolling rarity (blizzard offset = {card_blizzard_offset}):")
        roll = card_rng.random(99)
        adjusted_roll = roll + card_blizzard_offset
        print(f"  Raw roll: {roll}, Adjusted: {adjusted_roll}")

        if adjusted_roll < RARE_THRESHOLD:
            rarity = "RARE"
            card_blizzard_offset = 5  # Reset on rare
            pool = rare_pool
        elif adjusted_roll < RARE_THRESHOLD + UNCOMMON_THRESHOLD:
            rarity = "UNCOMMON"
            pool = uncommon_pool
        else:
            rarity = "COMMON"
            card_blizzard_offset -= 1  # Decrease on common
            pool = common_pool

        print(f"  Rarity: {rarity}")

        # Pick card from pool
        available_pool = [c for c in pool if c not in cards_picked]
        print(f"  Available pool size: {len(available_pool)}")

        idx = card_rng.random(len(available_pool) - 1)
        card_id = available_pool[idx]
        cards_picked.add(card_id)
        print(f"  Selected index {idx}: {card_id}")

    print("\n" + "="*60)
    print("FLOOR 2 CARD REWARD")
    print("="*60)

    # Reset cards picked for new reward
    cards_picked = set()

    for card_num in range(3):
        print(f"\n--- Card {card_num + 1} ---")

        # Roll rarity
        print(f"Rolling rarity (blizzard offset = {card_blizzard_offset}):")
        roll = card_rng.random(99)
        adjusted_roll = roll + card_blizzard_offset
        print(f"  Raw roll: {roll}, Adjusted: {adjusted_roll}")

        if adjusted_roll < RARE_THRESHOLD:
            rarity = "RARE"
            card_blizzard_offset = 5  # Reset on rare
            pool = rare_pool
        elif adjusted_roll < RARE_THRESHOLD + UNCOMMON_THRESHOLD:
            rarity = "UNCOMMON"
            pool = uncommon_pool
        else:
            rarity = "COMMON"
            card_blizzard_offset -= 1  # Decrease on common
            pool = common_pool

        print(f"  Rarity: {rarity}")

        # Pick card from pool
        available_pool = [c for c in pool if c not in cards_picked]
        print(f"  Available pool size: {len(available_pool)}")

        idx = card_rng.random(len(available_pool) - 1)
        card_id = available_pool[idx]
        cards_picked.add(card_id)
        print(f"  Selected index {idx}: {card_id}")

    print("\n" + "="*60)
    print("EXPECTED vs ACTUAL")
    print("="*60)
    print("\nFloor 2 ACTUAL from game: Sash Whip (C), Evaluate (C), Worship (U)")
    print("Check: Is 'SashWhip' in COMMON pool?", "SashWhip" in common_pool)
    print("Check: Is 'Evaluate' in COMMON pool?", "Evaluate" in common_pool)
    print("Check: Is 'Worship' in UNCOMMON pool?", "Worship" in uncommon_pool)

    # Show positions
    if "SashWhip" in common_pool:
        print(f"  SashWhip position in COMMON: {common_pool.index('SashWhip')}")
    if "Evaluate" in common_pool:
        print(f"  Evaluate position in COMMON: {common_pool.index('Evaluate')}")
    if "Worship" in uncommon_pool:
        print(f"  Worship position in UNCOMMON: {uncommon_pool.index('Worship')}")


def test_raw_rng_sequence():
    """Test the raw RNG sequence to verify XorShift128."""
    seed_str = "1ABCD"
    seed = seed_to_long(seed_str)

    print("\n" + "="*60)
    print(f"RAW RNG SEQUENCE FOR SEED {seed_str} ({seed})")
    print("="*60)

    rng = Random(seed)

    print("\nFirst 20 random(99) calls:")
    for i in range(20):
        val = rng.random(99)
        print(f"  Call {i+1}: {val}")


if __name__ == "__main__":
    trace_card_rewards()
    test_raw_rng_sequence()
