#!/usr/bin/env python3
"""
Slay the Spire RL - Command Line Interface

CLI for running and comparing Slay the Spire games against the real game.
Tests subsystems like map generation, rewards, encounters, and RNG.

Usage:
    uv run python cli.py run --seed ABC123 --ascension 20
    uv run python cli.py map --seed ABC123 --act 1
    uv run python cli.py rewards --seed ABC123 --floor 3
    uv run python cli.py encounter --seed ABC123 --floor 1
    uv run python cli.py rng --seed ABC123 --count 20
    uv run python cli.py interactive --seed ABC123
"""

import argparse
import json
import sys
import os
import random as py_random
from typing import List, Dict, Any, Optional, Tuple

# Add core to path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

from core.state.rng import Random, GameRNG, seed_to_long, long_to_seed, XorShift128
from core.state.run import RunState, create_watcher_run, CardInstance, RelicInstance
from core.generation.map import (
    MapGenerator, MapGeneratorConfig, RoomType, MapRoomNode,
    map_to_string, get_map_seed_offset, generate_act4_map
)
from core.generation.rewards import (
    generate_card_rewards, generate_gold_reward, check_potion_drop,
    generate_boss_relics, generate_elite_relic_reward,
    RewardState, CardBlizzardState, PotionBlizzardState
)
from core.content.enemies import (
    Enemy, EnemyState, MoveInfo, Intent,
    JawWorm, Cultist
)
from core.state.combat import CombatState, EntityState, EnemyCombatState, create_combat, create_enemy


# =============================================================================
# OUTPUT FORMATTING
# =============================================================================

def format_seed_info(seed_string: str, numeric_seed: int) -> str:
    """Format seed information header."""
    return f"Seed: {seed_string} (numeric: {numeric_seed})"


def format_map_node(node: MapRoomNode) -> str:
    """Format a single map node."""
    if node.room_type is None:
        return " "
    symbol = node.room_type.value
    if node.has_emerald_key:
        symbol += "*"
    return symbol


def format_map(nodes: List[List[MapRoomNode]], act: int, boss_name: str = None) -> str:
    """Format a complete map for display."""
    lines = []

    # Header
    lines.append(f"Act {act} Map:")

    # Show map from top to bottom (reverse order)
    for row_idx in range(len(nodes) - 1, -1, -1):
        row = nodes[row_idx]
        symbols = []
        for node in row:
            if node.has_edges() or (row_idx == len(nodes) - 1 and any(
                any(e.dst_x == node.x for e in lower_node.edges)
                for lower_node in nodes[row_idx - 1] if row_idx > 0
            )):
                symbols.append(f"[{format_map_node(node)}]")
            else:
                symbols.append("   ")
        floor_label = str(row_idx).rjust(2)
        lines.append(f"Floor {floor_label}: {' '.join(symbols)}")

    # Boss info
    if boss_name:
        lines.append(f"Floor 16: [BOSS: {boss_name}]")

    return "\n".join(lines)


def format_card_reward(cards: List, gold: int, potion: Optional[Any], potion_dropped: bool) -> str:
    """Format card rewards for display."""
    lines = []
    lines.append(f"Gold: {gold}")

    if potion_dropped and potion:
        lines.append(f"Potion: {potion.name} (dropped: True)")
    else:
        lines.append(f"Potion: None (dropped: False)")

    lines.append("Card Choices:")
    for i, card in enumerate(cards, 1):
        upgraded = "+" if card.upgraded else ""
        rarity = card.rarity.name.capitalize()
        card_type = card.card_type.name.capitalize() if hasattr(card, 'card_type') else "Unknown"
        lines.append(f"  {i}. {card.name}{upgraded} ({rarity}, {card_type})")

    return "\n".join(lines)


def format_rng_sequence(rng: XorShift128, count: int) -> List[int]:
    """Get a sequence of random values for verification."""
    values = []
    for _ in range(count):
        val = rng.next_int(100)  # 0-99
        values.append(val)
    return values


