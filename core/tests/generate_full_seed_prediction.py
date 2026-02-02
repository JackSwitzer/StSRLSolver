#!/usr/bin/env python3
"""
Generate Complete Seed Predictions for All Acts (A20)

Creates comprehensive prediction files for full game testing:
- All 4 acts with map layouts
- Every encounter with HP and first moves
- Card rewards, gold, potions, relics
- Shop inventory predictions
- Event predictions
- Boss relic choices

Usage:
    python -m core.tests.generate_full_seed_prediction SEED1 [SEED2] [SEED3]

Output:
    docs/vault/seed-SEEDNAME-full-prediction.md
"""

import os
import sys
from typing import Dict, List, Optional, Tuple, Any
from dataclasses import dataclass

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
    generate_exordium_encounters, generate_ending_encounters,
    get_enemy_hp, ENEMY_HP_RANGES,
    normalize_weights, roll_monster, populate_monster_list, MonsterInfo,
    ENDING_ELITE, ENDING_BOSS,
)
from core.generation.rewards import (
    RewardState, generate_card_rewards, generate_gold_reward,
    check_potion_drop, generate_elite_relic_reward, generate_boss_relics,
    generate_shop_inventory
)


# =============================================================================
# ASCENSION 20 MODIFIERS
# =============================================================================

A20_MODIFIERS = """
## Ascension 20 Modifiers

| Ascension | Effect |
|-----------|--------|
| A1 | +1 elite per act |
| A2 | +1 enemy damage (weak monsters) |
| A3 | -1 potion slot |
| A4 | +1 enemy damage (elites) |
| A5 | Heal only 75% at rest |
| A6 | Start with Ascender's Bane curse |
| A7 | +7% enemy HP |
| A8 | +1 enemy damage (bosses) |
| A9 | +10% boss HP |
| A10 | Start with 10% less gold |
| A11 | Start at 90% HP |
| A12 | -50% card upgrade chance |
| A13 | Reduced gold drops |
| A14 | More ? room combats |
| A15 | +1 elite per act (total +2) |
| A16 | +7% more enemy HP (total +14%) |
| A17 | No healing between acts |
| A18 | -1 potion slot (total -2) |
| A19 | +1 enemy damage (all) |
| A20 | Double boss fight at end |
"""


# =============================================================================
# ACT 2 (THE CITY) ENCOUNTERS
# =============================================================================

def get_city_weak_pool() -> List[MonsterInfo]:
    """Weak monster pool for Act 2 floors 1-2."""
    return normalize_weights([
        MonsterInfo("Spheric Guardian", 2.0),
        MonsterInfo("Chosen", 2.0),
        MonsterInfo("Shell Parasite", 2.0),
        MonsterInfo("3 Byrds", 2.0),
        MonsterInfo("2 Thieves", 2.0),
    ])


def get_city_strong_pool() -> List[MonsterInfo]:
    """Strong monster pool for Act 2 floors 3+."""
    return normalize_weights([
        MonsterInfo("Chosen and Byrds", 2.0),
        MonsterInfo("Cultist and Chosen", 1.0),
        MonsterInfo("Snecko", 4.0),
        MonsterInfo("Shelled Parasite and Fungi", 1.0),
        MonsterInfo("Snake Plant", 6.0),
        MonsterInfo("Centurion and Healer", 6.0),
        MonsterInfo("3 Cultists", 3.0),
        MonsterInfo("3 Darklings", 1.0),  # Can appear in City
    ])


def get_city_elite_pool() -> List[MonsterInfo]:
    """Elite monster pool for Act 2."""
    return normalize_weights([
        MonsterInfo("Gremlin Leader", 1.0),
        MonsterInfo("Slavers", 1.0),
        MonsterInfo("Book of Stabbing", 1.0),
    ])


def generate_city_encounters(rng: Random) -> Tuple[List[str], List[str]]:
    """Generate Act 2 encounters."""
    monster_list: List[str] = []
    elite_list: List[str] = []

    weak_pool = get_city_weak_pool()
    populate_monster_list(monster_list, weak_pool, rng, 2)

    strong_pool = get_city_strong_pool()
    populate_monster_list(monster_list, strong_pool, rng, 12)

    elite_pool = get_city_elite_pool()
    populate_monster_list(elite_list, elite_pool, rng, 10, is_elite=True)

    return monster_list, elite_list


