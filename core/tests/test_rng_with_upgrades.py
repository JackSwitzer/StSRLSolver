"""
RNG trace accounting for upgrade check RNG calls.

The game calls cardRng.randomBoolean(cardUpgradedChance) for EVERY
non-rare card AFTER selection, even if upgrade chance is 0 in Act 1!
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


def load_pools():
    _card_lib_module = load_module(
        "card_library_order",
        os.path.join(core_dir, "utils", "card_library_order.py")
    )
    return {
        "COMMON": _card_lib_module.get_watcher_pool_by_rarity("COMMON"),
        "UNCOMMON": _card_lib_module.get_watcher_pool_by_rarity("UNCOMMON"),
        "RARE": _card_lib_module.get_watcher_pool_by_rarity("RARE"),
    }


def simulate_card_reward_with_upgrades(rng, pools, blizzard, act=1, ascension=0):
    """
    Simulate card reward with upgrade checks (matches game exactly).

    Algorithm:
    1. Roll rarity for each card
    2. Roll index for each card (reroll on duplicate)
    3. AFTER all cards selected, roll upgrade check for non-rare cards
    """
    RARE_THRESHOLD = 3
    UNCOMMON_THRESHOLD = 37

    # Upgrade chances by act
    if act == 1:
        upgrade_chance = 0.0
    elif act == 2:
        upgrade_chance = 0.125 if ascension >= 12 else 0.25
    elif act >= 3:
        upgrade_chance = 0.25 if ascension >= 12 else 0.50
    else:
        upgrade_chance = 0.0

    cards = []
    cards_picked = set()

    # Phase 1: Select all cards
    for i in range(3):
        # Roll rarity
        roll = rng.random(99)
        adjusted = roll + blizzard

        if adjusted < RARE_THRESHOLD:
            rarity = "RARE"
            blizzard = 5  # Reset blizzard
        elif adjusted < RARE_THRESHOLD + UNCOMMON_THRESHOLD:
            rarity = "UNCOMMON"
            # Blizzard unchanged for uncommon
        else:
            rarity = "COMMON"
            blizzard = max(blizzard - 1, -40)  # Decrease, min -40

        pool = pools[rarity]

        # Roll index (reroll on duplicate)
        while True:
            idx = rng.random(len(pool) - 1)
            card_id = pool[idx]
            if card_id not in cards_picked:
                break

        cards_picked.add(card_id)
        cards.append((card_id, rarity))

    # Phase 2: Upgrade checks for non-rare cards
    upgrades = []
    for card_id, rarity in cards:
        if rarity != "RARE":
            # randomBoolean consumes RNG even if upgrade_chance is 0
            upgrade_roll = rng.random_boolean(upgrade_chance)
            upgrades.append(upgrade_roll)
        else:
            upgrades.append(None)  # No roll for rare

    return cards, blizzard, upgrades


def main():
    seed = seed_to_long("1ABCD")
    pools = load_pools()

    print(f"Seed: 1ABCD = {seed}")
    print(f"Pool sizes: COMMON={len(pools['COMMON'])}, UNCOMMON={len(pools['UNCOMMON'])}, RARE={len(pools['RARE'])}")

    rng = Random(seed)
    blizzard = 5  # Starting blizzard offset

    print("\n" + "="*60)
    print("FLOOR 1 CARD REWARD (with upgrade checks)")
    print("="*60)

    floor1_cards, blizzard, floor1_upgrades = simulate_card_reward_with_upgrades(
        rng, pools, blizzard, act=1
    )

    print(f"\nCards selected:")
    for i, (card_id, rarity) in enumerate(floor1_cards):
        upgrade_str = f" (upgrade roll: {floor1_upgrades[i]})" if floor1_upgrades[i] is not None else " (no upgrade roll - RARE)"
        print(f"  {i+1}. {card_id} ({rarity}){upgrade_str}")

    print(f"\nBlizzard after Floor 1: {blizzard}")
    print(f"RNG counter after Floor 1: {rng.counter}")

    print("\n" + "="*60)
    print("FLOOR 2 CARD REWARD (with upgrade checks)")
    print("="*60)

    floor2_cards, blizzard, floor2_upgrades = simulate_card_reward_with_upgrades(
        rng, pools, blizzard, act=1
    )

    print(f"\nCards selected:")
    for i, (card_id, rarity) in enumerate(floor2_cards):
        upgrade_str = f" (upgrade roll: {floor2_upgrades[i]})" if floor2_upgrades[i] is not None else " (no upgrade roll - RARE)"
        print(f"  {i+1}. {card_id} ({rarity}){upgrade_str}")

    print(f"\nBlizzard after Floor 2: {blizzard}")
    print(f"RNG counter after Floor 2: {rng.counter}")

    print("\n" + "="*60)
    print("EXPECTED")
    print("="*60)
    print("Floor 1: LikeWater (U), BowlingBash (C), DeceiveReality (U)")
    print("Floor 2: SashWhip (C), Evaluate (C), Worship (U)")


if __name__ == "__main__":
    main()
