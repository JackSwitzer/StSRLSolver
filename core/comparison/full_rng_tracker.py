#!/usr/bin/env python3
"""
Full RNG Tracker - 100% Accuracy Goal

Tracks ALL RNG streams and predicts everything:
- Card rewards (cardRng)
- Boss relic offerings (relicRng)
- Enemy encounters (monsterRng)
- Events (eventRng)
- Potions (potionRng)
- Shop contents (merchantRng, cardRng)

Usage:
    python full_rng_tracker.py          # Full state report
    python full_rng_tracker.py --watch  # Auto-refresh
    python full_rng_tracker.py --debug  # Verbose RNG tracing
"""

import sys
import os
import json
import base64
import time
from pathlib import Path
from typing import List, Dict, Tuple, Optional
from dataclasses import dataclass

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from core.state.game_rng import GameRNGState, RNGStream
from core.state.rng import Random, long_to_seed, seed_to_long
from core.generation.rewards import generate_card_rewards, RewardState


# =============================================================================
# CONSTANTS
# =============================================================================

SAVE_PATH = os.path.expanduser(
    "~/Library/Application Support/Steam/steamapps/common/SlayTheSpire/"
    "SlayTheSpire.app/Contents/Resources/saves/WATCHER.autosave"
)

OBFUSCATION_KEY = "key"

# Starter-upgrade boss relics: these require the corresponding starter relic to spawn
# The canSpawn() method checks player.hasRelic(starter_id)
STARTER_UPGRADE_RELICS = {
    # Relic ID: Required starter relic ID
    "Black Blood": "Burning Blood",      # Ironclad
    "Ring of the Serpent": "Ring of the Snake",  # Silent (note: starter is "Ring of the Snake" not "SnakeRing")
    "FrozenCore": "Cracked Core",         # Defect
    "HolyWater": "PureWater",             # Watcher
}

# Starter relics by class (used to determine which upgrade relic can spawn)
CLASS_STARTER_RELICS = {
    "IRONCLAD": "Burning Blood",
    "SILENT": "Ring of the Snake",
    "DEFECT": "Cracked Core",
    "WATCHER": "PureWater",
}

# Relics that consume cardRng on pickup (and how much)
BOSS_RELIC_CARDRNG_CONSUMPTION = {
    # Most boss relics consume 0 cardRng
    # Astrolabe uses miscRng for transforms
    # Pandora's Box uses cardRandomRng (different stream)
    # Calling Bell uses relicRng
    # Empty Cage uses nothing
    # Tiny House uses miscRng
}

# Room type to symbol mapping
ROOM_SYMBOLS = {
    'MonsterRoom': 'M',
    'MonsterRoomElite': 'E',
    'MonsterRoomBoss': 'B',
    'ShopRoom': '$',
    'RestRoom': 'R',
    'TreasureRoom': 'T',
    'EventRoom': '?',
}


# =============================================================================
# SAVE FILE READING
# =============================================================================

def xor_with_key(data: bytes, key: str) -> bytes:
    key_bytes = key.encode('utf-8')
    result = bytearray(len(data))
    for i, byte in enumerate(data):
        result[i] = byte ^ key_bytes[i % len(key_bytes)]
    return bytes(result)


def read_save_file(path: str = SAVE_PATH) -> dict:
    with open(path, 'r') as f:
        raw = f.read()

    if '{' in raw:
        return json.loads(raw)

    decoded = base64.b64decode(raw)
    decrypted = xor_with_key(decoded, OBFUSCATION_KEY)
    return json.loads(decrypted.decode('utf-8'))


# =============================================================================
# RNG SIMULATION
# =============================================================================

@dataclass
class FullRNGState:
    """Complete RNG state for all streams."""
    seed_str: str
    seed_long: int

    # Counters for all persistent streams
    card_counter: int = 0
    relic_counter: int = 0
    potion_counter: int = 0
    monster_counter: int = 0
    event_counter: int = 0
    merchant_counter: int = 0
    treasure_counter: int = 0

    # Boss relic pool state
    boss_relics_taken: int = 0

    # Tracking
    floor: int = 0
    act: int = 1


