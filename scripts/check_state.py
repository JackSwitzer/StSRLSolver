#!/usr/bin/env python3
"""
Check current game state and show predictions.

Usage:
    uv run scripts/check_state.py           # One-time check
    uv run scripts/check_state.py --watch   # Continuous monitoring
    uv run scripts/check_state.py --watch --interval 5  # Check every 5 seconds
"""

import argparse
import os
import sys
import time

PROJECT_ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
os.chdir(PROJECT_ROOT)
sys.path.insert(0, PROJECT_ROOT)

from core.comparison.full_rng_tracker import (
    read_save_file, predict_boss_relics, SAVE_PATH, CLASS_STARTER_RELICS
)
from core.state.rng import long_to_seed, Random
from core.state.game_rng import GameRNGState, RNGStream
from core.generation.map import MapGenerator, MapGeneratorConfig, get_map_seed_offset
from core.generation.rewards import generate_card_rewards, RewardState


ENEMY_HP = {
    'Cultist': '50-56', 'Jaw Worm': '42-46', 'Large Slime': '65-70',
    'Blue Slaver': '48-52', 'Red Slaver': '48-52', 'Small Slimes': '2x(13-17)',
    '3 Louse': '3x(11-17)', '2 Fungi Beasts': '2x(24-28)',
    'Exordium Wildlife': 'Looter+', 'Exordium Thugs': 'Looter+Slaver',
    'Gremlin Gang': '4-5 Gremlins', 'Looter': '46-50',
    'Gremlin Nob': '82-86', 'Lagavulin': '112-116', '3 Sentries': '3x(39-45)',
    'The Guardian': '240', 'Hexaghost': '264', 'Slime Boss': '150',
    # Act 2
    'Chosen': '96-99', 'Byrd': '28-32', 'Spheric Guardian': '20',
    'Snake Plant': '60-64', 'Snecko': '60-64', 'Centurion and Mystic': 'C:76-80, M:56-60',
    'Cultist and Chosen': 'varies', 'Shelled Parasite': '68-72', 'Snecko and Shelled': 'varies',
    'Book of Stabbing': '168', 'Gremlin Leader': '162', 'Slavers': 'varies',
    'The Champ': '440', 'Automaton': '320', 'Collector': '300',
}


def predict_reward(seed_str: str, card_counter: int, blizzard: int, act: int, room_type: str):
    """Predict card reward."""
    state = GameRNGState(seed_str)
    state.set_counter(RNGStream.CARD, card_counter)
    reward_state = RewardState()
    reward_state.card_blizzard.offset = blizzard
    card_rng = state.get_rng(RNGStream.CARD)
    cards = generate_card_rewards(
        rng=card_rng, reward_state=reward_state, act=act,
        player_class='WATCHER', ascension=20,
        room_type='elite' if room_type == 'E' else 'normal', num_cards=3,
    )
    return [c.name for c in cards]


