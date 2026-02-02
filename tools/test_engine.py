#!/usr/bin/env python3
"""
Interactive Python Engine Tester - Run side-by-side with Java game.

Usage:
    uv run tools/test_engine.py --seed 1234567890

Then play the same seed in Java and compare states.
"""
import argparse
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent))

from core.game import GameRunner, GamePhase, GameAction
from core.generation.encounters import predict_all_acts
from core.state.rng import seed_to_long, long_to_seed


def format_state(runner: GameRunner) -> str:
    """Format current game state for display."""
    state = runner.run_state
    lines = [
        f"\n{'='*60}",
        f"PYTHON ENGINE STATE",
        f"{'='*60}",
        f"Seed: {runner.seed_string}",
        f"Floor: {state.floor} | Act: {state.act}",
        f"HP: {state.current_hp}/{state.max_hp}",
        f"Gold: {state.gold}",
        f"Phase: {runner.phase.name}",
        f"",
        f"Deck ({len(state.deck)} cards):",
    ]

    # Show deck summary
    card_counts = {}
    for card in state.deck:
        # Handle both dict and object formats
        if hasattr(card, 'id'):
            name = card.id
            upgraded = getattr(card, 'upgraded', False) or getattr(card, 'upgrades', 0) > 0
        else:
            name = card.get('id', 'Unknown')
            upgraded = card.get('upgrades', 0) > 0
        if upgraded:
            name += '+'
        card_counts[name] = card_counts.get(name, 0) + 1

    for name, count in sorted(card_counts.items()):
        lines.append(f"  {count}x {name}")

    lines.append(f"\nRelics ({len(state.relics)}):")
    for relic in state.relics[:10]:
        relic_name = getattr(relic, 'id', str(relic)) if hasattr(relic, 'id') else str(relic)
        lines.append(f"  - {relic_name}")
    if len(state.relics) > 10:
        lines.append(f"  ... and {len(state.relics) - 10} more")

    potions = getattr(state, 'potion_slots', [])
    lines.append(f"\nPotions: {potions}")

    return '\n'.join(lines)


def format_actions(actions: list) -> str:
    """Format available actions for display."""
    if not actions:
        return "No actions available"

    lines = ["\nAVAILABLE ACTIONS:"]
    for i, action in enumerate(actions):
        lines.append(f"  [{i}] {action}")
    return '\n'.join(lines)


def show_predictions(seed: str):
    """Show encounter predictions for the seed."""
    print(f"\n{'='*60}")
    print("ENCOUNTER PREDICTIONS")
    print(f"{'='*60}")

    acts = predict_all_acts(seed)

    for act_num in [1, 2, 3]:
        act_data = acts.get(f"act{act_num}", {})
        monsters = act_data.get("monsters", [])[:5]
        elites = act_data.get("elites", [])[:3]
        boss = act_data.get("boss", "Unknown")

        print(f"\nAct {act_num}:")
        print(f"  Monsters: {', '.join(monsters)}")
        print(f"  Elites: {', '.join(elites)}")
        print(f"  Boss: {boss}")


def main():
    parser = argparse.ArgumentParser(description="Interactive Python Engine Tester")
    parser.add_argument("--seed", type=str, default="1234567890", help="Seed to use")
    parser.add_argument("--ascension", type=int, default=20, help="Ascension level")
    args = parser.parse_args()

    seed = args.seed

    # Convert seed if needed
    if seed.lstrip('-').isdigit():
        seed_long = int(seed)
        seed_alpha = long_to_seed(seed_long & 0xFFFFFFFFFFFFFFFF)
    else:
        seed_long = seed_to_long(seed)
        seed_alpha = seed

    print(f"\n{'='*60}")
    print("PYTHON ENGINE TESTER")
    print(f"{'='*60}")
    print(f"Seed (alpha): {seed_alpha}")
    print(f"Seed (numeric): {seed_long}")
    print(f"Ascension: {args.ascension}")
    print(f"\nStart STS with this seed to compare.")
    print(f"{'='*60}")

    # Show predictions first
    show_predictions(seed)

    # Initialize the game runner
    print("\nInitializing Python engine...")
    try:
        runner = GameRunner(
            seed=seed_alpha,
            ascension=args.ascension,
            character="WATCHER",
            verbose=False
        )
        print("Engine initialized!")
    except Exception as e:
        print(f"ERROR initializing engine: {e}")
        print("\nThe engine may not be fully implemented yet.")
        print("For now, use the parity script to compare predictions:")
        print(f"  uv run scripts/dev/test_parity.py --seed {seed}")
        return

    # Interactive loop
    while True:
        print(format_state(runner))

        if runner.game_over:
            print("\n*** GAME OVER ***")
            stats = runner.get_run_statistics()
            print(f"Result: {'Victory' if stats.get('victory') else 'Defeat'}")
            print(f"Final Floor: {stats.get('floor')}")
            break

        actions = runner.get_available_actions()
        print(format_actions(actions))

        try:
            choice = input("\nEnter action number (or 'q' to quit, 'p' for predictions): ").strip()

            if choice.lower() == 'q':
                print("Exiting...")
                break
            elif choice.lower() == 'p':
                show_predictions(seed)
                continue
            elif choice.isdigit():
                idx = int(choice)
                if 0 <= idx < len(actions):
                    runner.take_action(actions[idx])
                else:
                    print(f"Invalid choice. Enter 0-{len(actions)-1}")
            else:
                print("Invalid input. Enter a number or 'q'/'p'")
        except KeyboardInterrupt:
            print("\nExiting...")
            break
        except EOFError:
            print("\nExiting...")
            break


if __name__ == "__main__":
    main()