# =============================================================================
# ACT 3 (THE BEYOND) ENCOUNTERS
# =============================================================================

def get_beyond_weak_pool() -> List[MonsterInfo]:
    """Weak monster pool for Act 3 floors 1-2."""
    return normalize_weights([
        MonsterInfo("3 Darklings", 2.0),
        MonsterInfo("Orb Walker", 2.0),
        MonsterInfo("3 Shapes", 2.0),
    ])


def get_beyond_strong_pool() -> List[MonsterInfo]:
    """Strong monster pool for Act 3 floors 3+."""
    return normalize_weights([
        MonsterInfo("Spire Growth", 1.0),
        MonsterInfo("Transient", 1.0),
        MonsterInfo("4 Shapes", 1.0),
        MonsterInfo("Maw", 2.0),
        MonsterInfo("Jaw Worm Horde", 2.0),
        MonsterInfo("Sphere and 2 Shapes", 1.0),
        MonsterInfo("Writhing Mass", 3.0),
        MonsterInfo("Giant Head", 0.0),  # Elite only
        MonsterInfo("Reptomancer", 0.0),  # Elite only
    ])


def get_beyond_elite_pool() -> List[MonsterInfo]:
    """Elite monster pool for Act 3."""
    return normalize_weights([
        MonsterInfo("Giant Head", 1.0),
        MonsterInfo("Nemesis", 1.0),
        MonsterInfo("Reptomancer", 1.0),
    ])


def generate_beyond_encounters(rng: Random) -> Tuple[List[str], List[str]]:
    """Generate Act 3 encounters."""
    monster_list: List[str] = []
    elite_list: List[str] = []

    weak_pool = get_beyond_weak_pool()
    populate_monster_list(monster_list, weak_pool, rng, 2)

    strong_pool = get_beyond_strong_pool()
    # Filter out elites from strong pool
    strong_pool = [m for m in strong_pool if m.weight > 0]
    populate_monster_list(monster_list, strong_pool, rng, 12)

    elite_pool = get_beyond_elite_pool()
    populate_monster_list(elite_list, elite_pool, rng, 10, is_elite=True)

    return monster_list, elite_list


# =============================================================================
# BOSS DATA
# =============================================================================

BOSSES_BY_ACT = {
    1: [
        {"name": "Slime Boss", "hp": 140, "a9_hp": 154},
        {"name": "The Guardian", "hp": 240, "a9_hp": 264},
        {"name": "Hexaghost", "hp": 250, "a9_hp": 275},
    ],
    2: [
        {"name": "The Champ", "hp": 420, "a9_hp": 462},
        {"name": "The Collector", "hp": 282, "a9_hp": 310},  # Has minions
        {"name": "Automaton", "hp": 300, "a9_hp": 330},
    ],
    3: [
        {"name": "Awakened One", "hp": 300, "a9_hp": 330, "phase2_hp": 200},
        {"name": "Time Eater", "hp": 456, "a9_hp": 502},
        {"name": "Donu and Deca", "hp": 500, "a9_hp": 550},  # Combined HP
    ],
    4: [
        {"name": "Corrupt Heart", "hp": 800, "a9_hp": 880},
    ],
}