def simulate_full_path(save_data: dict) -> FullRNGState:
    """Simulate all RNG consumption from game start to current state."""
    seed_long = save_data.get('seed', 0)
    seed_str = long_to_seed(seed_long)

    state = FullRNGState(
        seed_str=seed_str,
        seed_long=seed_long,
    )

    neow = save_data.get('neow_bonus', 'NONE')
    path = [p for p in save_data.get('metric_path_per_floor', []) if p]
    current_floor = save_data.get('floor_num', 0)

    # Initialize game RNG state
    game_rng = GameRNGState(seed_str)
    game_rng.apply_neow_choice(neow)

    state.card_counter = game_rng.get_counter(RNGStream.CARD)

    # Simulate each floor
    for floor_idx, room in enumerate(path):
        floor_num = floor_idx + 1

        if floor_num >= current_floor:
            break  # Don't simulate current floor (not complete)

        if room == 'M':
            game_rng.apply_combat('monster')
        elif room == 'E':
            game_rng.apply_combat('elite')
        elif room == 'B':
            game_rng.apply_combat('boss')
            # Boss relic taken
            state.boss_relics_taken += 1
        elif room == '$':
            game_rng.apply_shop()
        elif room == '?':
            game_rng.apply_event()
        elif room == 'T':
            game_rng.apply_treasure()
        # R (rest) = no consumption

        # Check for act transitions
        if floor_num == 16:  # End of Act 1
            game_rng.transition_to_next_act()
        elif floor_num == 33:  # End of Act 2
            game_rng.transition_to_next_act()
        elif floor_num == 51:  # End of Act 3
            game_rng.transition_to_next_act()

    state.card_counter = game_rng.get_counter(RNGStream.CARD)
    state.floor = current_floor
    state.act = save_data.get('act_num', 1)

    return state


def predict_boss_relics(
    seed_long: int,
    player_class: str = "WATCHER",
    has_starter_relic: bool = True,
    relics_already_taken: int = 0,
    already_owned_relics: Optional[List[str]] = None,
) -> List[str]:
    """
    Predict the 3 boss relics that will be offered.

    Boss relic pool is shuffled using relicRng.randomLong() during init.
    Relics are taken from the front of the pool, skipping any that fail canSpawn().

    The canSpawn() check happens at selection time (returnRandomRelicKey), not at
    pool initialization. This means:
    - Starter-upgrade relics (Black Blood, Ring of the Serpent, etc.) are in the pool
    - When selected, if player doesn't have the required starter relic, they're skipped
    - The next valid relic in the pool is offered instead

    Args:
        seed_long: The game seed (long value)
        player_class: IRONCLAD, SILENT, DEFECT, or WATCHER
        has_starter_relic: Whether player still has their starter relic
                          (False if Neow swapped it for a boss relic)
        relics_already_taken: Number of boss relic selections already made (0, 1, or 2)
        already_owned_relics: List of relic IDs already owned (these are skipped)

    Returns:
        List of 3 boss relic IDs that will be offered
    """
    from core.generation.relics import predict_boss_relic_pool

    # Get the shuffled boss relic pool using the proper generation code
    # This uses the correct HashMap iteration order and shuffle algorithm
    shuffled_pool = predict_boss_relic_pool(seed_long, player_class)

    if already_owned_relics is None:
        already_owned_relics = []

    # Determine which starter relic the player has (if any)
    player_starter = CLASS_STARTER_RELICS.get(player_class)

    def can_spawn(relic_id: str) -> bool:
        """Check if a relic can spawn (replicates AbstractRelic.canSpawn())"""
        # Check if already owned
        if relic_id in already_owned_relics:
            return False

        # Check starter-upgrade relics
        if relic_id in STARTER_UPGRADE_RELICS:
            required_starter = STARTER_UPGRADE_RELICS[relic_id]
            # Player must have the corresponding starter relic
            if not has_starter_relic:
                return False
            if player_starter != required_starter:
                return False

        return True

    # Simulate the pool consumption
    # Each boss selection takes 3 relics from the front, skipping invalid ones
    pool_index = 0
    selections_made = 0

    while selections_made < relics_already_taken:
        # Take 3 valid relics for this selection
        relics_taken = 0
        while relics_taken < 3 and pool_index < len(shuffled_pool):
            relic = shuffled_pool[pool_index]
            pool_index += 1
            if can_spawn(relic):
                relics_taken += 1
        selections_made += 1

    # Now get the next 3 valid relics for the current offering
    result = []
    while len(result) < 3 and pool_index < len(shuffled_pool):
        relic = shuffled_pool[pool_index]
        pool_index += 1
        if can_spawn(relic):
            result.append(relic)

    # If pool exhausted, fill with Red Circlet
    while len(result) < 3:
        result.append("Red Circlet")

    return result