def display_state(last_floor: int = -1) -> int:
    """Display current state. Returns current floor."""
    try:
        save = read_save_file()
    except FileNotFoundError:
        print("No save file found. Start a game!")
        return -1

    floor = save.get('floor_num', 0)
    if floor == last_floor:
        return floor

    os.system('clear')

    seed_long = save.get('seed', 0)
    seed_str = long_to_seed(seed_long)
    act = save.get('act_num', 1)
    room_x = save.get('room_x', 0)
    room_y = save.get('room_y', 0)

    # Header
    print("=" * 70)
    print(f"SEED: {seed_str} | Floor {floor} | Act {act}")
    print("=" * 70)

    # Stats
    hp = save.get('current_health', 0)
    max_hp = save.get('max_health', 0)
    gold = save.get('gold', 0)
    print(f"HP: {hp}/{max_hp} | Gold: {gold}")

    # RNG
    card_rng = save.get('card_seed_count', 0)
    blizzard = save.get('card_random_seed_randomizer', 5)
    print(f"cardRng: {card_rng} | blizzard: {blizzard}")
    print()

    # Relics
    relics = save.get('relics', [])
    print(f"Relics: {', '.join(relics)}")

    # Path
    path = save.get('metric_path_per_floor', [])
    path_clean = [p for p in path if p]  # Remove None values
    print(f"Path: {' '.join(path_clean)}")
    print()

    # Generate map
    map_seed = seed_long + get_map_seed_offset(act)
    config = MapGeneratorConfig(ascension_level=20)
    rng = Random(map_seed)
    gen = MapGenerator(rng, config)
    dungeon_map = gen.generate()

    node_lookup = {}
    for row in dungeon_map:
        for node in row:
            if node:
                node_lookup[(node.x, node.y)] = node

    current_node = node_lookup.get((room_x, room_y))

    # Pre-generated lists
    monster_list = save.get('monster_list', [])
    event_list = save.get('event_list', [])
    elite_list = save.get('elite_monster_list', [])

    # Count consumption
    monsters_seen = sum(1 for p in path_clean if p == 'M')
    events_seen = sum(1 for p in path_clean if p == '?')
    elites_seen = sum(1 for p in path_clean if p == 'E')

    # Show next nodes
    print("-" * 70)
    print(f"CURRENT: ({room_x},{room_y}) | NEXT OPTIONS:")
    print("-" * 70)

    if current_node and current_node.edges:
        for edge in sorted(current_node.edges, key=lambda e: e.dst_x):
            next_node = node_lookup.get((edge.dst_x, edge.dst_y))
            if next_node:
                sym = next_node.room_type.value if next_node.room_type else '?'
                print()
                print(f">>> ({edge.dst_x},{edge.dst_y}) {sym}", end="")

                if sym == 'M':
                    monster = monster_list[monsters_seen] if monsters_seen < len(monster_list) else '???'
                    hp_info = ENEMY_HP.get(monster, '???')
                    reward = predict_reward(seed_str, card_rng, blizzard, act, 'M')
                    print(f" - {monster} ({hp_info})")
                    print(f"    Cards: {reward}")
                elif sym == 'E':
                    elite = elite_list[elites_seen] if elites_seen < len(elite_list) else '???'
                    hp_info = ENEMY_HP.get(elite, '???')
                    reward = predict_reward(seed_str, card_rng, blizzard, act, 'E')
                    print(f" - ELITE: {elite} ({hp_info})")
                    print(f"    Cards: {reward}")
                elif sym == '?':
                    event = event_list[events_seen] if events_seen < len(event_list) else '???'
                    print(f" - EVENT: {event}")
                elif sym == '$':
                    print(" - SHOP")
                elif sym == 'R':
                    print(" - REST (heal 30% / upgrade)")
                elif sym == 'T':
                    print(" - TREASURE")
                else:
                    print()
    else:
        # At boss or no edges
        boss = save.get('boss', 'Unknown')
        if floor >= 15:
            print(f"\n>>> BOSS: {boss}")
            hp_info = ENEMY_HP.get(boss, '???')
            print(f"    HP: {hp_info}")

            has_starter = any(r in relics for r in CLASS_STARTER_RELICS.values())
            boss_picked = len(save.get('metric_boss_relics', []))
            boss_pred = predict_boss_relics(seed_long, 'WATCHER', has_starter, boss_picked, relics)
            print(f"    Boss Relics: {boss_pred}")

    print()
    return floor


def main():
    parser = argparse.ArgumentParser(description="Check game state with predictions")
    parser.add_argument("--watch", action="store_true", help="Continuous monitoring")
    parser.add_argument("--interval", type=int, default=15, help="Check interval in seconds")
    args = parser.parse_args()

    if args.watch:
        print(f"Watching for changes every {args.interval}s... (Ctrl+C to stop)")
        last_floor = -1
        while True:
            try:
                last_floor = display_state(last_floor)
                time.sleep(args.interval)
            except KeyboardInterrupt:
                print("\nStopped.")
                break
    else:
        display_state()


if __name__ == "__main__":
    main()