BOSS_FIRST_MOVES = {
    # Act 1
    "Slime Boss": {"move": "Goop Spray", "intent": "DEBUFF", "details": "Shuffles 3 Slimed into deck. Splits at 50% HP."},
    "The Guardian": {"move": "Charging Up", "intent": "UNKNOWN", "details": "Enters Defensive Mode with 9 Block. Twin Slam (32 dmg) when mode ends."},
    "Hexaghost": {"move": "Activate", "intent": "UNKNOWN", "details": "Does nothing T1. T2: Divider = 6 x (HP/12 + 1) damage."},

    # Act 2
    "The Champ": {"move": "Defensive Stance", "intent": "DEFEND", "details": "Gains 15-18 Block. At 50% HP executes everyone."},
    "The Collector": {"move": "Spawn", "intent": "UNKNOWN", "details": "Summons 2 Torch Heads. Uses Mega Debuff (Vulnerable 3, Weak 3)."},
    "Automaton": {"move": "Spawn Orbs", "intent": "UNKNOWN", "details": "Summons 2 Bronze Orbs. Uses Hyper Beam (45 dmg, self stun)."},

    # Act 3
    "Awakened One": {"move": "Slash", "intent": "ATTACK", "details": "10 damage. Gains Curiosity (2 Str when you play Power). Phase 2: Reborn with 200 HP."},
    "Time Eater": {"move": "Ripple", "intent": "ATTACK_DEBUFF", "details": "7 dmg, applies Slow. After 12 cards: Haste (gain Str, heal, remove debuffs)."},
    "Donu and Deca": {"move": "Circle/Square", "intent": "BUFF", "details": "Donu buffs (+3 Str all), Deca blocks (+16 all). Kill Donu first."},

    # Act 4
    "Corrupt Heart": {"move": "Debilitate", "intent": "DEBUFF", "details": "Vulnerable 2, Weak 2, Frail 2. Gains 2 Invincible (max 300 dmg/turn). Beat of Death damages you per card."},
}


# =============================================================================
# ENEMY HP RANGES (Extended)
# =============================================================================

EXTENDED_HP_RANGES = {
    # Act 1 - Already in encounters.py, adding missing ones
    "Cultist": (48, 54),
    "Jaw Worm": (40, 44),
    "Blue Slaver": (46, 50),
    "Red Slaver": (46, 50),
    "Looter": (44, 48),
    "Gremlin Nob": (82, 86),
    "Lagavulin": (109, 111),
    "Sentry": (38, 42),
    "Fungi Beast": (22, 28),

    # Act 2
    "Chosen": (95, 99),
    "Spheric Guardian": (20, 20),  # Fixed HP, but gains Artifact
    "Shell Parasite": (68, 72),
    "Byrd": (25, 31),
    "Mugger": (48, 52),
    "Snecko": (114, 120),
    "Snake Plant": (75, 79),
    "Centurion": (76, 80),
    "Mystic": (48, 56),  # Healer
    "Cultist_Hard": (54, 60),
    "Gremlin Leader": (140, 148),
    "Slaver_Blue_Elite": (46, 50),
    "Slaver_Red_Elite": (46, 50),
    "Taskmaster": (64, 68),
    "Book of Stabbing": (160, 168),

    # Act 3
    "Darkling": (48, 56),
    "Orb Walker": (90, 96),
    "Spiker": (42, 56),
    "Repulsor": (29, 35),
    "Writhing Mass": (160, 160),
    "Transient": (999, 999),  # Flees after 5 turns
    "Spire Growth": (170, 190),
    "Maw": (300, 300),
    "Giant Head": (500, 520),
    "Nemesis": (185, 200),
    "Reptomancer": (180, 190),
    "Shape": (20, 25),  # Exploder/Spire Worm

    # Act 4
    "Shield": (120, 120),  # Shield and Spear
    "Spear": (160, 160),
}