def format_enemy_encounter(enemies: List[Enemy], floor: int) -> str:
    """Format an enemy encounter for display."""
    lines = []
    lines.append(f"=== Floor {floor}: Monster Room ===")

    enemy_parts = []
    for enemy in enemies:
        hp = enemy.state.current_hp
        max_hp = enemy.state.max_hp
        enemy_parts.append(f"{enemy.NAME} (HP: {hp}/{max_hp})")

    lines.append(f"Encounter: {', '.join(enemy_parts)}")

    # Show first move intents
    for enemy in enemies:
        move = enemy.state.next_move
        if move:
            intent_str = move.intent.value
            if move.base_damage > 0:
                if move.hits > 1:
                    intent_str += f" {move.base_damage}x{move.hits}"
                else:
                    intent_str += f" {move.base_damage}"
            if move.block > 0:
                intent_str += f" +{move.block} Block"
            lines.append(f"  {enemy.NAME}: {move.name} ({intent_str})")

    return "\n".join(lines)


# =============================================================================
# ENCOUNTER GENERATION
# =============================================================================

# Act 1 encounter pools (simplified - add more as needed)
ACT1_WEAK_ENEMIES = [
    ("JawWorm",),
    ("Cultist",),
    # Add more weak encounters
]

ACT1_STRONG_ENEMIES = [
    ("JawWorm", "JawWorm"),
    # Add more strong encounters
]

ACT1_ELITES = [
    ("Lagavulin",),
    ("Gremlin Nob",),
    ("Sentries",),
]

ACT1_BOSSES = ["Slime Boss", "The Guardian", "Hexaghost"]


def create_enemy_by_id(enemy_id: str, ai_rng: Random, hp_rng: Random, ascension: int) -> Optional[Enemy]:
    """Create an enemy instance by ID."""
    enemy_classes = {
        "JawWorm": JawWorm,
        "Cultist": Cultist,
    }

    enemy_class = enemy_classes.get(enemy_id)
    if enemy_class:
        return enemy_class(ai_rng, ascension, hp_rng)
    return None


def get_encounter_for_floor(
    floor: int,
    act: int,
    room_type: str,
    monster_rng: Random,
    ai_rng: Random,
    hp_rng: Random,
    ascension: int
) -> List[Enemy]:
    """Generate enemies for a given floor encounter."""
    enemies = []

    if room_type == "monster":
        # Simple pool selection for Act 1
        if act == 1:
            if floor <= 3:
                # Early floors - weak enemies
                pool = ACT1_WEAK_ENEMIES
            else:
                # Later floors - stronger enemies
                pool = ACT1_WEAK_ENEMIES + ACT1_STRONG_ENEMIES

            # Roll for encounter
            idx = monster_rng.random(len(pool) - 1)
            encounter = pool[idx]

            for enemy_id in encounter:
                enemy = create_enemy_by_id(enemy_id, ai_rng.copy(), hp_rng.copy(), ascension)
                if enemy:
                    # Roll first move
                    enemy.roll_move()
                    enemies.append(enemy)

    elif room_type == "elite":
        if act == 1:
            idx = monster_rng.random(len(ACT1_ELITES) - 1)
            encounter = ACT1_ELITES[idx]

            for enemy_id in encounter:
                enemy = create_enemy_by_id(enemy_id, ai_rng.copy(), hp_rng.copy(), ascension)
                if enemy:
                    enemy.roll_move()
                    enemies.append(enemy)

    return enemies


def get_boss_for_act(act: int, seed: int) -> str:
    """Get the boss for a given act based on seed."""
    if act == 1:
        bosses = ACT1_BOSSES
    elif act == 2:
        bosses = ["Automaton", "Collector", "Champ"]
    elif act == 3:
        bosses = ["Awakened One", "Time Eater", "Donu & Deca"]
    else:
        return "The Heart"

    # Boss selection is deterministic from seed
    rng = Random(seed)
    idx = rng.random(len(bosses) - 1)
    return bosses[idx]


# =============================================================================
# COMMAND IMPLEMENTATIONS
# =============================================================================