def predict_card_reward(
    seed_str: str,
    card_counter: int,
    act: int,
    room_type: str = 'normal',
    num_cards: int = 3,
    card_blizzard: int = 5,  # USE ACTUAL VALUE FROM SAVE
    relics: Optional[List[str]] = None,  # For egg auto-upgrades
) -> Tuple[List[str], int]:
    """
    Predict card reward and return (cards, new_counter).

    IMPORTANT: card_blizzard should come from save file for 100% accuracy!

    Egg relics auto-upgrade:
    - Molten Egg 2: ATTACK cards
    - Toxic Egg 2: SKILL cards
    - Frozen Egg: POWER cards
    """
    from core.content.cards import CardType

    state = GameRNGState(seed_str)
    state.set_counter(RNGStream.CARD, card_counter)

    # Use actual blizzard from save file
    reward_state = RewardState()
    reward_state.card_blizzard.offset = card_blizzard

    card_rng = state.get_rng(RNGStream.CARD)

    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=reward_state,
        act=act,
        player_class='WATCHER',
        ascension=20,
        room_type=room_type,
        num_cards=num_cards,
    )

    # Apply egg relic auto-upgrades
    relics = relics or []
    has_molten = 'Molten Egg 2' in relics
    has_toxic = 'Toxic Egg 2' in relics
    has_frozen = 'Frozen Egg' in relics

    card_names = []
    for c in cards:
        name = c.name
        # Check if this card would be auto-upgraded by egg relics
        # Use .value comparison to handle enum identity issues across imports
        card_type_value = c.card_type.value if hasattr(c.card_type, 'value') else str(c.card_type)
        if has_molten and card_type_value == 'ATTACK':
            name += '+'
        elif has_toxic and card_type_value == 'SKILL':
            name += '+'
        elif has_frozen and card_type_value == 'POWER':
            name += '+'
        card_names.append(name)

    return card_names, card_rng.counter


# =============================================================================
# DISPLAY
# =============================================================================

def display_full_state(save_data: dict, debug: bool = False):
    """Display complete game state with all RNG predictions."""
    seed_long = save_data.get('seed', 0)
    seed_str = long_to_seed(seed_long)
    floor = save_data.get('floor_num', 0)
    act = save_data.get('act_num', 1)

    # Game counters
    game_card_counter = save_data.get('card_seed_count', 0)
    game_relic_counter = save_data.get('relic_seed_count', 0)
    game_potion_counter = save_data.get('potion_seed_count', 0)
    game_monster_counter = save_data.get('monster_seed_count', 0)
    game_event_counter = save_data.get('event_seed_count', 0)

    # Simulate our state
    sim_state = simulate_full_path(save_data)

    # Current room
    current_room = save_data.get('current_room', '')
    room_type = '?'
    for java_name, symbol in ROOM_SYMBOLS.items():
        if java_name in current_room:
            room_type = symbol
            break

    print("\n" + "="*70)
    print("FULL RNG TRACKER - 100% ACCURACY MODE")
    print("="*70)

    # Basic info
    print(f"Seed: {seed_str}")
    print(f"Floor: {floor} | Act: {act} | Room: {room_type}")
    print(f"HP: {save_data.get('current_health', 0)}/{save_data.get('max_health', 0)} | Gold: {save_data.get('gold', 0)}")
    print()

    # RNG Sync Status
    print("-"*70)
    print("RNG SYNCHRONIZATION")
    print("-"*70)

    sync_status = "✓" if sim_state.card_counter == game_card_counter else "✗"
    card_blizzard = save_data.get('card_random_seed_randomizer', 5)
    print(f"  cardRng:    Game={game_card_counter:4d}  Sim={sim_state.card_counter:4d}  {sync_status}")
    print(f"  blizzard:   {card_blizzard:4d}  (pity timer: higher = more rares)")
    print(f"  relicRng:   Game={game_relic_counter:4d}")
    print(f"  potionRng:  Game={game_potion_counter:4d}")
    print(f"  monsterRng: Game={game_monster_counter:4d}")
    print(f"  eventRng:   Game={game_event_counter:4d}")
    print()

    # Deck
    print("-"*70)
    print("DECK")
    print("-"*70)
    cards = save_data.get('cards', [])
    deck_str = []
    for card in cards:
        name = card.get('id', '')
        upgrades = '+' * card.get('upgrades', 0)
        deck_str.append(f"{name}{upgrades}")
    print(f"  {', '.join(sorted(deck_str))}")
    print()

    # Relics
    print("-"*70)
    print("RELICS")
    print("-"*70)
    relics = save_data.get('relics', [])
    print(f"  {', '.join(relics)}")
    print()

    # Current room prediction
    if room_type in ['M', 'E', 'B']:
        print("-"*70)
        print(f"PREDICTION: Floor {floor} {room_type} Combat")
        print("-"*70)

        rt = 'elite' if room_type == 'E' else 'normal'
        # Get actual card_blizzard (pity timer) from save for 100% accuracy
        card_blizzard = save_data.get('card_random_seed_randomizer', 5)
        cards, new_counter = predict_card_reward(
            seed_str, game_card_counter, act, rt,
            card_blizzard=card_blizzard
        )
        print(f"  Card Reward: {cards}")
        print(f"  cardRng: {game_card_counter} -> {new_counter}")
        print()

    # If near boss, show boss relic prediction
    boss_floors = {16: 1, 33: 2, 51: 3}
    if floor in boss_floors or floor == boss_floors.get(act * 17 - 1, -1):
        print("-"*70)
        print(f"BOSS RELIC PREDICTION (Act {act})")
        print("-"*70)

        # Determine player class from save file path or default to WATCHER
        # The save file name contains the class (e.g., WATCHER.autosave)
        player_class = "WATCHER"  # Default, could be extracted from save path

        # Check if player still has their starter relic
        has_starter = any(
            r in relics
            for r in CLASS_STARTER_RELICS.values()
        )

        # Count boss relics already taken
        boss_relics_picked = save_data.get('metric_boss_relics', [])
        relics_taken = len(boss_relics_picked)

        predicted_relics = predict_boss_relics(
            seed_long,
            player_class=player_class,
            has_starter_relic=has_starter,
            relics_already_taken=relics_taken,
            already_owned_relics=relics,
        )
        print(f"  Offerings: {predicted_relics}")
        print(f"  Has starter relic: {has_starter}")
        print()

    # Card choice history
    print("-"*70)
    print("RECENT CARD CHOICES")
    print("-"*70)
    choices = save_data.get('metric_card_choices', [])
    for choice in choices[-5:]:
        f = choice.get('floor', '?')
        picked = choice.get('picked', 'SKIP')
        not_picked = choice.get('not_picked', [])
        all_cards = not_picked + ([picked] if picked != 'SKIP' else [])
        print(f"  F{f}: {all_cards} -> {picked}")
    print()

    # Debug mode: show full path
    if debug:
        print("-"*70)
        print("DEBUG: FULL PATH")
        print("-"*70)
        path = [p for p in save_data.get('metric_path_per_floor', []) if p]
        print(f"  {' '.join(path)}")
        print()