ENEMY_FIRST_MOVES = {
    # Act 1
    "Cultist": {"move": "Incantation", "intent": "BUFF", "details": "Gains 3 Ritual (+3 Str/turn)"},
    "Jaw Worm": {"move": "Chomp", "intent": "ATTACK", "damage": 11, "details": "11 dmg (12 at A2+)"},
    "Blue Slaver": {"move": "Stab", "intent": "ATTACK", "damage": 12, "details": "12 dmg (13 at A2+)"},
    "Red Slaver": {"move": "Stab", "intent": "ATTACK", "damage": 13, "details": "13 dmg (14 at A2+)"},
    "Looter": {"move": "Mug", "intent": "ATTACK", "damage": 10, "details": "Steals 15 gold"},
    "Gremlin Nob": {"move": "Bellow", "intent": "BUFF", "details": "Gains 2 Enrage (2 Str per Skill you play)"},
    "Lagavulin": {"move": "Sleep", "intent": "SLEEP", "details": "Asleep with 8 Metallicize. Wakes after 3 turns or 10+ dmg hit."},
    "3 Sentries": {"move_pattern": "Bolt x2 + Beam", "details": "9 dmg each. Beam adds Dazed. Alternates."},
    "Fungi Beast": {"move": "Grow", "intent": "BUFF", "details": "60% Grow (+3 Str), 40% Bite (6 dmg)"},

    # Act 2
    "Chosen": {"move": "Poke", "intent": "ATTACK", "damage": 5, "details": "If < 50% HP: Debilitate (Vulnerable 2, Weak 2, Hex - add Dazed on draw)"},
    "Spheric Guardian": {"move": "Slam", "intent": "ATTACK", "damage": 10, "details": "Has 40 Block. Loses 4 Artifact when you attack."},
    "Shell Parasite": {"move": "Double Strike", "intent": "ATTACK", "damage": 7, "hits": 2, "details": "7x2 dmg. Applies Frail with Suck."},
    "Byrd": {"move": "Caw", "intent": "BUFF", "details": "Gains 1 Str. Flying: 50% miss chance."},
    "Snecko": {"move": "Perplexing Glare", "intent": "DEBUFF", "details": "Applies 2 Confused (random card costs 0-3)"},
    "Snake Plant": {"move": "Chomp", "intent": "ATTACK", "damage": 7, "hits": 3, "details": "7x3 dmg. Applies 2 Weak with Enfeebling Spores."},
    "Centurion": {"move": "Slash", "intent": "ATTACK", "damage": 12, "details": "12 dmg. Fury gains 5 Str."},
    "Mystic": {"move": "Heal", "intent": "BUFF", "details": "Heals ally for 16 HP"},
    "Gremlin Leader": {"move": "Encourage", "intent": "BUFF", "details": "Summons gremlins. All gain 3 Str and 6 Block."},
    "Slavers": {"move_pattern": "Mixed attacks", "details": "Blue: Stab 12. Red: Stab 13. Taskmaster: Whip 7."},
    "Book of Stabbing": {"move": "Multi-Stab", "intent": "ATTACK", "damage": 6, "hits": "3+", "details": "6 dmg x (3 + turn count)"},

    # Act 3
    "Darkling": {"move": "Nip", "intent": "ATTACK", "damage": 7, "details": "If ally dies, gains 2 Str. Regrows if all 3 not killed same turn."},
    "Orb Walker": {"move": "Claw", "intent": "ATTACK", "damage": 15, "details": "Burns a card in hand."},
    "Writhing Mass": {"move": "Implant", "intent": "UNKNOWN", "details": "Copies a card from your deck. Random intent each turn. Malleable (gains Block when hit)."},
    "Transient": {"move": "Attack", "intent": "ATTACK", "damage": 30, "details": "30 dmg (+10/turn). Flees turn 5. Only gives gold if killed."},
    "Giant Head": {"move": "Glare", "intent": "DEBUFF", "details": "Applies 1 Weak. Slow debuff. 35-40 dmg attacks."},
    "Nemesis": {"move": "Debuff", "intent": "DEBUFF", "details": "Burns cards. Goes intangible (1 dmg max). 45 dmg Scythe attack."},
    "Reptomancer": {"move": "Summon", "intent": "UNKNOWN", "details": "Summons 2 Daggers. Big Attack (25 dmg x2). Daggers hit hard (9x2 dmg)."},
    "Maw": {"move": "Slam", "intent": "ATTACK", "damage": 25, "details": "25 dmg. Roar weakens. HUNGRY doubles damage when low HP."},

    # Act 4
    "Shield and Spear": {"move_pattern": "Shield defends, Spear attacks", "details": "Shield: gains 30 Block. Spear: Strong Stab 30 dmg."},
    "Corrupt Heart": {"move": "Debilitate", "intent": "STRONG_DEBUFF", "details": "Vulnerable 2, Weak 2, Frail 2. Max 300 dmg/turn. Beat of Death: 2 dmg per card."},
}


# =============================================================================
# FULL GAME PREDICTION
# =============================================================================

