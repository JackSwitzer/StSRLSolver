#!/usr/bin/env python3
"""
Generate Full Act 1 Predictions for Testing

Generates comprehensive predictions for seed validation:
- Map layout with all room types
- Enemy encounters with HP and first moves
- Card rewards for each combat
- Gold rewards
- Event predictions
- Potion drops
- Elite/boss relic rewards

Usage:
    python -m core.tests.generate_full_act1_seeds [SEED1] [SEED2] [SEED3]
"""

import os
import sys
from typing import Dict, List, Optional, Tuple, Any

# Setup path
_script_dir = os.path.dirname(os.path.abspath(__file__))
_core_dir = os.path.dirname(_script_dir)
_project_dir = os.path.dirname(_core_dir)
sys.path.insert(0, _project_dir)

from core.state.rng import Random, seed_to_long
from core.state.game_rng import GameRNGState, RNGStream
from core.generation.map import (
    MapGenerator, MapGeneratorConfig, RoomType,
    get_map_seed_offset, map_to_string
)
from core.generation.encounters import (
    generate_exordium_encounters, get_enemy_hp, ENEMY_HP_RANGES
)
from core.generation.rewards import (
    RewardState, generate_card_rewards, generate_gold_reward,
    check_potion_drop, generate_elite_relic_reward, generate_boss_relics
)


# =============================================================================
# ENEMY FIRST MOVE PATTERNS (from decompiled AI)
# =============================================================================

ENEMY_FIRST_MOVES = {
    # Act 1 Weak
    "Cultist": {
        "move": "Incantation",
        "intent": "BUFF",
        "details": "Gains 3 Ritual (gains +3 Strength each turn)"
    },
    "Jaw Worm": {
        "move": "Chomp",
        "intent": "ATTACK",
        "damage": 11,
        "details": "11 damage attack (12 at A2+)"
    },
    "2 Louse": {
        "move_pattern": "Each louse: 50% Bite (5-7 dmg), 50% Grow (+3 Str)",
        "details": "Red Louse: Bite or Grow. Green Louse: Bite or Spit Web (Weak 2)"
    },
    "Small Slimes": {
        "move_pattern": "Acid Slime S: Lick (Weak 1) or Tackle (3 dmg). Spike Slime S: Tackle (5 dmg)",
        "details": "Small slimes have simple attack patterns"
    },

    # Act 1 Strong
    "Blue Slaver": {
        "move": "Stab",
        "intent": "ATTACK",
        "damage": 12,
        "details": "Always opens with 12 damage attack (13 at A2+)"
    },
    "Red Slaver": {
        "move": "Stab",
        "intent": "ATTACK",
        "damage": 13,
        "details": "Always opens with 13 damage attack (14 at A2+)"
    },
    "Looter": {
        "move": "Mug",
        "intent": "ATTACK",
        "damage": 10,
        "details": "Steals 15 gold (can steal more at low HP)"
    },
    "Gremlin Gang": {
        "move_pattern": "Random mix: Fat Gremlin (Smash 4), Mad Gremlin (Scratch 4), Sneaky Gremlin (Puncture 9), Shield Gremlin (Protect/Shield Bash), Wizard Gremlin (Charging)",
        "details": "3-4 gremlins with varied intents"
    },
    "Large Slime": {
        "move_pattern": "Large slime splits when HP < 50%. Spike Slime L: Flame Tackle (16+Frail) or Lick (Frail 2). Acid Slime L: Corrosive Spit (11+Slimed) or Tackle (16+Weak) or Lick (Weak 2)",
        "details": "Splits into 2 medium slimes at half HP"
    },
    "Lots of Slimes": {
        "move_pattern": "5 Spike Slime S: Tackle (5 dmg each)",
        "details": "5 small spike slimes"
    },
    "Exordium Thugs": {
        "move_pattern": "Pointy (Blue Slaver) + Looter combo",
        "details": "Blue Slaver opens with Stab 12, Looter opens with Mug 10"
    },
    "Exordium Wildlife": {
        "move_pattern": "Jaw Worm + 2 Louse OR Jaw Worm + Fungi Beast",
        "details": "Mixed wildlife encounter"
    },
    "3 Louse": {
        "move_pattern": "Each louse: 50% Bite (5-7 dmg), 50% Grow/Spit Web",
        "details": "Same as 2 Louse but with 3"
    },
    "2 Fungi Beasts": {
        "move_pattern": "Fungi Beast: Grow (gain Strength) 60% first turn",
        "details": "Usually both buff first turn"
    },

    # Act 1 Elites
    "Gremlin Nob": {
        "move": "Bellow",
        "intent": "BUFF",
        "details": "Gains 2 Enrage (gains 2 Strength when you play a Skill). 14 dmg basic attack."
    },
    "Lagavulin": {
        "move": "Sleep",
        "intent": "SLEEP",
        "details": "Starts asleep with 8 Metallicize. Wakes after 3 turns or if attacked for 10+ damage."
    },
    "3 Sentries": {
        "move_pattern": "Alternates: Bolt (9 dmg) and Beam (adds Dazed). One sentry beams while two bolt on turn 1.",
        "details": "9 damage each, inflicts Dazed (unplayable status card)"
    },

    # Act 1 Boss
    "Slime Boss": {
        "move": "Goop Spray",
        "intent": "DEBUFF",
        "details": "Adds 3 Slimed to your deck. Splits at 50% HP into 2 large slimes."
    },
    "The Guardian": {
        "move": "Charging Up",
        "intent": "UNKNOWN",
        "details": "Enters defensive mode (9 Block), attacks with Twin Slam (32 dmg total) when mode ends"
    },
    "Hexaghost": {
        "move": "Activate",
        "intent": "UNKNOWN",
        "details": "Does nothing turn 1. Turn 2: Divider (6 x (Player HP / 12 + 1) damage)"
    },
}

