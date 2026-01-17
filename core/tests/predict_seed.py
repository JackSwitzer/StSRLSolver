"""
Full seed prediction including Neow, encounters, cards, and events.
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
rng_mod = load_module("rng", os.path.join(core_dir, "state", "rng.py"))
rewards_mod = load_module("rewards", os.path.join(core_dir, "generation", "rewards.py"))
encounters_mod = load_module("encounters", os.path.join(core_dir, "generation", "encounters.py"))


# Neow reward categories
NEOW_CATEGORY_0 = [
    "THREE_CARDS",           # Choose 1 of 3 cards
    "ONE_RANDOM_RARE_CARD",  # Obtain random rare
    "REMOVE_CARD",           # Remove a card
    "UPGRADE_CARD",          # Upgrade a card
    "TRANSFORM_CARD",        # Transform a card
    "RANDOM_COLORLESS",      # Choose 1 of 3 colorless
]

NEOW_CATEGORY_1 = [
    "THREE_SMALL_POTIONS",   # 3 random potions
    "RANDOM_COMMON_RELIC",   # Random common relic
    "TEN_PERCENT_HP_BONUS",  # +10% max HP
    "THREE_ENEMY_KILL",      # Neow's Lament (first 3 enemies have 1 HP)
    "HUNDRED_GOLD",          # +100 gold
]

NEOW_DRAWBACKS = [
    "TEN_PERCENT_HP_LOSS",   # -10% max HP
    "NO_GOLD",               # Lose all gold
    "CURSE",                 # Obtain a curse
    "PERCENT_DAMAGE",        # Take 30% current HP damage
]

NEOW_CATEGORY_2 = [
    "RANDOM_COLORLESS_2",    # Choose 1 of 3 rare colorless
    "REMOVE_TWO",            # Remove 2 cards (not with CURSE)
    "ONE_RARE_RELIC",        # Random rare relic
    "THREE_RARE_CARDS",      # Choose 1 of 3 rare cards
    "TWO_FIFTY_GOLD",        # +250 gold (not with NO_GOLD)
    "TRANSFORM_TWO_CARDS",   # Transform 2 cards
    "TWENTY_PERCENT_HP_BONUS",  # +20% max HP (not with HP_LOSS)
]

# Act 1 events (in initialization order)
ACT1_EVENTS = [
    "Big Fish",
    "The Cleric",
    "Dead Adventurer",  # floor > 6 only
    "Golden Idol",
    "Golden Wing",
    "World of Goop",
    "Liars Game",
    "Living Wall",
    "Mushrooms",  # floor > 6 only
    "Scrap Ooze",
    "Shining Light",
]

ACT1_SHRINES = [
    "Match and Keep!",
    "Golden Shrine",
    "Transmorgrifier",
    "Purifier",
    "Upgrade Shrine",
    "Wheel of Change",
]


def predict_neow(seed):
    """Predict Neow's 4 blessing options."""
    rng = rng_mod.Random(seed)

    # Category 0: Basic reward
    cat0_idx = rng.random_int_range(0, len(NEOW_CATEGORY_0) - 1)
    cat0 = NEOW_CATEGORY_0[cat0_idx]

    # Category 1: Small reward
    cat1_idx = rng.random_int_range(0, len(NEOW_CATEGORY_1) - 1)
    cat1 = NEOW_CATEGORY_1[cat1_idx]

    # Category 2: First pick drawback
    drawback_idx = rng.random_int_range(0, len(NEOW_DRAWBACKS) - 1)
    drawback = NEOW_DRAWBACKS[drawback_idx]

    # Then pick reward (some excluded based on drawback)
    cat2_options = NEOW_CATEGORY_2.copy()
    if drawback == "CURSE":
        cat2_options.remove("REMOVE_TWO")
    if drawback == "NO_GOLD":
        cat2_options.remove("TWO_FIFTY_GOLD")
    if drawback == "TEN_PERCENT_HP_LOSS":
        cat2_options.remove("TWENTY_PERCENT_HP_BONUS")

    cat2_idx = rng.random_int_range(0, len(cat2_options) - 1)
    cat2 = cat2_options[cat2_idx]

    # Category 3: Always boss swap
    cat3 = "BOSS_RELIC"

    return {
        "option1": cat0,
        "option2": cat1,
        "option3_drawback": drawback,
        "option3_reward": cat2,
        "option4": cat3,
        "rng_counter": rng.counter,
    }


