#!/usr/bin/env python3
"""
Seed Verification Tool

Generate predictions for a seed that can be verified against the actual game.
Start a game with the same seed and compare predictions at each floor.
"""

import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__)))))

from typing import List, Dict, Any, Optional
from dataclasses import dataclass, field

from packages.engine.state.game_rng import GameRNGState, RNGStream
from packages.engine.generation.rewards import generate_card_rewards, RewardState


@dataclass
class FloorPrediction:
    """Prediction for a single floor."""
    floor: int
    room_type: str
    card_reward: Optional[List[str]] = None
    card_rng_counter_before: int = 0
    card_rng_counter_after: int = 0
    notes: str = ""


@dataclass
class SeedPredictions:
    """All predictions for a seed."""
    seed: str
    character: str = "WATCHER"
    ascension: int = 20
    neow_choice: str = "NONE"
    floors: List[FloorPrediction] = field(default_factory=list)


def predict_card_reward(
    rng_state: GameRNGState,
    reward_state: RewardState,
    act: int,
    room_type: str = "normal",
    num_cards: int = 3,
) -> tuple[List[str], int, int]:
    """
    Generate card reward prediction.

    Returns: (card_names, counter_before, counter_after)
    """
    counter_before = rng_state.get_counter(RNGStream.CARD)
    card_rng = rng_state.get_rng(RNGStream.CARD)

    cards = generate_card_rewards(
        rng=card_rng,
        reward_state=reward_state,
        act=act,
        player_class="WATCHER",
        ascension=20,
        room_type=room_type,
        num_cards=num_cards,
    )

    counter_after = card_rng.counter
    rng_state.set_counter(RNGStream.CARD, counter_after)

    return [c.name for c in cards], counter_before, counter_after


def generate_act1_predictions(
    seed: str,
    neow_choice: str = "BOSS_SWAP",
    num_floors: int = 15,
) -> SeedPredictions:
    """
    Generate predictions for first N floors of Act 1.

    Args:
        seed: Seed string (e.g., "TEST123")
        neow_choice: Neow bonus choice
        num_floors: Number of floors to predict

    Returns:
        SeedPredictions with all floor predictions
    """
    predictions = SeedPredictions(seed=seed, neow_choice=neow_choice)

    # Initialize RNG state
    rng_state = GameRNGState(seed)
    rng_state.apply_neow_choice(neow_choice)

    reward_state = RewardState()

    print(f"\nSeed: {seed}")
    print(f"Neow Choice: {neow_choice}")
    print(f"Initial cardRng counter: {rng_state.get_counter(RNGStream.CARD)}")
    print("\n" + "="*60)

    # For each floor, predict card rewards for combat rooms
    floor = 1
    combats_predicted = 0

    while floor <= num_floors and floor <= 50:  # Act 1 is floors 1-16
        # Determine room type (simplified - actual depends on path chosen)
        if floor == 8:
            room_type = "elite"
        elif floor == 16:
            room_type = "boss"
        elif floor in [4, 9, 12]:  # Typical event floors
            room_type = "event"
        elif floor in [5, 15]:  # Typical shop floors
            room_type = "shop"
        elif floor in [6, 10]:  # Typical rest floors
            room_type = "rest"
        elif floor == 9:
            room_type = "treasure"
        else:
            room_type = "monster"

        pred = FloorPrediction(
            floor=floor,
            room_type=room_type,
        )

        # Handle different room types
        if room_type in ["monster", "elite", "boss"]:
            # Generate card reward prediction
            cards, before, after = predict_card_reward(
                rng_state, reward_state, act=1,
                room_type="elite" if room_type == "elite" else "normal",
            )
            pred.card_reward = cards
            pred.card_rng_counter_before = before
            pred.card_rng_counter_after = after
            combats_predicted += 1

            print(f"Floor {floor:2d} [{room_type.upper():7s}]: cardRng {before:3d} -> {after:3d}")
            print(f"         Cards: {cards}")

        elif room_type == "shop":
            # Shop consumes ~12 cardRng calls
            counter_before = rng_state.get_counter(RNGStream.CARD)
            rng_state.apply_shop()
            counter_after = rng_state.get_counter(RNGStream.CARD)
            pred.card_rng_counter_before = counter_before
            pred.card_rng_counter_after = counter_after
            pred.notes = "Shop: +12 cardRng"

            print(f"Floor {floor:2d} [SHOP   ]: cardRng {counter_before:3d} -> {counter_after:3d} (shop inventory)")

        elif room_type == "event":
            rng_state.apply_event()
            pred.notes = "Event (uses miscRng)"
            print(f"Floor {floor:2d} [EVENT  ]: cardRng unchanged (uses miscRng)")

        elif room_type == "rest":
            pred.notes = "Rest site"
            print(f"Floor {floor:2d} [REST   ]: cardRng unchanged")

        elif room_type == "treasure":
            rng_state.apply_treasure()
            pred.notes = "Treasure (uses treasureRng)"
            print(f"Floor {floor:2d} [CHEST  ]: cardRng unchanged (uses treasureRng)")

        predictions.floors.append(pred)
        floor += 1

    print("\n" + "="*60)
    print(f"Predicted {combats_predicted} combat card rewards")
    print(f"Final cardRng counter: {rng_state.get_counter(RNGStream.CARD)}")

    return predictions


def print_verification_checklist(predictions: SeedPredictions):
    """Print a checklist for manual verification."""
    print("\n" + "="*60)
    print("VERIFICATION CHECKLIST")
    print("="*60)
    print(f"\n1. Start new Watcher game with seed: {predictions.seed}")
    print(f"2. Choose Neow option: {predictions.neow_choice}")
    print("\n3. Verify card rewards at each combat:\n")

    for pred in predictions.floors:
        if pred.card_reward:
            print(f"   Floor {pred.floor:2d} ({pred.room_type}):")
            print(f"      Expected: {pred.card_reward}")
            print(f"      Actual:   [                                        ]")
            print(f"      Match:    [ ] Yes  [ ] No")
            print()


# =============================================================================
# CLI
# =============================================================================

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="Generate seed predictions for verification")
    parser.add_argument("seed", nargs="?", default="33J85JVCVSPJY", help="Seed string")
    parser.add_argument("--neow", default="BOSS_SWAP", help="Neow choice")
    parser.add_argument("--floors", type=int, default=15, help="Number of floors")
    parser.add_argument("--checklist", action="store_true", help="Print verification checklist")

    args = parser.parse_args()

    predictions = generate_act1_predictions(
        seed=args.seed,
        neow_choice=args.neow,
        num_floors=args.floors,
    )

    if args.checklist:
        print_verification_checklist(predictions)
    else:
        print("\nRun with --checklist to print a verification form")

    print("\n" + "="*60)
    print("TO VERIFY:")
    print("="*60)
    print(f"1. Launch Slay the Spire")
    print(f"2. New Run -> Watcher -> Set Seed: {args.seed}")
    print(f"3. Choose Neow: {args.neow}")
    print(f"4. Check card rewards match predictions above")
    print(f"5. After each act, save game and run:")
    print(f"   python -m core.comparison.state_comparator")
