#!/usr/bin/env python3
"""
Debug what cards are generated at counter 39 for floor 8.
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.state.rng import Random, seed_to_long
from core.state.game_rng import GameRNGState, RNGStream
from core.generation.rewards import generate_card_rewards, RewardState, CARD_RARITY_THRESHOLDS

SEED = "33J85JVCVSPJY"


def main():
    print(f"Debugging floor 8 card reward at counter 39")
    print(f"Seed: {SEED}")
    print("=" * 60)

    # Get seed value
    seed = seed_to_long(SEED)
    print(f"Seed long: {seed}")
    print()

    # Create RNG at counter 39
    rng = Random(seed, 39)

    print("Manual card generation trace:")
    print("-" * 40)

    # Card 1
    rarity_roll_1 = rng.random(99)
    print(f"Card 1 rarity roll: {rarity_roll_1}")
    print(f"  -> Thresholds (normal): rare<3, uncommon<37")
    if rarity_roll_1 < 3:
        rarity_1 = "RARE"
    elif rarity_roll_1 < 37:
        rarity_1 = "UNCOMMON"
    else:
        rarity_1 = "COMMON"
    print(f"  -> Rarity: {rarity_1}")

    # Get pool size for rarity
    pool_sizes = {"COMMON": 19, "UNCOMMON": 35, "RARE": 17}
    pool_1_size = pool_sizes[rarity_1]
    idx_1 = rng.random(pool_1_size - 1)
    print(f"Card 1 index roll: {idx_1} (pool size {pool_1_size})")
    print()

    # Card 2
    rarity_roll_2 = rng.random(99)
    print(f"Card 2 rarity roll: {rarity_roll_2}")
    if rarity_roll_2 < 3:
        rarity_2 = "RARE"
    elif rarity_roll_2 < 37:
        rarity_2 = "UNCOMMON"
    else:
        rarity_2 = "COMMON"
    print(f"  -> Rarity: {rarity_2}")
    pool_2_size = pool_sizes[rarity_2]
    idx_2 = rng.random(pool_2_size - 1)
    print(f"Card 2 index roll: {idx_2} (pool size {pool_2_size})")
    print()

    # Card 3
    rarity_roll_3 = rng.random(99)
    print(f"Card 3 rarity roll: {rarity_roll_3}")
    if rarity_roll_3 < 3:
        rarity_3 = "RARE"
    elif rarity_roll_3 < 37:
        rarity_3 = "UNCOMMON"
    else:
        rarity_3 = "COMMON"
    print(f"  -> Rarity: {rarity_3}")
    pool_3_size = pool_sizes[rarity_3]
    idx_3 = rng.random(pool_3_size - 1)
    print(f"Card 3 index roll: {idx_3} (pool size {pool_3_size})")
    print()

    # Upgrade checks (Act 1 = 0% chance, but still consumes RNG for non-rare)
    print("Upgrade checks:")
    for i, rarity in enumerate([rarity_1, rarity_2, rarity_3], 1):
        if rarity != "RARE":
            upgrade_roll = rng.random_boolean(0.0)
            print(f"  Card {i} ({rarity}): consumed RNG, upgrade={upgrade_roll}")
        else:
            print(f"  Card {i} (RARE): skipped RNG call")
    print()

    print(f"Final counter: {rng.counter}")
    print()

    # Now use actual function
    print("=" * 60)
    print("Using generate_card_rewards():")
    print("-" * 40)

    state = GameRNGState(SEED)
    state.set_counter(RNGStream.CARD, 39)
    card_rng = state.get_rng(RNGStream.CARD)

    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=RewardState(),
        act=1,
        player_class="WATCHER",
        ascension=20,
        room_type="normal",
        num_cards=3
    )

    print(f"Generated cards: {[c.name for c in cards]}")
    print(f"Counter after: {card_rng.counter}")

    # Print card pool at relevant indices
    print()
    print("=" * 60)
    print("Card pools at the rolled indices:")
    print("-" * 40)

    from core.utils.card_library_order import get_watcher_pool_by_rarity

    common_pool = get_watcher_pool_by_rarity("COMMON")
    uncommon_pool = get_watcher_pool_by_rarity("UNCOMMON")
    rare_pool = get_watcher_pool_by_rarity("RARE")

    pools = {
        "COMMON": common_pool,
        "UNCOMMON": uncommon_pool,
        "RARE": rare_pool
    }

    print(f"\nCard 1: rarity={rarity_1}, index={idx_1}")
    if idx_1 < len(pools[rarity_1]):
        print(f"  -> {pools[rarity_1][idx_1]}")

    print(f"\nCard 2: rarity={rarity_2}, index={idx_2}")
    if idx_2 < len(pools[rarity_2]):
        print(f"  -> {pools[rarity_2][idx_2]}")

    print(f"\nCard 3: rarity={rarity_3}, index={idx_3}")
    if idx_3 < len(pools[rarity_3]):
        print(f"  -> {pools[rarity_3][idx_3]}")

    # Check what ReachHeaven's index is
    print()
    print("=" * 60)
    print("Checking ReachHeaven position:")
    try:
        reach_idx = uncommon_pool.index("ReachHeaven")
        print(f"ReachHeaven is at UNCOMMON index {reach_idx}")
    except ValueError:
        print("ReachHeaven not found in UNCOMMON pool")


if __name__ == "__main__":
    main()