def cmd_run(args) -> int:
    """Run a game simulation with random actions."""
    seed_string = args.seed.upper()
    seed = seed_to_long(seed_string)
    ascension = args.ascension
    max_floors = args.floors if args.floors else 50

    print(format_seed_info(seed_string, seed))
    print(f"Ascension: {ascension}")
    print(f"Character: Watcher")
    print(f"Max floors: {max_floors}")
    print()

    # Create run
    run = create_watcher_run(seed_string, ascension)
    game_rng = GameRNG(seed=seed)
    reward_state = RewardState()
    reward_state.add_relic("PureWater")

    # Simulate floors
    for floor_num in range(1, max_floors + 1):
        game_rng.advance_floor()
        run.advance_floor()

        # Get available paths
        paths = run.get_available_paths()
        if not paths:
            print(f"No available paths at floor {floor_num}")
            break

        # Pick random path
        path_idx = game_rng.misc_rng.random(len(paths) - 1)
        chosen_node = paths[path_idx]
        run.move_to(chosen_node.x, chosen_node.y)

        room_type = chosen_node.room_type
        print(f"=== Floor {floor_num}: {room_type.name if room_type else 'Unknown'} Room ===")

        if room_type == RoomType.MONSTER:
            # Generate encounter
            enemies = get_encounter_for_floor(
                floor_num, run.act, "monster",
                game_rng.monster_rng, game_rng.ai_rng, game_rng.monster_hp_rng,
                ascension
            )

            if enemies:
                enemy_strs = [f"{e.NAME} (HP: {e.state.current_hp})" for e in enemies]
                print(f"Encounter: {', '.join(enemy_strs)}")

            # Simulate simple combat outcome
            damage_taken = game_rng.misc_rng.random_range(5, 20)
            run.damage(damage_taken)
            print(f"Combat result: Took {damage_taken} damage")
            print(f"Player HP: {run.current_hp}/{run.max_hp}")

            # Generate rewards
            gold = generate_gold_reward(game_rng.treasure_rng, "normal", ascension)
            run.add_gold(gold)

            dropped, potion = check_potion_drop(
                game_rng.potion_rng, reward_state, "normal"
            )

            cards = generate_card_rewards(
                game_rng.card_rng, reward_state, run.act, "WATCHER",
                ascension, "normal"
            )

            print(f"Rewards: {gold} gold")
            if dropped and potion:
                print(f"  Potion: {potion.name}")
            print(f"  Card choices: {', '.join(c.name for c in cards)}")

            # Randomly pick a card or skip
            if cards and game_rng.misc_rng.random_boolean(0.7):
                card_idx = game_rng.misc_rng.random(len(cards) - 1)
                picked = cards[card_idx]
                run.add_card(picked.id, picked.upgraded)
                print(f"  Picked: {picked.name}")
            else:
                print(f"  Skipped card reward")

        elif room_type == RoomType.ELITE:
            print("Elite combat (simplified)")
            damage_taken = game_rng.misc_rng.random_range(15, 35)
            run.damage(damage_taken)
            print(f"Took {damage_taken} damage, HP: {run.current_hp}/{run.max_hp}")

            gold = generate_gold_reward(game_rng.treasure_rng, "elite", ascension)
            run.add_gold(gold)

            relic = generate_elite_relic_reward(game_rng.relic_rng, reward_state, "WATCHER", run.act)
            if relic:
                run.add_relic(relic.id)
                print(f"Rewards: {gold} gold, Relic: {relic.name}")

        elif room_type == RoomType.REST:
            # Rest or upgrade
            if run.hp_percent() < 0.7:
                heal_amount = int(run.max_hp * 0.3)
                run.heal(heal_amount)
                print(f"Rested: Healed {heal_amount} HP, now {run.current_hp}/{run.max_hp}")
            else:
                upgradeable = run.get_upgradeable_cards()
                if upgradeable:
                    idx, card = upgradeable[game_rng.misc_rng.random(len(upgradeable) - 1)]
                    run.upgrade_card(idx)
                    print(f"Upgraded: {card.id}")
                else:
                    heal_amount = int(run.max_hp * 0.3)
                    run.heal(heal_amount)
                    print(f"Rested (no upgrades): Healed {heal_amount}")

        elif room_type == RoomType.SHOP:
            print("Visited shop (no purchases simulated)")

        elif room_type == RoomType.EVENT:
            print("Event room (no events implemented)")

        elif room_type == RoomType.TREASURE:
            print("Treasure room (simplified)")
            # Would give a relic from chest

        elif room_type == RoomType.BOSS:
            boss = get_boss_for_act(run.act, seed)
            print(f"Boss: {boss}")
            damage_taken = game_rng.misc_rng.random_range(30, 60)
            run.damage(damage_taken)
            print(f"Boss defeated! Took {damage_taken} damage, HP: {run.current_hp}/{run.max_hp}")

            boss_relics = generate_boss_relics(game_rng.relic_rng, reward_state, "WATCHER", run.act)
            if boss_relics:
                picked = boss_relics[game_rng.misc_rng.random(len(boss_relics) - 1)]
                run.add_relic(picked.id)
                print(f"Boss relic: {picked.name}")

            # Advance act
            print(f"\n=== Completed Act {run.act} ===\n")
            run.advance_act()

            if run.act > 3:
                print("=== RUN COMPLETE ===")
                break

        print()

        # Check death
        if run.current_hp <= 0:
            print("=== DEFEAT ===")
            print(f"Died on floor {floor_num}")
            return 1

    print("=== Run Summary ===")
    print(f"Final state: {run}")
    print(f"Deck ({len(run.deck)} cards):")
    for card in run.deck:
        print(f"  {card}")
    print(f"Relics: {', '.join(r.id for r in run.relics)}")

    return 0