def predict_full_game(seed_str: str, ascension: int = 20, neow_option: str = "HUNDRED_GOLD") -> Dict[str, Any]:
    """Generate comprehensive predictions for all 4 acts."""
    seed = seed_to_long(seed_str.upper())

    result = {
        "seed": seed_str.upper(),
        "seed_value": seed,
        "ascension": ascension,
        "neow_option": neow_option,
        "acts": [],
    }

    # Initialize persistent RNG state
    rng_state = GameRNGState(seed_str)
    rng_state.apply_neow_choice(neow_option)

    reward_state = RewardState()
    reward_state.add_relic("PureWater")  # Watcher starting relic

    # Track global encounter indices
    total_normal_idx = 0
    total_elite_idx = 0

    for act_num in range(1, 5):
        act_data = {
            "act": act_num,
            "name": ["Exordium", "The City", "The Beyond", "The Ending"][act_num - 1],
            "map": None,
            "encounters": {"normal": [], "elite": []},
            "floors": [],
            "boss": None,
        }

        # ======================================================================
        # GENERATE MAP
        # ======================================================================

        if act_num < 4:
            config = MapGeneratorConfig(ascension_level=ascension)
            map_rng = Random(seed + get_map_seed_offset(act_num))
            generator = MapGenerator(map_rng, config)
            dungeon_map = generator.generate()

            act_data["map"] = {
                "ascii": map_to_string(dungeon_map),
                "room_counts": {},
            }

            for row in dungeon_map:
                for node in row:
                    if node.room_type:
                        key = node.room_type.name
                        act_data["map"]["room_counts"][key] = act_data["map"]["room_counts"].get(key, 0) + 1
        else:
            # Act 4 has fixed map
            act_data["map"] = {
                "ascii": "Rest -> Shop -> Elite (Shield & Spear) -> Boss (Heart)",
                "room_counts": {"REST": 1, "SHOP": 1, "ELITE": 1, "BOSS": 1},
            }

        # ======================================================================
        # GENERATE ENCOUNTERS
        # ======================================================================

        monster_rng = Random(seed)

        if act_num == 1:
            from core.generation.encounters import generate_exordium_encounters
            normal_encounters, elite_encounters = generate_exordium_encounters(monster_rng)
        elif act_num == 2:
            normal_encounters, elite_encounters = generate_city_encounters(monster_rng)
        elif act_num == 3:
            normal_encounters, elite_encounters = generate_beyond_encounters(monster_rng)
        else:
            # Act 4 has fixed encounters
            normal_encounters, elite_encounters, _ = generate_ending_encounters()

        act_data["encounters"]["normal"] = normal_encounters
        act_data["encounters"]["elite"] = elite_encounters

        # ======================================================================
        # FLOOR-BY-FLOOR PREDICTIONS
        # ======================================================================

        num_floors = 15 if act_num < 4 else 4
        normal_idx = 0
        elite_idx = 0

        for floor_num in range(1, num_floors + 1):
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

            # Determine room type based on typical path (simplified)
            if act_num < 4 and dungeon_map:
                row = dungeon_map[floor_num - 1] if floor_num <= 15 else None
                if row:
                    for node in row:
                        if node.room_type is not None:
                            floor_data["room_type"] = node.room_type.name
                            break
            elif act_num == 4:
                floor_data["room_type"] = ["REST", "SHOP", "ELITE", "BOSS"][floor_num - 1]

            hp_rng = Random(seed + floor_num + (act_num - 1) * 100)

            # Combat floors
            if floor_data["room_type"] in ["MONSTER", "ELITE", "BOSS"]:
                if floor_data["room_type"] == "ELITE":
                    if elite_idx < len(elite_encounters):
                        enemy_name = elite_encounters[elite_idx]
                        elite_idx += 1
                    else:
                        enemy_name = "Unknown Elite"
                elif floor_data["room_type"] == "BOSS":
                    enemy_name = "Boss"  # Will be set below
                else:
                    if normal_idx < len(normal_encounters):
                        enemy_name = normal_encounters[normal_idx]
                        normal_idx += 1
                    else:
                        enemy_name = "Unknown Monster"

                floor_data["enemy"] = enemy_name

                # Get HP with A20 modifier (14% more HP at A16+)
                hp_range = EXTENDED_HP_RANGES.get(enemy_name)
                if hp_range:
                    hp = hp_rng.random_int_range(hp_range[0], hp_range[1])
                    if ascension >= 7:
                        hp = int(hp * 1.07)
                    if ascension >= 16:
                        hp = int(hp * 1.07)  # Additional 7%
                    floor_data["enemy_hp"] = hp

                # Get first move
                if enemy_name in ENEMY_FIRST_MOVES:
                    floor_data["enemy_first_move"] = ENEMY_FIRST_MOVES[enemy_name]

                # Gold reward
                if floor_data["room_type"] != "BOSS":
                    room_type_for_gold = "elite" if floor_data["room_type"] == "ELITE" else "normal"
                    treasure_rng = rng_state.get_rng(RNGStream.TREASURE)
                    floor_data["gold_reward"] = generate_gold_reward(
                        treasure_rng, room_type_for_gold, ascension
                    )

                # Card rewards
                card_rng = rng_state.get_rng(RNGStream.CARD)
                room_type_for_cards = "elite" if floor_data["room_type"] == "ELITE" else "normal"
                cards = generate_card_rewards(
                    card_rng, reward_state, act=act_num, player_class="WATCHER",
                    ascension=ascension, room_type=room_type_for_cards
                )
                floor_data["card_rewards"] = [
                    {"name": c.name, "rarity": c.rarity.name, "upgraded": c.upgraded}
                    for c in cards
                ]

                # Potion drop
                potion_rng = rng_state.get_rng(RNGStream.POTION)
                dropped, potion = check_potion_drop(potion_rng, reward_state, room_type_for_cards)
                if dropped and potion:
                    floor_data["potion_drop"] = {"name": potion.name, "rarity": potion.rarity.name}

                # Elite relic
                if floor_data["room_type"] == "ELITE":
                    relic_rng = rng_state.get_rng(RNGStream.RELIC)
                    relic = generate_elite_relic_reward(relic_rng, reward_state, "WATCHER", act=act_num)
                    if relic:
                        floor_data["relic_reward"] = {"name": relic.name, "tier": relic.tier.name}
                    rng_state.apply_combat("elite")
                else:
                    rng_state.apply_combat("monster")

            elif floor_data["room_type"] == "SHOP":
                floor_data["note"] = "Shop - generates cards (12+ cardRng calls), relics, potions"
                # Generate shop preview
                merchant_rng = rng_state.get_rng(RNGStream.MERCHANT)
                try:
                    shop = generate_shop_inventory(
                        merchant_rng, reward_state, act=act_num, player_class="WATCHER", ascension=ascension
                    )
                    floor_data["shop"] = {
                        "colored_cards": [(c.name, p) for c, p in shop.colored_cards],
                        "colorless_cards": [(c.name, p) for c, p in shop.colorless_cards],
                        "relics": [(r.name, p) for r, p in shop.relics],
                        "potions": [(p.name, pr) for p, pr in shop.potions],
                        "purge_cost": shop.purge_cost,
                    }
                except:
                    floor_data["shop"] = {"error": "Could not generate shop"}
                rng_state.apply_shop()

            elif floor_data["room_type"] == "EVENT":
                floor_data["note"] = "Event room - uses eventRng/miscRng (most don't affect cardRng)"
                rng_state.apply_event()

            elif floor_data["room_type"] == "TREASURE":
                floor_data["note"] = "Treasure chest - uses treasureRng only"
                rng_state.apply_treasure()

            elif floor_data["room_type"] == "REST":
                floor_data["note"] = "Rest site - Rest (heal 30% or 22.5% at A5+), Upgrade, Smith, Dig, Toke, Lift"
                rng_state.apply_rest()

            act_data["floors"].append(floor_data)

        # ======================================================================
        # BOSS PREDICTION
        # ======================================================================

        bosses = BOSSES_BY_ACT.get(act_num, [])
        if bosses:
            if act_num < 4:
                boss_rng = Random(seed)
                # Advance past encounter rolls
                for _ in range(len(normal_encounters) + len(elite_encounters) + 20):
                    boss_rng.random_float()
                boss_idx = boss_rng.random(len(bosses) - 1)
                boss = bosses[boss_idx]
            else:
                boss = bosses[0]  # Heart is fixed

            boss_hp = boss.get("a9_hp", boss["hp"]) if ascension >= 9 else boss["hp"]

            act_data["boss"] = {
                "name": boss["name"],
                "hp": boss_hp,
                "first_move": BOSS_FIRST_MOVES.get(boss["name"], {}),
            }

            # Boss relic choices (not for Act 4)
            if act_num < 4:
                relic_rng = Random(seed, rng_state.get_counter(RNGStream.RELIC))
                boss_relics = generate_boss_relics(relic_rng, reward_state, "WATCHER", act=act_num)
                act_data["boss"]["boss_relics"] = [
                    {"name": r.name, "tier": r.tier.name} for r in boss_relics
                ]

        result["acts"].append(act_data)

        # Act transition - cardRng snapping
        if act_num < 4:
            rng_state.transition_to_next_act()

    return result


