#!/usr/bin/env python3
"""
Live Game Monitor - Continuous tracking with predictions.

Monitors save file every 15 seconds and shows:
- Current state and RNG streams
- All accessible next nodes with predictions
- Card rewards, events, shop contents, etc.
"""

import sys
import os
import time
from typing import List, Dict, Optional, Tuple

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.comparison.full_rng_tracker import (
    read_save_file, SAVE_PATH, predict_boss_relics,
    CLASS_STARTER_RELICS, STARTER_UPGRADE_RELICS
)
from core.state.rng import long_to_seed, Random
from core.state.game_rng import GameRNGState, RNGStream
from core.generation.map import MapGenerator, MapGeneratorConfig, get_map_seed_offset
from core.generation.rewards import generate_card_rewards, RewardState


# Enemy HP data (A20)
ENEMY_HP = {
    'Cultist': '50-56',
    'Jaw Worm': '42-46',
    'Small Slimes': '2x Slime (13-17 each)',
    'Large Slime': 'Acid/Spike Slime (L): 65-70',
    'Blue Slaver': '48-52',
    'Red Slaver': '48-52',
    '3 Louse': '3x Louse (11-17 each)',
    '2 Fungi Beasts': '2x Fungi Beast (24-28 each)',
    'Exordium Wildlife': 'Looter + others',
    'Exordium Thugs': 'Looter + Slaver',
    'Gremlin Gang': '4-5 Gremlins (varies)',
    'Looter': '46-50',
    'Gremlin Nob': '82-86',
    'Lagavulin': '112-116 (asleep)',
    '3 Sentries': '3x Sentry (39-45 each)',
    'The Guardian': '240 (250 A9+)',
    'Hexaghost': '264 (A9+)',
    'Slime Boss': '150 (splits at 50%)',
}

# Event options
EVENT_OPTIONS = {
    'Big Fish': ['Heal 5', 'MaxHP+5 (Donut)', 'Relic+Curse'],
    'The Cleric': ['Heal 15 (35g)', 'Remove (50g)', 'Leave'],
    'Dead Adventurer': ['Search (elite or gold)', 'Leave'],
    'Golden Idol': ['Take (curse+300g)', 'Leave'],
    'Golden Wing': ['Heal 6', 'Heal 30% (lose gold, get relic)'],
    'World of Goop': ['Lose 11 HP for 75g', 'Leave'],
    'Living Wall': ['Remove', 'Transform', 'Upgrade'],
    'Mushrooms': ['Stomp (fight)', 'Eat (heal 50%)'],
    'Scrap Ooze': ['Reach (relic or curse+dmg)', 'Leave'],
    'Shining Light': ['Enter (upgrade 2, lose 20% HP)', 'Leave'],
    'Liars Game': ['Agree (gold gamble)', 'Disagree'],
}


def get_map_and_path(seed_long: int, act: int = 1) -> Tuple[Dict, List]:
    """Generate map and build node lookup."""
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

    return node_lookup, dungeon_map


def predict_combat_reward(
    seed_str: str,
    card_counter: int,
    act: int,
    room_type: str,
    blizzard: int = 5,
    player_class: str = 'WATCHER'
) -> List[str]:
    """Predict card reward for a combat."""
    game_state = GameRNGState(seed_str)
    game_state.set_counter(RNGStream.CARD, card_counter)

    reward_state = RewardState()
    reward_state.card_blizzard.offset = blizzard
    card_rng = game_state.get_rng(RNGStream.CARD)

    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=reward_state,
        act=act,
        player_class=player_class,
        ascension=20,
        room_type='elite' if room_type == 'E' else 'normal',
        num_cards=3,
    )
    return [c.name for c in cards]


