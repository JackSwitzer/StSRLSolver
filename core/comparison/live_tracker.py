#!/usr/bin/env python3
"""
Live Game Tracker - Read save file and predict upcoming rewards.

Usage:
    python live_tracker.py          # Auto-read save, show current state + predictions
    python live_tracker.py --watch  # Continuously watch for save changes
"""

import sys
import os
import json
import base64
import time
from pathlib import Path

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.state.game_rng import GameRNGState, RNGStream
from core.state.rng import long_to_seed
from core.generation.rewards import generate_card_rewards, RewardState


# XOR key for save file decryption
OBFUSCATION_KEY = "key"

# Default save path
SAVE_PATH = os.path.expanduser(
    "~/Library/Application Support/Steam/steamapps/common/SlayTheSpire/"
    "SlayTheSpire.app/Contents/Resources/saves/WATCHER.autosave"
)


def xor_with_key(data: bytes, key: str) -> bytes:
    """XOR data with repeating key."""
    key_bytes = key.encode('utf-8')
    result = bytearray(len(data))
    for i, byte in enumerate(data):
        result[i] = byte ^ key_bytes[i % len(key_bytes)]
    return bytes(result)


def read_save_file(path: str = SAVE_PATH) -> dict:
    """Read and decode save file."""
    with open(path, 'r') as f:
        raw = f.read()

    if '{' in raw:
        return json.loads(raw)

    decoded = base64.b64decode(raw)
    decrypted = xor_with_key(decoded, OBFUSCATION_KEY)
    return json.loads(decrypted.decode('utf-8'))


def simulate_path(seed_long: int, neow: str, path: list, current_floor: int) -> GameRNGState:
    """Simulate RNG state up to current floor."""
    seed_str = long_to_seed(seed_long)
    state = GameRNGState(seed_str)

    # Apply Neow
    state.apply_neow_choice(neow)

    # Simulate each floor
    for floor, room_type in enumerate(path, 1):
        if floor >= current_floor:
            break  # Stop before current floor (combat not complete)

        if room_type == 'M':
            state.apply_combat('monster')
        elif room_type == 'E':
            state.apply_combat('elite')
        elif room_type == '$':
            state.apply_shop()
        elif room_type == '?':
            state.apply_event()
        elif room_type == 'T':
            state.apply_treasure()
        # R (rest) doesn't consume cardRng

    return state


def predict_next_reward(state: GameRNGState, room_type: str, act: int = 1, card_blizzard: int = 5) -> list:
    """Predict card reward for next combat."""
    reward_state = RewardState()
    reward_state.card_blizzard.offset = card_blizzard  # Use actual pity timer from save
    card_rng = state.get_rng(RNGStream.CARD)

    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=reward_state,
        act=act,
        player_class='WATCHER',
        ascension=20,
        room_type='elite' if room_type == 'E' else 'normal',
        num_cards=3,
    )

    return [c.name for c in cards]


def display_state(save_data: dict):
    """Display current game state and predictions."""
    seed_long = save_data.get('seed', 0)
    seed_str = long_to_seed(seed_long)
    floor = save_data.get('floor_num', 0)
    act = save_data.get('act_num', 1)
    neow = save_data.get('neow_bonus', 'NONE')
    path = save_data.get('metric_path_per_floor', [])
    card_counter = save_data.get('card_seed_count', 0)
    current_room = save_data.get('current_room', '')
    hp = save_data.get('current_health', 0)
    max_hp = save_data.get('max_health', 0)
    gold = save_data.get('gold', 0)
    relics = save_data.get('relics', [])

    # Determine current room type
    if 'MonsterRoom' in current_room:
        if 'Elite' in current_room:
            room_type = 'E'
        elif 'Boss' in current_room:
            room_type = 'BOSS'
        else:
            room_type = 'M'
    elif 'Shop' in current_room:
        room_type = '$'
    elif 'Rest' in current_room:
        room_type = 'R'
    elif 'Treasure' in current_room:
        room_type = 'T'
    elif 'Event' in current_room:
        room_type = '?'
    else:
        room_type = '?'

    print("\n" + "="*70)
    print("LIVE GAME TRACKER")
    print("="*70)
    print(f"Seed: {seed_str} ({seed_long})")
    print(f"Floor: {floor} | Act: {act} | Room: {room_type}")
    print(f"Neow: {neow}")
    print(f"HP: {hp}/{max_hp} | Gold: {gold}")
    print(f"Relics: {', '.join(relics)}")
    print(f"Path: {' '.join(path)}")
    print()

    # Simulate state
    state = simulate_path(seed_long, neow, path, floor)
    sim_counter = state.get_counter(RNGStream.CARD)

    print(f"cardRng Counter:")
    print(f"  Game:      {card_counter}")
    print(f"  Simulated: {sim_counter}")

    if sim_counter == card_counter:
        print(f"  Status:    SYNCHRONIZED")
    else:
        diff = sim_counter - card_counter
        print(f"  Status:    OFF BY {diff}")

    # Predict next combat reward
    if room_type in ['M', 'E', 'BOSS']:
        print()
        print("-"*70)
        print(f"PREDICTION for Floor {floor} {room_type} combat:")
        card_blizzard = save_data.get('card_random_seed_randomizer', 5)
        cards = predict_next_reward(state, room_type, act, card_blizzard)
        print(f"  Card Reward: {cards}")
        print(f"  Blizzard (pity): {card_blizzard}")
        print("-"*70)

    # Card choices from save
    card_choices = save_data.get('metric_card_choices', [])
    if card_choices:
        print()
        print("Recent Card Choices:")
        for choice in card_choices[-3:]:
            floor_num = choice.get('floor', '?')
            picked = choice.get('picked', 'SKIP')
            not_picked = choice.get('not_picked', [])
            all_cards = not_picked + ([picked] if picked != 'SKIP' else [])
            print(f"  F{floor_num}: {all_cards} -> {picked}")


def watch_mode():
    """Continuously watch for save file changes."""
    print("Watching for save file changes... (Ctrl+C to stop)")
    last_mtime = 0

    while True:
        try:
            mtime = os.path.getmtime(SAVE_PATH)
            if mtime != last_mtime:
                last_mtime = mtime
                os.system('clear')
                save_data = read_save_file()
                display_state(save_data)
            time.sleep(0.5)
        except FileNotFoundError:
            print("Save file not found. Start a game...")
            time.sleep(2)
        except KeyboardInterrupt:
            print("\nStopped.")
            break


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Live game tracker")
    parser.add_argument("--watch", action="store_true", help="Watch mode")
    args = parser.parse_args()

    if args.watch:
        watch_mode()
    else:
        try:
            save_data = read_save_file()
            display_state(save_data)
        except FileNotFoundError:
            print(f"Save file not found: {SAVE_PATH}")