# =============================================================================
# MARKDOWN FORMATTING
# =============================================================================

def format_full_prediction_md(pred: Dict[str, Any]) -> str:
    """Format full prediction as markdown."""
    lines = []

    lines.append(f"# Full Game Prediction: Seed {pred['seed']} ({pred['seed_value']})")
    lines.append("")
    lines.append(f"**Ascension:** {pred['ascension']}")
    lines.append(f"**Character:** Watcher")
    lines.append(f"**Neow Option:** {pred['neow_option']}")
    lines.append("")
    lines.append("---")
    lines.append("")
    lines.append(A20_MODIFIERS)
    lines.append("")
    lines.append("---")

    for act in pred["acts"]:
        lines.append("")
        lines.append(f"# Act {act['act']}: {act['name']}")
        lines.append("")

        # Map
        lines.append("## Map")
        lines.append("```")
        lines.append(act["map"]["ascii"])
        lines.append("```")
        lines.append("")
        lines.append("**Room Counts:**")
        for room_type, count in sorted(act["map"]["room_counts"].items()):
            lines.append(f"- {room_type}: {count}")
        lines.append("")

        # Encounters
        lines.append("## Encounters (Pre-Generated)")
        lines.append("")
        if act["encounters"]["normal"]:
            lines.append("### Normal Encounters")
            lines.append("| # | Encounter | Type |")
            lines.append("|---|-----------|------|")
            for i, enc in enumerate(act["encounters"]["normal"], 1):
                enc_type = "WEAK" if i <= 2 and act["act"] > 1 else ("WEAK" if i <= 3 and act["act"] == 1 else "STRONG")
                lines.append(f"| {i} | {enc} | {enc_type} |")
            lines.append("")

        if act["encounters"]["elite"]:
            lines.append("### Elite Encounters")
            lines.append("| # | Encounter |")
            lines.append("|---|-----------|")
            for i, enc in enumerate(act["encounters"]["elite"], 1):
                lines.append(f"| {i} | {enc} |")
            lines.append("")

        # Floor details
        lines.append("## Floor-by-Floor Details")
        lines.append("")

        for floor in act["floors"]:
            room_type = floor['room_type'] or 'Unknown'
            lines.append(f"### Floor {floor['floor']} - {room_type}")

            if floor["enemy"]:
                lines.append(f"**Enemy:** {floor['enemy']}")
                if floor["enemy_hp"]:
                    if isinstance(floor["enemy_hp"], list):
                        for enemy in floor["enemy_hp"]:
                            lines.append(f"- {enemy['name']}: {enemy['hp']} HP")
                    else:
                        lines.append(f"- HP: {floor['enemy_hp']}")

                if floor["enemy_first_move"]:
                    move = floor["enemy_first_move"]
                    if "move" in move:
                        dmg = f" ({move['damage']} dmg)" if "damage" in move else ""
                        lines.append(f"- **First Move:** {move['move']} [{move['intent']}]{dmg}")
                    elif "move_pattern" in move:
                        lines.append(f"- **Pattern:** {move['move_pattern']}")
                    if "details" in move:
                        lines.append(f"- *{move['details']}*")

            if floor["gold_reward"]:
                lines.append(f"**Gold:** {floor['gold_reward']}")

            if floor["card_rewards"]:
                cards = []
                for c in floor["card_rewards"]:
                    upgrade = "+" if c["upgraded"] else ""
                    cards.append(f"{c['name']}{upgrade} ({c['rarity'][0]})")
                lines.append(f"**Cards:** {', '.join(cards)}")

            if floor["potion_drop"]:
                lines.append(f"**Potion:** {floor['potion_drop']['name']} ({floor['potion_drop']['rarity']})")

            if floor["relic_reward"]:
                lines.append(f"**Relic:** {floor['relic_reward']['name']} ({floor['relic_reward']['tier']})")

            if floor.get("shop"):
                shop = floor["shop"]
                if "error" not in shop:
                    lines.append("**Shop Contents:**")
                    lines.append(f"- Cards: {', '.join([f'{c} ({p}g)' for c, p in shop['colored_cards']])}")
                    lines.append(f"- Colorless: {', '.join([f'{c} ({p}g)' for c, p in shop['colorless_cards']])}")
                    lines.append(f"- Relics: {', '.join([f'{r} ({p}g)' for r, p in shop['relics']])}")
                    lines.append(f"- Potions: {', '.join([f'{p} ({pr}g)' for p, pr in shop['potions']])}")
                    lines.append(f"- Card Remove: {shop['purge_cost']}g")

            if floor.get("note"):
                lines.append(f"*{floor['note']}*")

            lines.append("")

        # Boss
        if act["boss"]:
            boss = act["boss"]
            lines.append("## Boss")
            lines.append(f"### {boss['name']} ({boss['hp']} HP)")
            if boss["first_move"]:
                move = boss["first_move"]
                if "move" in move:
                    lines.append(f"- **First Move:** {move['move']} [{move.get('intent', 'UNKNOWN')}]")
                if "details" in move:
                    lines.append(f"- {move['details']}")

            if boss.get("boss_relics"):
                lines.append("")
                lines.append("**Boss Relic Choices:**")
                for i, relic in enumerate(boss["boss_relics"], 1):
                    lines.append(f"{i}. {relic['name']}")
            lines.append("")

        lines.append("---")

    lines.append("")
    lines.append("## Verification Checklist")
    lines.append("")
    lines.append("For each act, verify:")
    lines.append("- [ ] Floor 1-3 encounters match")
    lines.append("- [ ] Enemy HP values match")
    lines.append("- [ ] Card rewards match")
    lines.append("- [ ] Gold amounts match")
    lines.append("- [ ] Elite relic drops match")
    lines.append("- [ ] Boss matches")
    lines.append("- [ ] Boss relic choices match")
    lines.append("")
    lines.append("**Known Limitations:**")
    lines.append("- Shop visits shift cardRng for subsequent floors")
    lines.append("- Event choices may consume various RNG streams")
    lines.append("- Multi-enemy HP requires fresh RNG calls per enemy")
    lines.append("- Path through map affects actual room types encountered")

    return "\n".join(lines)


# =============================================================================
# MAIN
# =============================================================================

def main():
    if len(sys.argv) < 2:
        seeds = ["TEST123", "WATCHER", "A20WIN"]
    else:
        seeds = sys.argv[1:4]

    output_dir = os.path.join(_project_dir, "docs", "vault")
    os.makedirs(output_dir, exist_ok=True)

    print("=" * 80)
    print("FULL GAME SEED PREDICTION GENERATOR (A20)")
    print("=" * 80)
    print()

    for seed_str in seeds:
        print(f"Generating prediction for seed: {seed_str}...")

        pred = predict_full_game(seed_str, ascension=20, neow_option="HUNDRED_GOLD")
        md_content = format_full_prediction_md(pred)

        filename = f"seed-{seed_str.upper()}-{pred['seed_value']}-full-prediction.md"
        filepath = os.path.join(output_dir, filename)

        with open(filepath, "w") as f:
            f.write(md_content)

        print(f"  -> Saved to: {filepath}")

    print()
    print("Done! Files saved to docs/vault/")


if __name__ == "__main__":
    main()