def cmd_map(args) -> int:
    """Generate and display a map."""
    seed_string = args.seed.upper()
    seed = seed_to_long(seed_string)
    act = args.act
    ascension = args.ascension

    print(format_seed_info(seed_string, seed))
    print()

    if act == 4:
        # Act 4 has fixed layout
        nodes = generate_act4_map()
        print("Act 4 Map (Fixed Layout):")
        print(map_to_string(nodes))
        return 0

    # Generate map for specified act
    config = MapGeneratorConfig(ascension_level=ascension)
    map_seed = seed + get_map_seed_offset(act)
    map_rng = Random(map_seed)
    generator = MapGenerator(map_rng, config)

    nodes = generator.generate()

    boss = get_boss_for_act(act, seed)
    print(format_map(nodes, act, boss))

    # Room distribution
    print("\nRoom Distribution:")
    counts: Dict[str, int] = {}
    for row in nodes:
        for node in row:
            if node.room_type and node.has_edges():
                key = node.room_type.name
                counts[key] = counts.get(key, 0) + 1

    for room_type, count in sorted(counts.items()):
        symbol = RoomType[room_type].value
        print(f"  [{symbol}] {room_type}: {count}")

    # JSON output option
    if args.json:
        map_data = {
            "seed": seed_string,
            "numeric_seed": seed,
            "act": act,
            "boss": boss,
            "rooms": []
        }
        for y, row in enumerate(nodes):
            for node in row:
                if node.has_edges():
                    map_data["rooms"].append({
                        "x": node.x,
                        "y": y,
                        "type": node.room_type.name if node.room_type else None,
                        "symbol": node.room_type.value if node.room_type else None,
                        "has_emerald_key": node.has_emerald_key,
                        "edges": [(e.dst_x, e.dst_y) for e in node.edges]
                    })
        print("\nJSON:")
        print(json.dumps(map_data, indent=2))

    return 0


def cmd_rewards(args) -> int:
    """Show card rewards for a specific seed and floor."""
    seed_string = args.seed.upper()
    seed = seed_to_long(seed_string)
    floor = args.floor
    room_type = args.room_type
    ascension = args.ascension
    act = args.act

    print(format_seed_info(seed_string, seed))
    print(f"Floor: {floor}, Room Type: {room_type}, Act: {act}")
    print()

    # Initialize RNG streams
    game_rng = GameRNG(seed=seed, floor=floor)
    reward_state = RewardState()
    reward_state.add_relic("PureWater")

    # Advance RNG to simulate reaching this floor
    for _ in range(floor - 1):
        # Each floor consumes some RNG calls
        game_rng.card_rng.random(99)
        game_rng.treasure_rng.random(99)
        game_rng.potion_rng.random(99)

    # Generate rewards
    cards = generate_card_rewards(
        game_rng.card_rng, reward_state, act, "WATCHER",
        ascension, room_type
    )

    gold = generate_gold_reward(game_rng.treasure_rng, room_type, ascension)

    dropped, potion = check_potion_drop(
        game_rng.potion_rng, reward_state, room_type
    )

    print(f"Seed: {seed_string}, Floor {floor}, {room_type.capitalize()} Combat")
    print(format_card_reward(cards, gold, potion, dropped))

    # JSON output
    if args.json:
        reward_data = {
            "seed": seed_string,
            "floor": floor,
            "room_type": room_type,
            "gold": gold,
            "potion": {
                "dropped": dropped,
                "name": potion.name if potion else None,
                "rarity": potion.rarity.name if potion else None
            },
            "cards": [
                {
                    "id": c.id,
                    "name": c.name,
                    "rarity": c.rarity.name,
                    "type": c.card_type.name if hasattr(c, 'card_type') else None,
                    "upgraded": c.upgraded
                }
                for c in cards
            ]
        }
        print("\nJSON:")
        print(json.dumps(reward_data, indent=2))

    return 0