def watch_mode(debug: bool = False):
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
                display_full_state(save_data, debug)
            time.sleep(0.5)
        except FileNotFoundError:
            print("Save file not found. Start a game...")
            time.sleep(2)
        except KeyboardInterrupt:
            print("\nStopped.")
            break


# =============================================================================
# CLI
# =============================================================================

def test_boss_relics(seed_str: str, player_class: str = "WATCHER"):
    """Test boss relic prediction for a given seed."""
    from core.generation.relics import predict_boss_relic_pool

    seed = seed_to_long(seed_str)

    print(f"{'='*60}")
    print(f"BOSS RELIC PREDICTION TEST")
    print(f"Seed: {seed_str} ({seed})")
    print(f"Class: {player_class}")
    print(f"{'='*60}")

    # Show shuffled pool
    pool = predict_boss_relic_pool(seed, player_class)
    print(f"\nShuffled boss pool (first 10):")
    for i, r in enumerate(pool[:10]):
        marker = " *starter-upgrade*" if r in STARTER_UPGRADE_RELICS else ""
        print(f"  {i}: {r}{marker}")

    # Test with/without starter
    print(f"\nFirst boss selection:")
    print(f"  With starter:    {predict_boss_relics(seed, player_class, True)}")
    print(f"  Without starter: {predict_boss_relics(seed, player_class, False)}")

    print(f"\nSecond boss selection (after first pick):")
    print(f"  With starter:    {predict_boss_relics(seed, player_class, True, 1)}")
    print(f"  Without starter: {predict_boss_relics(seed, player_class, False, 1)}")

    print(f"\nThird boss selection (after two picks):")
    print(f"  With starter:    {predict_boss_relics(seed, player_class, True, 2)}")
    print(f"  Without starter: {predict_boss_relics(seed, player_class, False, 2)}")


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Full RNG Tracker")
    parser.add_argument("--watch", action="store_true", help="Watch mode")
    parser.add_argument("--debug", action="store_true", help="Debug mode")
    parser.add_argument("--test-boss", metavar="SEED", help="Test boss relic prediction for a seed")
    parser.add_argument("--class", dest="player_class", default="WATCHER",
                       choices=["IRONCLAD", "SILENT", "DEFECT", "WATCHER"],
                       help="Player class for testing")
    args = parser.parse_args()

    if args.test_boss:
        test_boss_relics(args.test_boss, args.player_class)
    elif args.watch:
        watch_mode(args.debug)
    else:
        try:
            save_data = read_save_file()
            display_full_state(save_data, args.debug)
        except FileNotFoundError:
            print(f"Save file not found: {SAVE_PATH}")