# Boss names by Act 1
ACT1_BOSSES = ["Slime Boss", "The Guardian", "Hexaghost"]


# =============================================================================
# DETAILED HP RANGES FOR MULTI-ENEMY ENCOUNTERS
# =============================================================================

MULTI_ENEMY_HP = {
    "2 Louse": [
        {"name": "Louse", "hp_range": (10, 15), "type": "random (red or green)"},
        {"name": "Louse", "hp_range": (10, 15), "type": "random (red or green)"},
    ],
    "Small Slimes": [
        {"name": "Acid Slime S", "hp_range": (8, 12)},
        {"name": "Spike Slime S/M", "hp_range": (10, 14), "note": "S or M variant"},
    ],
    "Gremlin Gang": [
        {"name": "Gremlin", "hp_range": (10, 17), "type": "random 4 of 5 types"},
        {"name": "Gremlin", "hp_range": (10, 17), "type": "random"},
        {"name": "Gremlin", "hp_range": (10, 17), "type": "random"},
        {"name": "Gremlin", "hp_range": (10, 17), "type": "random"},
    ],
    "3 Sentries": [
        {"name": "Sentry", "hp_range": (38, 42)},
        {"name": "Sentry", "hp_range": (38, 42)},
        {"name": "Sentry", "hp_range": (38, 42)},
    ],
    "3 Louse": [
        {"name": "Louse", "hp_range": (10, 15)},
        {"name": "Louse", "hp_range": (10, 15)},
        {"name": "Louse", "hp_range": (10, 15)},
    ],
    "2 Fungi Beasts": [
        {"name": "Fungi Beast", "hp_range": (22, 28)},
        {"name": "Fungi Beast", "hp_range": (22, 28)},
    ],
    "Lots of Slimes": [
        {"name": "Spike Slime S", "hp_range": (10, 14)},
        {"name": "Spike Slime S", "hp_range": (10, 14)},
        {"name": "Spike Slime S", "hp_range": (10, 14)},
        {"name": "Spike Slime S", "hp_range": (10, 14)},
        {"name": "Spike Slime S", "hp_range": (10, 14)},
    ],
    "Large Slime": [
        {"name": "Spike/Acid Slime L", "hp_range": (64, 70)},
    ],
    "Exordium Thugs": [
        {"name": "Blue Slaver (Pointy)", "hp_range": (46, 50)},
        {"name": "Looter", "hp_range": (44, 48)},
    ],
    "Exordium Wildlife": [
        {"name": "Jaw Worm", "hp_range": (40, 44)},
        {"name": "Louse/Fungi Beast", "hp_range": (10, 28), "note": "varies by variant"},
    ],
}


# =============================================================================
# NEOW OPTIONS
# =============================================================================

def predict_neow_options(seed: int) -> Dict[str, Any]:
    """Predict Neow options for a seed."""
    from core.state.rng import Random

    # Neow uses its own RNG seeded with base seed
    neow_rng = Random(seed)

    # The exact Neow option generation is complex and involves:
    # 1. Checking if player died to specific boss
    # 2. Rolling for option categories
    # For now, return placeholder structure
    return {
        "note": "Neow options depend on prior run death. Use game to verify.",
        "expected_categories": ["Blessing", "Drawback+Bonus", "Boss Swap available"]
    }


# =============================================================================
# FULL ACT 1 PREDICTION
# =============================================================================