def display_state(save_data: dict, last_floor: int) -> int:
    """Display current state and predictions. Returns current floor."""
    seed_long = save_data.get('seed', 0)
    seed_str = long_to_seed(seed_long)
    floor = save_data.get('floor_num', 0)
    act = save_data.get('act_num', 1)

    # Check if floor changed
    if floor == last_floor:
        return floor

    # Clear screen for new floor
    os.system('clear')

    # Basic info
    print("=" * 70)
    print(f"LIVE MONITOR | Seed: {seed_str} | Floor {floor} | Act {act}")
    print("=" * 70)

    # HP and resources
    hp = save_data.get('current_health', 0)
    max_hp = save_data.get('max_health', 0)
    gold = save_data.get('gold', 0)
    print(f"HP: {hp}/{max_hp} | Gold: {gold}")

    # RNG Streams
    print()
    print("-" * 70)
    print("RNG STREAMS")
    print("-" * 70)
    card_rng = save_data.get('card_seed_count', 0)
    blizzard = save_data.get('card_random_seed_randomizer', 5)
    relic_rng = save_data.get('relic_seed_count', 0)
    potion_rng = save_data.get('potion_seed_count', 0)
    monster_rng = save_data.get('monster_seed_count', 0)
    event_rng = save_data.get('event_seed_count', 0)

    print(f"  cardRng: {card_rng}  |  blizzard: {blizzard}  |  relicRng: {relic_rng}")
    print(f"  potionRng: {potion_rng}  |  monsterRng: {monster_rng}  |  eventRng: {event_rng}")

    # Current position
    room_x = save_data.get('room_x', 0)
    room_y = save_data.get('room_y', 0)
    current_room = save_data.get('current_room', '')

    # Relics
    relics = save_data.get('relics', [])
    print()
    print(f"Relics: {', '.join(relics)}")

    # Deck summary
    cards = save_data.get('cards', [])
    deck_size = len(cards)
    print(f"Deck: {deck_size} cards")

    # Pre-generated lists
    monster_list = save_data.get('monster_list', [])
    event_list = save_data.get('event_list', [])
    elite_list = save_data.get('elite_monster_list', [])

    # Get map
    node_lookup, dungeon_map = get_map_and_path(seed_long, act)

    # Find current node and its children
    current_node = node_lookup.get((room_x, room_y))

    print()
    print("-" * 70)
    print(f"CURRENT NODE: ({room_x}, {room_y})")
    print("-" * 70)

    if current_node:
        sym = current_node.room_type.value if current_node.room_type else '?'
        print(f"  Type: {sym}")

        # If combat, show prediction
        if sym in ['M', 'E']:
            reward = predict_combat_reward(
                seed_str, card_rng, act, sym, blizzard, 'WATCHER'
            )
            print(f"  Card Reward: {reward}")

    # Show next accessible nodes
    print()
    print("-" * 70)
    print("NEXT NODES (accessible from current position)")
    print("-" * 70)

    if current_node and current_node.edges:
        # Track monster/event consumption for predictions
        path_taken = save_data.get('metric_path_per_floor', [])
        monsters_seen = sum(1 for p in path_taken if p == 'M')
        elites_seen = sum(1 for p in path_taken if p == 'E')
        events_seen = sum(1 for p in path_taken if p == '?')

        for edge in sorted(current_node.edges, key=lambda e: e.dst_x):
            next_node = node_lookup.get((edge.dst_x, edge.dst_y))
            if next_node:
                sym = next_node.room_type.value if next_node.room_type else '?'
                print()
                print(f"  >>> ({edge.dst_x}, {edge.dst_y}) - {sym}")

                if sym == 'M':
                    # Monster combat
                    if monsters_seen < len(monster_list):
                        monster = monster_list[monsters_seen]
                    else:
                        monster = "???"
                    hp_info = ENEMY_HP.get(monster, 'HP varies')

                    # Predict reward (after this combat)
                    reward = predict_combat_reward(
                        seed_str, card_rng, act, 'M', blizzard, 'WATCHER'
                    )

                    print(f"      Monster: {monster} ({hp_info})")
                    print(f"      Card Reward: {reward}")
                    print(f"      Gold: 10-20 base")

                elif sym == 'E':
                    # Elite combat
                    if elites_seen < len(elite_list):
                        elite = elite_list[elites_seen]
                    else:
                        elite = "???"
                    hp_info = ENEMY_HP.get(elite, 'HP varies')

                    reward = predict_combat_reward(
                        seed_str, card_rng, act, 'E', blizzard, 'WATCHER'
                    )

                    print(f"      Elite: {elite} ({hp_info})")
                    print(f"      Card Reward: {reward}")
                    print(f"      Relic: guaranteed")

                elif sym == '?':
                    # Event
                    if events_seen < len(event_list):
                        event = event_list[events_seen]
                    else:
                        event = "???"

                    options = EVENT_OPTIONS.get(event, ['Options vary'])
                    print(f"      Event: {event}")
                    for opt in options:
                        print(f"        - {opt}")

                elif sym == '$':
                    print(f"      Shop")
                    print(f"        5 colored cards + 2 colorless")
                    print(f"        3 relics, 3 potions")
                    print(f"        Remove: 75g")

                elif sym == 'R':
                    print(f"      Campfire")
                    print(f"        - Rest: Heal 30%")
                    print(f"        - Smith: Upgrade")

                elif sym == 'T':
                    print(f"      Treasure")
                    print(f"        Relic (common tier)")
    else:
        # At boss or no edges
        boss = save_data.get('boss', 'Unknown Boss')
        if floor >= 15:
            print(f"  >>> BOSS: {boss}")
            hp_info = ENEMY_HP.get(boss, 'HP varies')
            print(f"      HP: {hp_info}")

            # Boss relic prediction
            has_starter = any(r in relics for r in CLASS_STARTER_RELICS.values())
            boss_relics_picked = save_data.get('metric_boss_relics', [])

            predicted = predict_boss_relics(
                seed_long,
                player_class="WATCHER",
                has_starter_relic=has_starter,
                relics_already_taken=len(boss_relics_picked),
                already_owned_relics=relics,
            )
            print(f"      Boss Relics: {predicted}")

    print()
    print("-" * 70)
    print(f"Watching... (updates on floor change)")
    print("-" * 70)

    return floor


def monitor():
    """Main monitoring loop."""
    print("Starting live monitor... (Ctrl+C to stop)")
    last_floor = -1
    last_mtime = 0

    while True:
        try:
            if os.path.exists(SAVE_PATH):
                mtime = os.path.getmtime(SAVE_PATH)
                if mtime != last_mtime:
                    last_mtime = mtime
                    save_data = read_save_file()
                    last_floor = display_state(save_data, last_floor)
            else:
                os.system('clear')
                print("Waiting for save file... Start a game!")
                last_floor = -1

            time.sleep(15)

        except KeyboardInterrupt:
            print("\nStopped.")
            break
        except Exception as e:
            print(f"Error: {e}")
            time.sleep(5)


if __name__ == "__main__":
    monitor()