def predict_event_room(seed, floor, event_rng_counter=0, shrine_chance=0.25):
    """
    Predict what happens at a ? room.

    Returns:
        room_type: "EVENT", "MONSTER", "SHOP", "TREASURE"
        event_name: If room_type is EVENT, which event
    """
    rng = rng_mod.Random(seed)
    # Advance to current counter
    for _ in range(event_rng_counter):
        rng.random_float()

    # Initial probabilities (start of act)
    # These change after each ? room, but for first ? room:
    elite_chance = 0.10  # but 0 if floor < 6
    monster_chance = 0.10
    shop_chance = 0.03
    treasure_chance = 0.02

    if floor < 6:
        elite_chance = 0

    # Roll for room type
    roll = rng.random_float()

    # Build probability array
    # Order: ELITE (if applicable), MONSTER, SHOP, TREASURE, rest is EVENT
    cumulative = 0
    room_type = "EVENT"

    cumulative += elite_chance
    if roll < cumulative:
        room_type = "ELITE"
    else:
        cumulative += monster_chance
        if roll < cumulative:
            room_type = "MONSTER"
        else:
            cumulative += shop_chance
            if roll < cumulative:
                room_type = "SHOP"
            else:
                cumulative += treasure_chance
                if roll < cumulative:
                    room_type = "TREASURE"

    event_name = None
    if room_type == "EVENT":
        # Roll for shrine vs event
        shrine_roll = rng.random_float()
        if shrine_roll < shrine_chance:
            # Shrine
            shrine_idx = rng.random_int_range(0, len(ACT1_SHRINES) - 1)
            event_name = ACT1_SHRINES[shrine_idx]
        else:
            # Regular event (filter by floor)
            available_events = [e for e in ACT1_EVENTS
                               if e not in ["Dead Adventurer", "Mushrooms"] or floor > 6]
            event_idx = rng.random_int_range(0, len(available_events) - 1)
            event_name = available_events[event_idx]

    return {
        "roll": roll,
        "room_type": room_type,
        "event_name": event_name,
        "rng_counter": rng.counter,
    }


def full_prediction(seed_str):
    """Generate full prediction for a seed."""
    seed = rng_mod.seed_to_long(seed_str)

    print(f"{'='*60}")
    print(f"FULL PREDICTION FOR SEED: {seed_str}")
    print(f"Seed value: {seed}")
    print(f"{'='*60}")

    # Neow
    print("\n=== NEOW BLESSINGS ===")
    neow = predict_neow(seed)
    print(f"  Option 1: {neow['option1']}")
    print(f"  Option 2: {neow['option2']}")
    print(f"  Option 3: {neow['option3_drawback']} -> {neow['option3_reward']}")
    print(f"  Option 4: {neow['option4']} (Boss Swap)")

    # Encounters
    print("\n=== ENCOUNTERS ===")
    monster_rng = rng_mod.Random(seed)
    normal, elite = encounters_mod.generate_exordium_encounters(monster_rng)

    for floor in range(1, 6):
        enemy = normal[floor - 1]
        hp_rng = rng_mod.Random(seed + floor)
        hp = encounters_mod.get_enemy_hp(enemy, hp_rng)
        hp_str = f"{hp} HP" if hp > 0 else "multi-enemy"
        print(f"  Floor {floor}: {enemy} ({hp_str})")

    # Card Rewards
    print("\n=== CARD REWARDS ===")
    card_rng = rng_mod.Random(seed)
    state = rewards_mod.RewardState()

    for floor in range(1, 4):
        cards = rewards_mod.generate_card_rewards(
            card_rng, state, act=1, player_class="WATCHER"
        )
        card_str = ", ".join([f"{c.name} ({c.rarity.name[0]})" for c in cards])
        print(f"  Floor {floor}: {card_str}")

    # First ? room prediction (assumes it's on floor 2 or 3)
    print("\n=== FIRST ? ROOM (hypothetical floor 2-3) ===")
    event_result = predict_event_room(seed, floor=3, event_rng_counter=0)
    print(f"  Roll: {event_result['roll']:.4f}")
    print(f"  Room type: {event_result['room_type']}")
    if event_result['event_name']:
        print(f"  Event: {event_result['event_name']}")


if __name__ == "__main__":
    import sys
    seed = sys.argv[1] if len(sys.argv) > 1 else "TEST123"
    full_prediction(seed)