def predict_full_act1(seed_str: str, ascension: int = 0, neow_option: str = "HUNDRED_GOLD") -> Dict[str, Any]:
    """
    Generate comprehensive Act 1 predictions for a seed.

    Args:
        seed_str: Seed string
        ascension: Ascension level (0-20)
        neow_option: Which Neow option was selected

    Returns:
        Complete prediction dictionary
    """
    seed = seed_to_long(seed_str.upper())

    result = {
        "seed": seed_str.upper(),
        "seed_value": seed,
        "ascension": ascension,
        "neow_option": neow_option,
        "act": 1,
        "map": None,
        "encounters": {"normal": [], "elite": []},
        "floors": [],
        "boss": None,
    }

    # ==========================================================================
    # 1. GENERATE MAP
    # ==========================================================================

    config = MapGeneratorConfig(ascension_level=ascension)
    map_rng = Random(seed + get_map_seed_offset(1))
    generator = MapGenerator(map_rng, config)
    dungeon_map = generator.generate()

    result["map"] = {
        "ascii": map_to_string(dungeon_map),
        "room_counts": {},
        "paths": [],
    }

    # Count room types
    for row in dungeon_map:
        for node in row:
            if node.room_type:
                key = node.room_type.name
                result["map"]["room_counts"][key] = result["map"]["room_counts"].get(key, 0) + 1

    # ==========================================================================
    # 2. GENERATE ENCOUNTERS
    # ==========================================================================

    monster_rng = Random(seed)
    normal_encounters, elite_encounters = generate_exordium_encounters(monster_rng)

    result["encounters"]["normal"] = normal_encounters
    result["encounters"]["elite"] = elite_encounters

    # ==========================================================================
    # 3. PREDICT EACH FLOOR
    # ==========================================================================

    # Initialize RNG state
    rng_state = GameRNGState(seed_str)

    # Apply Neow option
    rng_state.apply_neow_choice(neow_option)

    # Track rewards state
    reward_state = RewardState()
    reward_state.add_relic("PureWater")  # Watcher starting relic

    # Track encounter indices
    normal_idx = 0
    elite_idx = 0

    # Iterate through floors based on a sample path (leftmost nodes)
    for floor_num in range(1, 16):  # Floors 1-15
        rng_state.enter_floor(floor_num)

        floor_data = {
            "floor": floor_num,
            "room_type": None,
            "enemy": None,
            "enemy_hp": None,
            "enemy_first_move": None,
            "gold_reward": None,
            "card_rewards": None,
            "potion_drop": None,
            "relic_reward": None,
        }

        # Determine room type from map (use column 3 as representative path)
        row = dungeon_map[floor_num - 1] if floor_num <= 15 else None
        if row:
            # Find a connected node in this row
            for node in row:
                if node.room_type is not None:
                    floor_data["room_type"] = node.room_type.name
                    break

        # Get HP RNG for this floor
        hp_rng = Random(seed + floor_num)

        # =======================================================================
        # COMBAT FLOORS (Monster/Elite)
        # =======================================================================

        if floor_data["room_type"] in ["MONSTER", "ELITE"]:
            # Get encounter
            if floor_data["room_type"] == "ELITE":
                if elite_idx < len(elite_encounters):
                    enemy_name = elite_encounters[elite_idx]
                    elite_idx += 1
                else:
                    enemy_name = "Unknown Elite"
            else:
                if normal_idx < len(normal_encounters):
                    enemy_name = normal_encounters[normal_idx]
                    normal_idx += 1
                else:
                    enemy_name = "Unknown Monster"

            floor_data["enemy"] = enemy_name

            # Get HP
            if enemy_name in ENEMY_HP_RANGES:
                hp_range = ENEMY_HP_RANGES.get(enemy_name)
                if hp_range:
                    hp = hp_rng.random_int_range(hp_range[0], hp_range[1])
                    if ascension >= 7:
                        hp = int(hp * 1.07)
                    floor_data["enemy_hp"] = hp
            elif enemy_name in MULTI_ENEMY_HP:
                # Multi-enemy encounter
                enemies_hp = []
                for enemy_info in MULTI_ENEMY_HP[enemy_name]:
                    hp = hp_rng.random_int_range(
                        enemy_info["hp_range"][0],
                        enemy_info["hp_range"][1]
                    )
                    if ascension >= 7:
                        hp = int(hp * 1.07)
                    enemies_hp.append({
                        "name": enemy_info["name"],
                        "hp": hp
                    })
                floor_data["enemy_hp"] = enemies_hp

            # Get first move pattern
            if enemy_name in ENEMY_FIRST_MOVES:
                floor_data["enemy_first_move"] = ENEMY_FIRST_MOVES[enemy_name]

            # Gold reward
            room_type_for_gold = "elite" if floor_data["room_type"] == "ELITE" else "normal"
            treasure_rng = rng_state.get_rng(RNGStream.TREASURE)
            floor_data["gold_reward"] = generate_gold_reward(
                treasure_rng, room_type_for_gold, ascension
            )

            # Card rewards
            card_rng = rng_state.get_rng(RNGStream.CARD)
            room_type_for_cards = "elite" if floor_data["room_type"] == "ELITE" else "normal"
            cards = generate_card_rewards(
                card_rng, reward_state, act=1, player_class="WATCHER",
                ascension=ascension, room_type=room_type_for_cards
            )
            floor_data["card_rewards"] = [
                {"name": c.name, "rarity": c.rarity.name, "upgraded": c.upgraded}
                for c in cards
            ]

            # Potion drop
            potion_rng = rng_state.get_rng(RNGStream.POTION)
            dropped, potion = check_potion_drop(
                potion_rng, reward_state, room_type_for_cards
            )
            if dropped and potion:
                floor_data["potion_drop"] = {
                    "name": potion.name,
                    "rarity": potion.rarity.name
                }

            # Elite relic reward
            if floor_data["room_type"] == "ELITE":
                relic_rng = rng_state.get_rng(RNGStream.RELIC)
                relic = generate_elite_relic_reward(
                    relic_rng, reward_state, "WATCHER", act=1
                )
                if relic:
                    floor_data["relic_reward"] = {
                        "name": relic.name,
                        "tier": relic.tier.name
                    }

            # Advance RNG state
            rng_state.apply_combat(room_type_for_gold)

        # =======================================================================
        # EVENT FLOORS
        # =======================================================================

        elif floor_data["room_type"] == "EVENT":
            event_rng = rng_state.get_rng(RNGStream.EVENT)
            # Event selection is complex - just note that it uses eventRng
            floor_data["note"] = "Event selection uses eventRng. Most events use miscRng, not cardRng."
            rng_state.apply_event()

        # =======================================================================
        # SHOP FLOORS
        # =======================================================================

        elif floor_data["room_type"] == "SHOP":
            floor_data["note"] = "Shop consumes ~12 cardRng calls for card generation"
            rng_state.apply_shop()

        # =======================================================================
        # TREASURE FLOORS
        # =======================================================================

        elif floor_data["room_type"] == "TREASURE":
            floor_data["note"] = "Treasure room uses treasureRng only (no cardRng)"
            rng_state.apply_treasure()

        # =======================================================================
        # REST FLOORS
        # =======================================================================

        elif floor_data["room_type"] == "REST":
            floor_data["note"] = "Rest site - no RNG consumption"
            rng_state.apply_rest()

        result["floors"].append(floor_data)

    # ==========================================================================
    # 4. BOSS PREDICTION
    # ==========================================================================

    # Boss is determined by monsterRng during dungeon init
    # For Act 1, it's one of: Slime Boss, The Guardian, Hexaghost
    boss_rng = Random(seed)

    # Skip past all normal/elite encounters to get to boss roll
    # The boss is selected during generateMonsters() after all normal/elite lists
    for _ in range(len(normal_encounters) + len(elite_encounters) + 10):
        boss_rng.random_float()  # Advance past encounter rolls

    # Actual boss selection uses remaining monsterRng state
    boss_idx = boss_rng.random(2)  # 0, 1, or 2
    boss_name = ACT1_BOSSES[boss_idx]

    boss_hp_ranges = {
        "Slime Boss": (140, 140),  # Fixed HP
        "The Guardian": (240, 240),
        "Hexaghost": (250, 250),
    }

    boss_hp = boss_hp_ranges.get(boss_name, (0, 0))[0]
    if ascension >= 9:
        boss_hp = int(boss_hp * 1.1)  # 10% more HP at A9+

    result["boss"] = {
        "name": boss_name,
        "hp": boss_hp,
        "first_move": ENEMY_FIRST_MOVES.get(boss_name, {}),
        "boss_relics": None,
    }

    # Boss relic choices
    relic_rng = Random(seed, rng_state.get_counter(RNGStream.RELIC))
    boss_relics = generate_boss_relics(relic_rng, reward_state, "WATCHER", act=1)
    result["boss"]["boss_relics"] = [
        {"name": r.name, "tier": r.tier.name} for r in boss_relics
    ]

    return result