def cmd_encounter(args) -> int:
    """Show enemy encounter for a specific seed and floor."""
    seed_string = args.seed.upper()
    seed = seed_to_long(seed_string)
    floor = args.floor
    ascension = args.ascension
    act = args.act

    print(format_seed_info(seed_string, seed))
    print()

    # Initialize RNG
    game_rng = GameRNG(seed=seed, floor=floor)

    # Generate encounter
    enemies = get_encounter_for_floor(
        floor, act, "monster",
        game_rng.monster_rng, game_rng.ai_rng, game_rng.monster_hp_rng,
        ascension
    )

    if not enemies:
        print(f"No encounter data for floor {floor}")
        # Create a placeholder with JawWorm
        enemy = JawWorm(game_rng.ai_rng, ascension, game_rng.monster_hp_rng)
        enemy.roll_move()
        enemies = [enemy]

    print(format_enemy_encounter(enemies, floor))

    # Show detailed move info
    print("\nDetailed Enemy Info:")
    for enemy in enemies:
        print(f"  {enemy.NAME}:")
        print(f"    HP: {enemy.state.current_hp}/{enemy.state.max_hp}")
        print(f"    Type: {enemy.TYPE.name}")
        move = enemy.state.next_move
        if move:
            print(f"    Next Move: {move.name}")
            print(f"      Intent: {move.intent.value}")
            if move.base_damage > 0:
                total = move.base_damage * move.hits
                print(f"      Damage: {move.base_damage} x {move.hits} = {total}")
            if move.block > 0:
                print(f"      Block: {move.block}")
            if move.effects:
                print(f"      Effects: {move.effects}")

    # JSON output
    if args.json:
        encounter_data = {
            "seed": seed_string,
            "floor": floor,
            "act": act,
            "enemies": [
                {
                    "id": e.ID,
                    "name": e.NAME,
                    "hp": e.state.current_hp,
                    "max_hp": e.state.max_hp,
                    "type": e.TYPE.name,
                    "move": {
                        "id": e.state.next_move.move_id if e.state.next_move else None,
                        "name": e.state.next_move.name if e.state.next_move else None,
                        "intent": e.state.next_move.intent.value if e.state.next_move else None,
                        "damage": e.state.next_move.base_damage if e.state.next_move else 0,
                        "hits": e.state.next_move.hits if e.state.next_move else 1,
                        "block": e.state.next_move.block if e.state.next_move else 0,
                    } if e.state.next_move else None
                }
                for e in enemies
            ]
        }
        print("\nJSON:")
        print(json.dumps(encounter_data, indent=2))

    return 0


def cmd_rng(args) -> int:
    """Display RNG sequence for verification."""
    seed_string = args.seed.upper()
    seed = seed_to_long(seed_string)
    count = args.count

    print(format_seed_info(seed_string, seed))
    print()

    # Create raw XorShift128 to show internal state
    rng = XorShift128(seed)

    print(f"XorShift128 Initial State:")
    print(f"  seed0: {rng.seed0}")
    print(f"  seed1: {rng.seed1}")
    print()

    # Show sequence using Random wrapper (matches game)
    game_rng = Random(seed)

    print(f"First {count} random_int(99) values:")
    for i in range(count):
        val = game_rng.random_int(99)
        print(f"  {i}: {val}")

    print(f"\nRNG counter after {count} calls: {game_rng.counter}")

    # Additional sequences for verification
    print("\n--- Additional RNG Streams ---")

    # Card RNG
    card_rng = Random(seed)
    print(f"\nCard RNG first 5 random(99):")
    for i in range(5):
        val = card_rng.random(99)
        print(f"  {i}: {val}")

    # Float RNG
    float_rng = Random(seed)
    print(f"\nFloat RNG first 5 random_float():")
    for i in range(5):
        val = float_rng.random_float()
        print(f"  {i}: {val:.6f}")

    # Boolean RNG
    bool_rng = Random(seed)
    print(f"\nBoolean RNG first 5 random_boolean():")
    for i in range(5):
        val = bool_rng.random_boolean()
        print(f"  {i}: {val}")

    # JSON output
    if args.json:
        verify_rng = Random(seed)
        rng_data = {
            "seed": seed_string,
            "numeric_seed": seed,
            "xorshift_state": {
                "seed0": rng.seed0,
                "seed1": rng.seed1
            },
            "random_int_99": [Random(seed).random_int(99) for _ in range(count)]
        }
        # Regenerate clean sequence
        clean_rng = Random(seed)
        rng_data["random_int_99"] = []
        for _ in range(count):
            rng_data["random_int_99"].append(clean_rng.random_int(99))

        print("\nJSON:")
        print(json.dumps(rng_data, indent=2))

    return 0


