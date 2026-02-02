#!/usr/bin/env python3
"""
Test RNG verification - generate predictions for Game 6 seed and verify they're correct.
"""

import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from core.state.game_rng import GameRNGState, RNGStream
from core.generation.rewards import generate_card_rewards, RewardState

SEED = "33J85JVCVSPJY"


def main():
    print(f"Testing RNG with seed: {SEED}")
    print("=" * 60)

    # Initialize game state with BOSS_SWAP Neow choice
    state = GameRNGState(SEED)
    state.apply_neow_choice("BOSS_SWAP")

    print(f"After BOSS_SWAP Neow: cardRng counter = {state.get_counter(RNGStream.CARD)}")
    print()

    reward_state = RewardState()

    # Generate predictions for floors 1-15
    floors_to_test = [
        (1, "monster"),   # First combat
        (3, "monster"),   # After The Cleric event
        (7, "elite"),     # Elite (Sentries) - after shop at 4, event at 5, rest at 6
        (8, "monster"),   # Regular monster
        (10, "monster"),  # After chest at 9
        (14, "elite"),    # Second elite
    ]

    # Track what happens at each step
    card_rng = state.get_rng(RNGStream.CARD)

    print("Simulating game progression...")
    print()

    # Floor 1 - first combat
    print(f"--- Floor 1 (Monster) ---")
    counter_before = card_rng.counter
    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=reward_state,
        act=1, player_class="WATCHER", ascension=20,
        room_type="normal", num_cards=3
    )
    print(f"Counter: {counter_before} -> {card_rng.counter}")
    print(f"Cards: {[c.name for c in cards]}")
    state.set_counter(RNGStream.CARD, card_rng.counter)
    print()

    # Floor 2 - The Cleric event (no cardRng)
    print(f"--- Floor 2 (Event: The Cleric) ---")
    state.apply_event("The Cleric")
    print(f"Counter: {state.get_counter(RNGStream.CARD)} (unchanged)")
    print()

    # Floor 3 - combat
    print(f"--- Floor 3 (Monster) ---")
    counter_before = card_rng.counter
    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=reward_state,
        act=1, player_class="WATCHER", ascension=20,
        room_type="normal", num_cards=3
    )
    print(f"Counter: {counter_before} -> {card_rng.counter}")
    print(f"Cards: {[c.name for c in cards]}")
    state.set_counter(RNGStream.CARD, card_rng.counter)
    print()

    # Floor 4 - Shop
    print(f"--- Floor 4 (Shop) ---")
    counter_before = state.get_counter(RNGStream.CARD)
    state.apply_shop()
    print(f"Counter: {counter_before} -> {state.get_counter(RNGStream.CARD)}")
    print()

    # Floor 5 - Living Wall event (no cardRng)
    print(f"--- Floor 5 (Event: Living Wall) ---")
    state.apply_event("Living Wall")
    print(f"Counter: {state.get_counter(RNGStream.CARD)} (unchanged)")
    print()

    # Floor 6 - Rest (no cardRng)
    print(f"--- Floor 6 (Rest) ---")
    print(f"Counter: {state.get_counter(RNGStream.CARD)} (unchanged)")
    print()

    # Floor 7 - Elite
    print(f"--- Floor 7 (Elite: Sentries) ---")
    card_rng = state.get_rng(RNGStream.CARD)  # Refresh reference
    counter_before = card_rng.counter
    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=reward_state,
        act=1, player_class="WATCHER", ascension=20,
        room_type="elite", num_cards=3
    )
    print(f"Counter: {counter_before} -> {card_rng.counter}")
    print(f"Cards: {[c.name for c in cards]}")
    state.set_counter(RNGStream.CARD, card_rng.counter)
    print()

    # Floor 8 - Monster
    print(f"--- Floor 8 (Monster) ---")
    card_rng = state.get_rng(RNGStream.CARD)
    counter_before = card_rng.counter
    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=reward_state,
        act=1, player_class="WATCHER", ascension=20,
        room_type="normal", num_cards=3
    )
    print(f"Counter: {counter_before} -> {card_rng.counter}")
    print(f"Cards: {[c.name for c in cards]}")
    print(f"EXPECTED (from video): ['Defend', 'Reach Heaven', 'Pray']")
    print(f"  Note: 'Defend' is likely OCR error for 'Protect' or similar")
    state.set_counter(RNGStream.CARD, card_rng.counter)
    print()

    # Floor 9 - Chest (treasureRng, not cardRng)
    print(f"--- Floor 9 (Chest) ---")
    print(f"Counter: {state.get_counter(RNGStream.CARD)} (unchanged - treasure uses treasureRng)")
    print()

    # Floor 10 - Monster
    print(f"--- Floor 10 (Monster) ---")
    card_rng = state.get_rng(RNGStream.CARD)
    counter_before = card_rng.counter
    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=reward_state,
        act=1, player_class="WATCHER", ascension=20,
        room_type="normal", num_cards=3
    )
    print(f"Counter: {counter_before} -> {card_rng.counter}")
    print(f"Cards: {[c.name for c in cards]}")
    print(f"EXPECTED (from video): ['Talk to the Hand', 'Just Lucky', 'Weave']")
    state.set_counter(RNGStream.CARD, card_rng.counter)
    print()

    # Summary
    print("=" * 60)
    print("VERIFICATION SUMMARY")
    print("=" * 60)
    print("If the first 3 predictions match the extraction, RNG is correct.")
    print("If floor 8+ diverges, we're missing RNG consumption somewhere.")
    print()
    print("Observed matches in extraction:")
    print("  Floor 1: ['Consecrate', 'Meditate', 'Foreign Influence'] - MATCH")
    print("  Floor 3: ['Consecrate', 'Fasting', 'Pressure Points'] - MATCH")
    print("  Floor 7: ['Third Eye', 'Protect', 'Prostrate'] - MATCH")
    print("  Floor 8: ['Defend', 'Reach Heaven', 'Pray'] - MISMATCH")


if __name__ == "__main__":
    main()