# =============================================================================
# FORMATTING
# =============================================================================

def format_prediction(pred: Dict[str, Any]) -> str:
    """Format prediction as readable markdown."""
    lines = []

    lines.append(f"# Seed: {pred['seed']}")
    lines.append(f"")
    lines.append(f"**Seed Value:** {pred['seed_value']}")
    lines.append(f"**Ascension:** {pred['ascension']}")
    lines.append(f"**Neow Option:** {pred['neow_option']}")
    lines.append(f"")

    # Map
    lines.append("## Map Layout")
    lines.append("```")
    lines.append(pred["map"]["ascii"])
    lines.append("```")
    lines.append("")
    lines.append("**Room Counts:**")
    for room_type, count in sorted(pred["map"]["room_counts"].items()):
        lines.append(f"- {room_type}: {count}")
    lines.append("")

    # Encounters
    lines.append("## Pre-Generated Encounters")
    lines.append("")
    lines.append("### Normal Encounters (in order)")
    for i, enc in enumerate(pred["encounters"]["normal"], 1):
        floor_type = "WEAK" if i <= 3 else "STRONG"
        lines.append(f"{i}. **{enc}** ({floor_type})")
    lines.append("")
    lines.append("### Elite Encounters (in order)")
    for i, enc in enumerate(pred["encounters"]["elite"], 1):
        lines.append(f"{i}. **{enc}**")
    lines.append("")

    # Floor-by-floor details
    lines.append("## Floor-by-Floor Details")
    lines.append("")

    for floor in pred["floors"]:
        lines.append(f"### Floor {floor['floor']} - {floor['room_type'] or 'Unknown'}")

        if floor["enemy"]:
            lines.append(f"**Enemy:** {floor['enemy']}")

            if isinstance(floor["enemy_hp"], list):
                for enemy in floor["enemy_hp"]:
                    lines.append(f"  - {enemy['name']}: {enemy['hp']} HP")
            elif floor["enemy_hp"]:
                lines.append(f"  - HP: {floor['enemy_hp']}")

            if floor["enemy_first_move"]:
                move = floor["enemy_first_move"]
                if "move" in move:
                    lines.append(f"  - First Move: **{move['move']}** ({move['intent']})")
                    if "damage" in move:
                        lines.append(f"    - Damage: {move['damage']}")
                elif "move_pattern" in move:
                    lines.append(f"  - Pattern: {move['move_pattern']}")
                if "details" in move:
                    lines.append(f"  - Details: {move['details']}")

        if floor["gold_reward"]:
            lines.append(f"**Gold:** {floor['gold_reward']}")

        if floor["card_rewards"]:
            cards = ", ".join([
                f"{c['name']}{'+'if c['upgraded'] else ''} ({c['rarity']})"
                for c in floor["card_rewards"]
            ])
            lines.append(f"**Card Rewards:** {cards}")

        if floor["potion_drop"]:
            lines.append(f"**Potion:** {floor['potion_drop']['name']} ({floor['potion_drop']['rarity']})")

        if floor["relic_reward"]:
            lines.append(f"**Relic:** {floor['relic_reward']['name']} ({floor['relic_reward']['tier']})")

        if floor.get("note"):
            lines.append(f"*{floor['note']}*")

        lines.append("")

    # Boss
    lines.append("## Boss")
    boss = pred["boss"]
    lines.append(f"**{boss['name']}** - {boss['hp']} HP")
    if boss["first_move"]:
        move = boss["first_move"]
        if "move" in move:
            lines.append(f"- First Move: **{move['move']}** ({move['intent']})")
        lines.append(f"- Details: {move.get('details', 'N/A')}")
    lines.append("")
    lines.append("**Boss Relic Choices:**")
    for relic in boss["boss_relics"]:
        lines.append(f"- {relic['name']} ({relic['tier']})")
    lines.append("")

    return "\n".join(lines)


# =============================================================================
# MAIN
# =============================================================================

def main():
    # Default test seeds or use command line args
    if len(sys.argv) > 1:
        seeds = sys.argv[1:4]  # Up to 3 seeds
    else:
        seeds = ["TEST123", "WATCHER", "A20RUN"]

    print("=" * 80)
    print("FULL ACT 1 SEED PREDICTIONS")
    print("=" * 80)
    print()
    print("NOTE: These predictions assume:")
    print("- All content unlocked (use generate_full_unlocks.py)")
    print("- Neow option: HUNDRED_GOLD (no cardRng consumption)")
    print("- Watcher character")
    print("- Ascension 0 (modify in code for A20)")
    print()

    for seed_str in seeds:
        print("=" * 80)
        pred = predict_full_act1(seed_str, ascension=0, neow_option="HUNDRED_GOLD")
        print(format_prediction(pred))


if __name__ == "__main__":
    main()