def cmd_interactive(args) -> int:
    """Interactive mode for manual testing."""
    seed_string = args.seed.upper()
    seed = seed_to_long(seed_string)
    ascension = args.ascension

    print("=" * 60)
    print("Slay the Spire RL - Interactive Mode")
    print("=" * 60)
    print(format_seed_info(seed_string, seed))
    print(f"Ascension: {ascension}")
    print()

    # Create run
    run = create_watcher_run(seed_string, ascension)
    game_rng = GameRNG(seed=seed)
    reward_state = RewardState()
    reward_state.add_relic("PureWater")

    print("Commands:")
    print("  status     - Show current run status")
    print("  deck       - Show current deck")
    print("  relics     - Show current relics")
    print("  map        - Show current act map")
    print("  paths      - Show available paths")
    print("  move <idx> - Move to path index")
    print("  advance    - Advance to next floor (random path)")
    print("  rng <n>    - Show next n RNG values")
    print("  help       - Show this help")
    print("  quit       - Exit interactive mode")
    print()

    while True:
        try:
            cmd_input = input(f"[Floor {run.floor}] > ").strip().lower()
        except (EOFError, KeyboardInterrupt):
            print("\nExiting...")
            break

        if not cmd_input:
            continue

        parts = cmd_input.split()
        cmd = parts[0]
        cmd_args = parts[1:] if len(parts) > 1 else []

        if cmd == "quit" or cmd == "exit" or cmd == "q":
            print("Exiting interactive mode.")
            break

        elif cmd == "help" or cmd == "h":
            print("Commands: status, deck, relics, map, paths, move <idx>, advance, rng <n>, help, quit")

        elif cmd == "status":
            print(f"Run: {run}")
            print(f"Position: {run.map_position}")
            print(f"HP: {run.current_hp}/{run.max_hp}")
            print(f"Gold: {run.gold}")
            print(f"Deck size: {len(run.deck)}")
            print(f"Relics: {len(run.relics)}")

        elif cmd == "deck":
            print(f"Deck ({len(run.deck)} cards):")
            for i, card in enumerate(run.deck):
                print(f"  {i}: {card}")

        elif cmd == "relics":
            print(f"Relics ({len(run.relics)}):")
            for relic in run.relics:
                print(f"  {relic}")

        elif cmd == "map":
            current_map = run.get_current_map()
            if current_map:
                boss = get_boss_for_act(run.act, seed)
                print(format_map(current_map, run.act, boss))
            else:
                print("No map available")

        elif cmd == "paths":
            paths = run.get_available_paths()
            if paths:
                print(f"Available paths ({len(paths)}):")
                for i, node in enumerate(paths):
                    room_str = node.room_type.name if node.room_type else "Unknown"
                    print(f"  {i}: Column {node.x}, Floor {node.y} - {room_str}")
            else:
                print("No available paths")

        elif cmd == "move":
            if not cmd_args:
                print("Usage: move <index>")
                continue
            try:
                idx = int(cmd_args[0])
                paths = run.get_available_paths()
                if 0 <= idx < len(paths):
                    node = paths[idx]
                    run.move_to(node.x, node.y)
                    run.advance_floor()
                    game_rng.advance_floor()
                    room_str = node.room_type.name if node.room_type else "Unknown"
                    print(f"Moved to Floor {run.floor}, Column {node.x} - {room_str}")
                else:
                    print(f"Invalid index. Use 0-{len(paths)-1}")
            except ValueError:
                print("Invalid index")

        elif cmd == "advance":
            paths = run.get_available_paths()
            if paths:
                idx = game_rng.misc_rng.random(len(paths) - 1)
                node = paths[idx]
                run.move_to(node.x, node.y)
                run.advance_floor()
                game_rng.advance_floor()
                room_str = node.room_type.name if node.room_type else "Unknown"
                print(f"Advanced to Floor {run.floor}, Column {node.x} - {room_str}")
            else:
                print("No available paths")

        elif cmd == "rng":
            n = 10
            if cmd_args:
                try:
                    n = int(cmd_args[0])
                except ValueError:
                    pass
            test_rng = game_rng.misc_rng.copy()
            print(f"Next {n} misc_rng values:")
            for i in range(n):
                val = test_rng.random(99)
                print(f"  {i}: {val}")

        else:
            print(f"Unknown command: {cmd}")
            print("Type 'help' for available commands")

    return 0


# =============================================================================
# MAIN
# =============================================================================

def main():
    parser = argparse.ArgumentParser(
        description="Slay the Spire RL - CLI for testing and comparison",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  %(prog)s run --seed ABC123 --ascension 20
  %(prog)s run --seed ABC123 --floors 10
  %(prog)s map --seed ABC123 --act 1
  %(prog)s rewards --seed ABC123 --floor 3
  %(prog)s encounter --seed ABC123 --floor 1
  %(prog)s rng --seed ABC123 --count 20
  %(prog)s interactive --seed ABC123
        """
    )

    subparsers = parser.add_subparsers(dest="command", help="Available commands")

    # Run command
    run_parser = subparsers.add_parser("run", help="Run a game simulation")
    run_parser.add_argument("--seed", "-s", required=True, help="Game seed (e.g., ABC123)")
    run_parser.add_argument("--ascension", "-a", type=int, default=20, help="Ascension level (0-20)")
    run_parser.add_argument("--floors", "-f", type=int, help="Maximum floors to simulate")
    run_parser.add_argument("--json", "-j", action="store_true", help="Output in JSON format")

    # Map command
    map_parser = subparsers.add_parser("map", help="Generate and display a map")
    map_parser.add_argument("--seed", "-s", required=True, help="Game seed")
    map_parser.add_argument("--act", type=int, default=1, choices=[1, 2, 3, 4], help="Act number")
    map_parser.add_argument("--ascension", "-a", type=int, default=20, help="Ascension level")
    map_parser.add_argument("--json", "-j", action="store_true", help="Output in JSON format")

    # Rewards command
    rewards_parser = subparsers.add_parser("rewards", help="Show card rewards for a seed/floor")
    rewards_parser.add_argument("--seed", "-s", required=True, help="Game seed")
    rewards_parser.add_argument("--floor", type=int, default=1, help="Floor number")
    rewards_parser.add_argument("--room-type", type=str, default="normal",
                               choices=["normal", "elite", "boss"], help="Room type")
    rewards_parser.add_argument("--act", type=int, default=1, help="Act number")
    rewards_parser.add_argument("--ascension", "-a", type=int, default=20, help="Ascension level")
    rewards_parser.add_argument("--json", "-j", action="store_true", help="Output in JSON format")

    # Encounter command
    encounter_parser = subparsers.add_parser("encounter", help="Show enemy encounter for a seed/floor")
    encounter_parser.add_argument("--seed", "-s", required=True, help="Game seed")
    encounter_parser.add_argument("--floor", type=int, default=1, help="Floor number")
    encounter_parser.add_argument("--act", type=int, default=1, help="Act number")
    encounter_parser.add_argument("--ascension", "-a", type=int, default=20, help="Ascension level")
    encounter_parser.add_argument("--json", "-j", action="store_true", help="Output in JSON format")

    # RNG command
    rng_parser = subparsers.add_parser("rng", help="Verify RNG sequence")
    rng_parser.add_argument("--seed", "-s", required=True, help="Game seed")
    rng_parser.add_argument("--count", "-n", type=int, default=20, help="Number of values to show")
    rng_parser.add_argument("--json", "-j", action="store_true", help="Output in JSON format")

    # Interactive command
    interactive_parser = subparsers.add_parser("interactive", help="Interactive testing mode")
    interactive_parser.add_argument("--seed", "-s", required=True, help="Game seed")
    interactive_parser.add_argument("--ascension", "-a", type=int, default=20, help="Ascension level")

    args = parser.parse_args()

    if not args.command:
        parser.print_help()
        return 1

    # Dispatch to command handler
    commands = {
        "run": cmd_run,
        "map": cmd_map,
        "rewards": cmd_rewards,
        "encounter": cmd_encounter,
        "rng": cmd_rng,
        "interactive": cmd_interactive,
    }

    handler = commands.get(args.command)
    if handler:
        return handler(args)
    else:
        parser.print_help()
        return 1


if __name__ == "__main__":
    sys.exit(main())
